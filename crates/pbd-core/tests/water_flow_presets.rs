//! `pbd_core::liquid` against the mockup's own results (`water-flow` tasks 2.1
//! and 2.2): every preset of M25's Water sim, as `tools/water_flow_fixture.mjs`
//! ran it through `docs/mockups/water-flow/liquid.js`, must settle in the same
//! ticks to the same field, cell for cell.

use pbd_core::liquid::{Cell, DOWN, Grid, LOOP_MAX, Queue, Rules, SIDE, UP};

/// The mockup's sides, in its order.
const DIRS: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

fn hex_dist(q: i32, r: i32) -> i32 {
    q.abs().max(r.abs()).max((q + r).abs())
}

/// One hex patch: radius `n` cells in axial coordinates, layers `y0..=y1`.
struct Patch {
    n: i32,
    y0: i32,
    y1: i32,
    cells: Vec<Cell>,
}

impl Patch {
    fn new(n: i32, y0: i32, y1: i32) -> Self {
        let side = (2 * n + 1) as usize;
        let ny = (y1 - y0 + 1) as usize;
        Self {
            n,
            y0,
            y1,
            cells: vec![Cell::Air; side * side * ny],
        }
    }
    fn inside(&self, q: i32, r: i32, y: i32) -> bool {
        hex_dist(q, r) <= self.n && y >= self.y0 && y <= self.y1
    }
    fn idx(&self, q: i32, r: i32, y: i32) -> u32 {
        let side = 2 * self.n + 1;
        let ny = self.y1 - self.y0 + 1;
        (((q + self.n) * side + (r + self.n)) * ny + (y - self.y0)) as u32
    }
    fn unidx(&self, i: u32) -> (i32, i32, i32) {
        let side = 2 * self.n + 1;
        let ny = self.y1 - self.y0 + 1;
        let i = i as i32;
        let y = i % ny + self.y0;
        let t = i / ny;
        (t / side - self.n, t % side - self.n, y)
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
        let (q, r, y) = self.unidx(at);
        let mut out = [None; 8];
        let at = |q: i32, r: i32, y: i32| self.inside(q, r, y).then(|| self.idx(q, r, y));
        out[UP] = at(q, r, y + 1);
        for (k, (dq, dr)) in DIRS.iter().enumerate() {
            out[SIDE + k] = at(q + dq, r + dr, y);
        }
        out[DOWN] = at(q, r, y - 1);
        out
    }
}

struct Preset {
    name: String,
    patch: Patch,
    renew: u8,
    ticks: usize,
    counts: (usize, usize, usize),
    edits: Vec<(bool, i32, i32, i32, usize)>,
    expected: Vec<(i32, i32, i32, Cell)>,
}

fn presets() -> Vec<Preset> {
    let text = include_str!("fixtures/water_flow_presets.txt");
    let mut out = Vec::new();
    let mut current: Option<Preset> = None;
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        match f.first().copied() {
            Some("preset") => {
                let n = |i: usize| f[i].parse::<i32>().unwrap();
                current = Some(Preset {
                    name: f[1].to_string(),
                    patch: Patch::new(n(3), n(4), n(5)),
                    renew: f[7].parse().unwrap(),
                    ticks: f[9].parse().unwrap(),
                    counts: (
                        f[11].parse().unwrap(),
                        f[13].parse().unwrap(),
                        f[15].parse().unwrap(),
                    ),
                    edits: Vec::new(),
                    expected: Vec::new(),
                });
            }
            Some("ground") => {
                let p = current.as_mut().unwrap();
                for entry in &f[1..] {
                    let sea = entry.ends_with('s');
                    let v: Vec<i32> = entry
                        .trim_end_matches('s')
                        .split(',')
                        .map(|x| x.parse().unwrap())
                        .collect();
                    let (q, r, h) = (v[0], v[1], v[2]);
                    for y in p.patch.y0..h {
                        let i = p.patch.idx(q, r, y);
                        p.patch.cells[i as usize] = Cell::Neutral;
                    }
                    if sea {
                        for y in h..0 {
                            let i = p.patch.idx(q, r, y);
                            p.patch.cells[i as usize] = Cell::Source;
                        }
                    }
                }
            }
            Some("edit") => {
                let n = |i: usize| f[i].parse::<i32>().unwrap();
                current.as_mut().unwrap().edits.push((
                    f[1] == "pour",
                    n(2),
                    n(3),
                    n(4),
                    f[5].parse().unwrap(),
                ));
            }
            Some("cell") => {
                let n = |i: usize| f[i].parse::<i32>().unwrap();
                let s = f[4];
                let cell = match &s[..1] {
                    "s" => Cell::Source,
                    k => Cell::Flowing {
                        level: s[1..].parse().unwrap(),
                        falling: k == "d",
                    },
                };
                current
                    .as_mut()
                    .unwrap()
                    .expected
                    .push((n(1), n(2), n(3), cell));
            }
            Some("end") => out.push(current.take().unwrap()),
            _ => {}
        }
    }
    out
}

