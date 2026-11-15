//! Pens (decision 5; S5): derived from fence and gate pieces on hex edges,
//! never stored.
//!
//! A piece stands on the edge between two cells, named by their keys. A pen
//! is found by flooding from a cell across edges with no piece on them: if
//! the flood stays inside a bound and the pieces round it include a gate, the
//! cells it reached are a pen, whose id is its lowest cell key. The cells'
//! neighbours are the caller's (the real planet's five- and six-sided table,
//! or a flat grid in a test), so the rule never fakes the sphere.

use std::collections::{BTreeMap, BTreeSet};

/// The edge between two cells: the lower key in the high word, so the same
/// edge has one id whichever side names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge(pub u64);

impl Edge {
    pub fn new(a: u32, b: u32) -> Self {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Self((u64::from(lo) << 32) | u64::from(hi))
    }

    /// Its two cells, low then high.
    pub fn cells(self) -> (u32, u32) {
        ((self.0 >> 32) as u32, self.0 as u32)
    }
}

/// A pen: the cells inside a closed loop of pieces with a gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pen {
    /// Its lowest cell key: stable while the loop stands.
    pub id: u32,
    /// Every cell inside, sorted.
    pub cells: Vec<u32>,
    /// The pieces round it, and how many of them are gates.
    pub border: Vec<Edge>,
    pub gates: usize,
}

/// Why the cells round a start are not a pen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PenRefusal {
    /// The flood got out: the loop is broken, or there is none.
    Open,
    /// Closed, but with no gate.
    NoGate,
}

/// The pen `start` is in, if any. `neighbours(cell)` lists a cell's
/// neighbours, `None` where it cannot say (a cell off the resident set, which
/// counts as an escape). `pieces` maps each standing piece's edge to whether
/// it is a gate. At most `max_cells` cells are visited.
pub fn find_pen(
    start: u32,
    neighbours: impl Fn(u32) -> Option<Vec<u32>>,
    pieces: &BTreeMap<Edge, bool>,
    max_cells: usize,
) -> Result<Pen, PenRefusal> {
    let mut inside = BTreeSet::from([start]);
    let mut frontier = vec![start];
    let mut border = BTreeSet::new();
    while let Some(cell) = frontier.pop() {
        let Some(around) = neighbours(cell) else {
            return Err(PenRefusal::Open);
        };
        for next in around {
            let edge = Edge::new(cell, next);
            if pieces.contains_key(&edge) {
                border.insert(edge);
            } else if inside.insert(next) {
                if inside.len() > max_cells {
                    return Err(PenRefusal::Open);
                }
                frontier.push(next);
            }
        }
    }
    let gates = border.iter().filter(|e| pieces[e]).count();
    if gates == 0 {
        return Err(PenRefusal::NoGate);
    }
    let cells: Vec<u32> = inside.into_iter().collect();
    Ok(Pen {
        id: cells[0],
        cells,
        border: border.into_iter().collect(),
        gates,
    })
}
