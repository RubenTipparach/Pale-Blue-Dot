//! Water that flows: Luanti's liquid rule on the hex grid (`water-flow`), and
//! the flood that fills what a carve or a dig opens onto the sea
//! (`volumetric-water` phase 2).
//!
//! Written from the rule as `openspec/changes/water-flow/design.md` describes
//! it (Context and decisions 1 to 6), not translated from Luanti's C++
//! (decision 8). The mockup's port, `docs/mockups/water-flow/liquid.js`, is
//! the reference for behaviour: `tests/water_flow_presets.rs` runs its presets
//! from a fixture the page's own code wrote and holds this module to the same
//! field, cell for cell.
//!
//! **A cell** is a source (`Material::Water`), a flowing cell at a level from 7
//! beside its source down to 0 (`Material::Flow*`, `Material::Fall*` when the
//! cell below is open), air, or something neutral that water neither enters
//! nor leaves: every solid block, and every lamp, so water never washes away a
//! thing a player made.
//!
//! **The neighbourhood is eight**: up, the six sides (five on a pentagon, the
//! sixth absent), down; Luanti's order, upper before same before lower.
//!
//! **Only edits start flow** (decision 5): nothing here queues a cell by
//! itself. A caller queues the cells an edit touched, and generated water
//! stays where the generator put it until someone digs beside it.

use crate::terrain::Material;
use std::collections::{HashSet, VecDeque};

/// The highest flowing level, beside a source.
pub const LEVEL_MAX: u8 = 7;
/// What a flowing cell above gives the cell under it: Luanti's
/// `WATER_DROP_BOOST`.
pub const DROP_BOOST: u8 = 4;
/// How far water runs from a source or a fall: Luanti's `liquid_range`
/// (decision 3), every level 0 to 7.
pub const RANGE: u8 = 8;
/// Sources around a cell that make it a source: three on the hex grid
/// (decision 2), where Luanti's two runs away.
pub const RENEW: u8 = 3;
/// Seconds between ticks: Luanti's `liquid_update`.
pub const TICK_S: f32 = 1.0;
/// The most cells one tick looks at: Luanti's `liquid_loop_max`.
pub const LOOP_MAX: usize = 100_000;

/// Where a neighbour stands.
pub const UP: usize = 0;
/// The first side; sides are `1..=6`.
pub const SIDE: usize = 1;
pub const DOWN: usize = 7;

/// What the rule sees in a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// Solid, a lamp, anything water does not enter.
    Neutral,
    /// Floodable.
    Air,
    Source,
    Flowing {
        level: u8,
        falling: bool,
    },
}

impl Cell {
    pub fn of(material: Material) -> Cell {
        match material {
            Material::Air => Cell::Air,
            Material::Water => Cell::Source,
            other => match other.flow() {
                Some((level, falling)) => Cell::Flowing { level, falling },
                None => Cell::Neutral,
            },
        }
    }

    /// The material a cell the rule wrote holds. `Neutral` is never written.
    pub fn material(self) -> Option<Material> {
        match self {
            Cell::Neutral => None,
            Cell::Air => Some(Material::Air),
            Cell::Source => Some(Material::Water),
            Cell::Flowing { level, falling } => Some(Material::flowing(level, falling)),
        }
    }

    pub fn is_liquid(self) -> bool {
        matches!(self, Cell::Source | Cell::Flowing { .. })
    }
}

/// How high a water cell's surface stands in its layer, 0..1: a source full,
/// a flowing cell at Luanti's `(level + 0.5) / 8` (design 7e), anything else
/// nought.
pub fn top_fraction(material: Material) -> f32 {
    match Cell::of(material) {
        Cell::Source => 1.0,
        Cell::Flowing { level, .. } => (f32::from(level) + 0.5) / 8.0,
        _ => 0.0,
    }
}

/// The rule's two settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Sources round a cell that renew it as a source.
    pub renew: u8,
    /// How many levels count as liquid, 1 to 8.
    pub range: u8,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            renew: RENEW,
            range: RANGE,
        }
    }
}

/// What one update decided: the cell's new state where it changed, and which
/// neighbours to queue, in the order to queue them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub changed: Option<Cell>,
    pushes: [u8; 24],
    len: u8,
}