#[test]
fn every_mockup_preset_settles_to_the_mockups_field() {
    let all = presets();
    assert_eq!(all.len(), 8, "spring, channel, basin, pair, runaway, recede, pond, fall");
    for mut preset in all {
        let rules = Rules {
            renew: preset.renew,
            range: 8,
        };
        let mut queue = Queue::new();
        let mut tick = 0usize;
        let mut changed = Vec::new();
        for &(pour, q, r, y, at) in &preset.edits {
            while tick < at {
                queue.tick(&mut preset.patch, rules, LOOP_MAX, &mut changed);
                tick += 1;
            }
            let i = preset.patch.idx(q, r, y);
            preset.patch.set(i, if pour { Cell::Source } else { Cell::Air });
            queue.touch(&preset.patch, i);
        }
        let mut ticks = 0;
        while !queue.is_empty() && ticks < 5000 {
            queue.tick(&mut preset.patch, rules, LOOP_MAX, &mut changed);
            ticks += 1;
        }
        assert_eq!(ticks, preset.ticks, "{}: ticks to settle", preset.name);
        let mut counts = (0, 0, 0);
        let mut field = Vec::new();
        for (i, cell) in preset.patch.cells.iter().enumerate() {
            match cell {
                Cell::Source => counts.0 += 1,
                Cell::Flowing { falling, .. } => {
                    counts.1 += 1;
                    counts.2 += usize::from(*falling);
                }
                _ => continue,
            }
            let (q, r, y) = preset.patch.unidx(i as u32);
            field.push((q, r, y, *cell));
        }
        assert_eq!(
            counts, preset.counts,
            "{}: sources, flowing, falling",
            preset.name
        );
        assert_eq!(field, preset.expected, "{}: the field, cell for cell", preset.name);
    }
}

/// Decision 6: settling is a function of the sources and the solids. The
/// spring re-run from its source alone lands on the same field.
#[test]
fn a_settled_field_is_the_same_from_scratch() {
    let mut spring = presets().into_iter().next().unwrap();
    let rules = Rules::default();
    let mut queue = Queue::new();
    for &(_, q, r, y, _) in &spring.edits {
        let i = spring.patch.idx(q, r, y);
        spring.patch.set(i, Cell::Source);
        queue.touch(&spring.patch, i);
    }
    queue.settle(&mut spring.patch, rules, 5000);
    let settled = spring.patch.cells.clone();
    // Dry every flowing cell, queue every source, settle again.
    for cell in spring.patch.cells.iter_mut() {
        if matches!(cell, Cell::Flowing { .. }) {
            *cell = Cell::Air;
        }
    }
    for i in 0..spring.patch.cells.len() as u32 {
        if spring.patch.cell(i) == Cell::Source {
            queue.touch(&spring.patch, i);
        }
    }
    queue.settle(&mut spring.patch, rules, 5000);
    assert_eq!(spring.patch.cells, settled);
}