impl Outcome {
    fn new() -> Self {
        Self {
            changed: None,
            pushes: [0; 24],
            len: 0,
        }
    }

    fn push(&mut self, neighbour: usize) {
        self.pushes[usize::from(self.len)] = neighbour as u8;
        self.len += 1;
    }

    /// The neighbours to queue (`UP`, `SIDE..`, `DOWN`), in order; a
    /// neighbour may appear twice, and the queue keeps the first.
    pub fn pushes(&self) -> &[u8] {
        &self.pushes[..usize::from(self.len)]
    }
}

/// The highest level a flowing neighbour offers, against `current`.
fn offered(level: u8, falling: bool, place: usize, current: i8) -> i8 {
    let level = level as i8;
    match place {
        // Water falling onto this cell: its level plus the boost, at most
        // seven.
        UP if level + DROP_BOOST as i8 > current => (level + DROP_BOOST as i8).min(LEVEL_MAX as i8),
        UP if level > current => level,
        // A side gives one less, unless it is falling: water drops before it
        // spreads.
        DOWN | UP => current,
        _ if !falling && level > 0 && level - 1 > current => level - 1,
        _ => current,
    }
}

/// One cell's update from its eight neighbours (`None` is off the region or
/// a pentagon's missing side, and is neutral).
pub fn transform(here: Cell, around: &[Option<Cell>; 8], rules: Rules) -> Outcome {
    let mut out = Outcome::new();
    if here == Cell::Neutral {
        return out;
    }
    let mut sources = 0u8;
    let mut falling = false;
    let (mut flows, mut airs) = ([false; 8], [false; 8]);
    for (place, cell) in around.iter().enumerate() {
        match cell {
            Some(Cell::Air) => {
                airs[place] = true;
                // A liquid queues the air beside and under it, changed or not.
                if place != UP && here != Cell::Air {
                    out.push(place);
                }
                falling |= place == DOWN;
            }
            Some(Cell::Source) => sources += u8::from(place != DOWN),
            Some(Cell::Flowing { .. }) => {
                flows[place] = true;
                falling |= place == DOWN;
            }
            Some(Cell::Neutral) | None => {}
        }
    }
    let range = rules.range.min(LEVEL_MAX + 1);
    let lowest = (LEVEL_MAX + 1 - range) as i8;
    let new = if sources >= rules.renew || here == Cell::Source {
        Cell::Source
    } else {
        let level = if sources >= 1 {
            LEVEL_MAX as i8
        } else {
            let mut max = -1i8;
            for (place, cell) in around.iter().enumerate() {
                if let Some(Cell::Flowing { level, falling }) = cell {
                    max = offered(*level, *falling, place, max);
                }
            }
            max
        };
        if level >= lowest {
            Cell::Flowing {
                level: level as u8,
                falling,
            }
        } else {
            Cell::Air
        }
    };
    if new == here {
        return out;
    }
    out.changed = Some(new);
    if new.is_liquid() {
        for (place, &flow) in flows.iter().enumerate() {
            if flow && place != UP {
                out.push(place);
            }
        }
        for (place, &air) in airs.iter().enumerate() {
            if air && place != UP {
                out.push(place);
            }
        }
    } else {
        // This water has gone, so the flowing water it fed may follow.
        for (place, &flow) in flows.iter().enumerate() {
            if flow {
                out.push(place);
            }
        }
    }
    out
}

/// A region of cells the rule runs over, each named by a `u32`.
pub trait Grid {
    /// The cell's state.
    fn cell(&self, at: u32) -> Cell;
    /// Write the rule's answer.
    fn set(&mut self, at: u32, cell: Cell);
    /// Its eight neighbours, `None` off the region.
    fn around(&self, at: u32) -> [Option<u32>; 8];
}

/// The cells waiting to be looked at, in the order they were queued: Luanti's
/// `UniqueQueue`. The set is a keyed lookup only; the order is the deque's.
#[derive(Clone, Debug, Default)]
pub struct Queue {
    order: VecDeque<u32>,
    waiting: HashSet<u32>,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a cell, unless it is already waiting.
    pub fn push(&mut self, at: u32) {
        if self.waiting.insert(at) {
            self.order.push_back(at);
        }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn clear(&mut self) {
        self.order.clear();
        self.waiting.clear();
    }

    /// An edit at `at`: the cell and its eight neighbours.
    pub fn touch(&mut self, grid: &impl Grid, at: u32) {
        self.push(at);
        for neighbour in grid.around(at).into_iter().flatten() {
            self.push(neighbour);
        }
    }

    /// One tick: at most the queue's length at its start, and at most `max`.
    /// Every cell that changed is appended to `changed`, with its new state.
    /// Returns how many changed.
    pub fn tick(
        &mut self,
        grid: &mut impl Grid,
        rules: Rules,
        max: usize,
        changed: &mut Vec<(u32, Cell)>,
    ) -> usize {
        let count = self.order.len().min(max);
        let mut made = 0;
        for _ in 0..count {
            let Some(at) = self.order.pop_front() else {
                break;
            };
            self.waiting.remove(&at);
            let ids = grid.around(at);
            let cells = ids.map(|id| id.map(|id| grid.cell(id)));
            let outcome = transform(grid.cell(at), &cells, rules);
            if let Some(cell) = outcome.changed {
                grid.set(at, cell);
                changed.push((at, cell));
                made += 1;
            }
            for &place in outcome.pushes() {
                if let Some(id) = ids[usize::from(place)] {
                    self.push(id);
                }
            }
        }
        made
    }

    /// Tick until nothing is queued or `ticks` have run: how many ran.
    pub fn settle(&mut self, grid: &mut impl Grid, rules: Rules, ticks: usize) -> usize {
        let mut changed = Vec::new();
        let mut ran = 0;
        while !self.is_empty() && ran < ticks {
            self.tick(grid, rules, LOOP_MAX, &mut changed);
            ran += 1;
        }
        ran
    }
}

/// Phase 2 of `volumetric-water`: from each `seed` (a water cell), fill every
/// connected air cell across the sides and downward for which `fills` holds,
/// as a source. Never upward: water does not climb (design, phase 2). Stops
/// after `limit` cells, so a flood is bounded whatever the region. Returns
/// the cells filled, in the order they filled.
pub fn flood(
    grid: &mut impl Grid,
    seeds: impl IntoIterator<Item = u32>,
    fills: impl Fn(u32) -> bool,
    limit: usize,
) -> Vec<u32> {
    let mut filled = Vec::new();
    let mut open: VecDeque<u32> = seeds.into_iter().collect();
    while let Some(at) = open.pop_front() {
        let around = grid.around(at);
        for neighbour in around[SIDE..].iter().flatten() {
            if filled.len() >= limit {
                return filled;
            }
            if grid.cell(*neighbour) == Cell::Air && fills(*neighbour) {
                grid.set(*neighbour, Cell::Source);
                filled.push(*neighbour);
                open.push_back(*neighbour);
            }
        }
    }
    filled
}

#[cfg(test)]
mod tests {
    use super::*;

    const F7: Cell = Cell::Flowing {
        level: 7,
        falling: false,
    };

    fn flowing(level: u8, falling: bool) -> Cell {
        Cell::Flowing { level, falling }
    }

    /// Eight neighbours, all neutral but those given.
    fn around(given: &[(usize, Cell)]) -> [Option<Cell>; 8] {
        let mut cells = [Some(Cell::Neutral); 8];
        for &(place, cell) in given {
            cells[place] = Some(cell);
        }
        cells
    }

    #[test]
    fn every_material_round_trips_through_the_rule() {
        assert_eq!(Cell::of(Material::Water), Cell::Source);
        assert_eq!(Cell::of(Material::Air), Cell::Air);
        assert_eq!(Cell::of(Material::Stone), Cell::Neutral);
        assert_eq!(
            Cell::of(Material::Torch),
            Cell::Neutral,
            "a lamp is never washed away"
        );
        for level in 0..=7 {
            for falling in [false, true] {
                let m = Material::flowing(level, falling);
                assert_eq!(m.flow(), Some((level, falling)));
                assert!(m.is_water());
                assert_eq!(Cell::of(m).material(), Some(m));
            }
        }
        assert_eq!(Material::Water.flow(), None);
        assert!(Material::Water.is_water() && !Material::Air.is_water());
    }

    /// Falling first: a flowing cell over air falls, and offers nothing to its
    /// sides while it does.
    #[test]
    fn water_falls_before_it_spreads() {
        // Air beside a source on a floor: level 7.
        let out = transform(
            Cell::Air,
            &around(&[(SIDE, Cell::Source)]),
            Rules::default(),
        );
        assert_eq!(out.changed, Some(F7));
        // The same with air under it: level 7, falling.
        let out = transform(
            Cell::Air,
            &around(&[(SIDE, Cell::Source), (DOWN, Cell::Air)]),
            Rules::default(),
        );
        assert_eq!(out.changed, Some(flowing(7, true)));
        // A falling side offers nothing; a still one its level less one.
        let falling = transform(
            Cell::Air,
            &around(&[(SIDE + 2, flowing(6, true))]),
            Rules::default(),
        );
        assert_eq!(falling.changed, None, "air next to a fall stays air");
        let still = transform(
            Cell::Air,
            &around(&[(SIDE + 2, flowing(6, false))]),
            Rules::default(),
        );
        assert_eq!(still.changed, Some(flowing(5, false)));
        // Water falling onto a cell gives its level plus four, at most seven.
        let fed = transform(
            Cell::Air,
            &around(&[(UP, flowing(1, true))]),
            Rules::default(),
        );
        assert_eq!(fed.changed, Some(flowing(5, false)));
        let fed = transform(
            Cell::Air,
            &around(&[(UP, flowing(5, true))]),
            Rules::default(),
        );
        assert_eq!(fed.changed, Some(flowing(7, false)));
    }

    /// Spreading stops at the range, and a cell whose feed has gone recedes.
    #[test]
    fn a_film_at_level_nought_feeds_nothing_and_a_cut_off_cell_dries() {
        let out = transform(
            Cell::Air,
            &around(&[(SIDE, flowing(0, false))]),
            Rules::default(),
        );
        assert_eq!(out.changed, None, "level 0 is the last cell of a sheet");
        let out = transform(flowing(3, false), &around(&[]), Rules::default());
        assert_eq!(out.changed, Some(Cell::Air), "nothing feeds it: it recedes");
        // A shorter range dries the thin end of a sheet.
        let short = Rules {
            range: 4,
            ..Rules::default()
        };
        let out = transform(Cell::Air, &around(&[(SIDE, flowing(4, false))]), short);
        assert_eq!(out.changed, None, "level 3 is out of a range of four");
    }

    /// Three same-level or upper sources renew a source; two do not; the
    /// source below is not counted.
    #[test]
    fn three_sources_renew_and_two_do_not() {
        let two = around(&[(SIDE, Cell::Source), (SIDE + 1, Cell::Source)]);
        assert_eq!(
            transform(Cell::Air, &two, Rules::default()).changed,
            Some(F7)
        );
        let three = around(&[
            (SIDE, Cell::Source),
            (SIDE + 1, Cell::Source),
            (UP, Cell::Source),
        ]);
        assert_eq!(
            transform(Cell::Air, &three, Rules::default()).changed,
            Some(Cell::Source)
        );
        let under = around(&[
            (SIDE, Cell::Source),
            (SIDE + 1, Cell::Source),
            (DOWN, Cell::Source),
        ]);
        assert_eq!(
            transform(Cell::Air, &under, Rules::default()).changed,
            Some(F7),
            "the source below is not counted"
        );
        // Luanti's own two, which the hex grid runs away with.
        let luanti = Rules {
            renew: 2,
            ..Rules::default()
        };
        assert_eq!(
            transform(Cell::Air, &two, luanti).changed,
            Some(Cell::Source)
        );
    }

    /// A pentagon has five sides: the sixth is absent, and the rule counts
    /// what is there. Its centre fed by three of its five renews, and the
    /// source at its centre runs to all five.
    #[test]
    fn a_pentagon_counts_its_five_sides() {
        let mut cells = around(&[
            (SIDE, Cell::Source),
            (SIDE + 2, Cell::Source),
            (SIDE + 4, Cell::Source),
        ]);
        cells[SIDE + 5] = None;
        assert_eq!(
            transform(Cell::Air, &cells, Rules::default()).changed,
            Some(Cell::Source)
        );
        let mut grid = Patch::pentagon();
        let mut queue = Queue::new();
        grid.set(0, Cell::Source);
        queue.touch(&grid, 0);
        queue.settle(&mut grid, Rules::default(), 100);
        for side in 1..=5 {
            assert_eq!(grid.cell(side), F7, "side {side}");
        }
        for side in 6..=10 {
            assert_eq!(
                grid.cell(side),
                flowing(6, false),
                "the ring outside: one level lower"
            );
        }
    }

    /// A small region with explicit neighbour tables: one layer of a pentagon
    /// with its five sides and a ring of five beyond them, on a floor.
    struct Patch {
        cells: Vec<Cell>,
        sides: Vec<Vec<u32>>,
    }

    impl Patch {
        fn pentagon() -> Self {
            // 0 the pentagon; 1..=5 its sides (each touching 0, its two ring
            // neighbours, and one outer cell 6..=10); 6..=10 the outer ring.
            let mut sides = vec![vec![1, 2, 3, 4, 5]];
            for i in 0..5u32 {
                let prev = 1 + (i + 4) % 5;
                let next = 1 + (i + 1) % 5;
                sides.push(vec![0, prev, next, 6 + i]);
            }
            for i in 0..5u32 {
                sides.push(vec![1 + i]);
            }
            Self {
                cells: vec![Cell::Air; 11],
                sides,
            }
        }
    }

    impl Grid for Patch {
        fn cell(&self, at: u32) -> Cell {
            self.cells[at as usize]
        }
        fn set(&mut self, at: u32, cell: Cell) {
            self.cells[at as usize] = cell;
        }
        fn around(&self, at: u32) -> [Option<u32>; 8] {
            let mut out = [None; 8];
            for (k, &n) in self.sides[at as usize].iter().enumerate() {
                out[SIDE + k] = Some(n);
            }
            out
        }
    }

    #[test]
    fn a_flood_fills_what_is_open_and_never_climbs() {
        // A column of cells 0..4 bottom up, all air, a source at the top of a
        // neighbour column 5..9: cell 8 beside cell 3.
        struct Two(Vec<Cell>);
        impl Grid for Two {
            fn cell(&self, at: u32) -> Cell {
                self.0[at as usize]
            }
            fn set(&mut self, at: u32, cell: Cell) {
                self.0[at as usize] = cell;
            }
            fn around(&self, at: u32) -> [Option<u32>; 8] {
                let (col, layer) = (at / 5, at % 5);
                let mut out = [None; 8];
                if layer < 4 {
                    out[UP] = Some(at + 1);
                }
                if layer > 0 {
                    out[DOWN] = Some(at - 1);
                }
                out[SIDE] = Some((1 - col) * 5 + layer);
                out
            }
        }
        let mut grid = Two(vec![Cell::Air; 10]);
        grid.0[8] = Cell::Source;
        let filled = flood(&mut grid, [8], |_| true, 100);
        // Across to 3, then down 2, 1, 0 in that column and in its own; 4
        // (above) and 9 (above) stay dry.
        assert_eq!(grid.cell(3), Cell::Source);
        assert_eq!(grid.cell(0), Cell::Source);
        assert_eq!(grid.cell(5), Cell::Source);
        assert_eq!(grid.cell(4), Cell::Air, "never upward");
        assert_eq!(grid.cell(9), Cell::Air, "never upward");
        assert_eq!(filled.len(), 7);
        // A limit bounds it.
        let mut grid = Two(vec![Cell::Air; 10]);
        grid.0[8] = Cell::Source;
        assert_eq!(flood(&mut grid, [8], |_| true, 2).len(), 2);
        // And the predicate: nothing at or above layer 2.
        let mut grid = Two(vec![Cell::Air; 10]);
        grid.0[8] = Cell::Source;
        flood(&mut grid, [8], |at| at % 5 < 2, 100);
        assert_eq!(grid.cell(3), Cell::Air, "a cell the flood may not fill");
        assert_eq!(grid.cell(1), Cell::Air, "nor what it would have reached");
    }
}
