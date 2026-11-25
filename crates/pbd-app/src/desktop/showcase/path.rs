//! The showcase bot's path planner (`bot-path-planner`): A* over cells and
//! the layer the feet stand in, on the live ground, digging included. A plan
//! is its steps, each naming the blocks dug before the walker steps in.
//! Nothing in the game's rules uses it; it is how the bot finds its way.

#![allow(clippy::type_complexity)] // Bevy system parameters and search tables.
use bevy::math::Vec3;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// What one block is to a walker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Block {
    /// Air, or nothing a walker bumps into.
    Open,
    /// Water: waded at the feet, never stood on.
    Water,
    /// Solid, and dug at this cost, metres of walking.
    Dig(f32),
    /// Solid and never dug (a station, a lamp, unloaded ground).
    Never,
}

impl Block {
    pub fn solid(self) -> bool {
        matches!(self, Block::Dig(_) | Block::Never)
    }
}

/// The ground as the planner reads it: finest cells by index.
pub trait Ground {
    fn neighbours(&self, cell: usize) -> Vec<usize>;
    /// The cell's centre, a unit direction.
    fn centre(&self, cell: usize) -> Vec3;
    fn block(&self, cell: usize, layer: usize) -> Block;
}

/// One step of a plan: the cell and layer to stand in, and the blocks to dig
/// first (cell, layer), in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub cell: usize,
    pub layer: usize,
    pub digs: Vec<(usize, usize)>,
}

/// A plan from the walker's state toward the goal: its steps, and whether
/// the last one is the goal's cell (else a leg, planned on from its end).
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub arrives: bool,
}

/// States expanded at most in one search.
const MAX_EXPANDED: usize = 20_000;
/// The longest leg planned at once, m, and the margin round it.
pub const LEG_M: f32 = 60.0;
const MARGIN_M: f32 = 18.0;
/// The cost of a step up, m, on top of the walk.
const STEP_UP_M: f32 = 0.5;
/// Wading costs this many times walking.
const WADE: f32 = 6.0;

/// Plan from `(start, layer)` toward the cell `goal`, over cells within the
/// leg's span (`radius_m` the planet's radius). `None` when the start is not
/// in the ground's region at all.
pub fn plan(
    ground: &dyn Ground,
    start: usize,
    layer: usize,
    goal: usize,
    radius_m: f32,
) -> Option<Plan> {
    let metres = |a: usize, b: usize| ground.centre(a).angle_between(ground.centre(b)) * radius_m;
    let span = metres(start, goal);
    // The leg's end: the goal itself, or the cell 60 m along toward it.
    let reach = span.min(LEG_M) + MARGIN_M;
    let goal_dir = ground.centre(goal);
    let start_dir = ground.centre(start);
    let toward = |d: Vec3| d.angle_between(goal_dir) * radius_m;
    let inside = |c: usize| {
        let d = ground.centre(c);
        d.angle_between(start_dir) * radius_m <= reach
    };
    let mut best: HashMap<(usize, usize), f32> = HashMap::new();
    let mut from: HashMap<(usize, usize), ((usize, usize), Vec<(usize, usize)>)> = HashMap::new();
    let mut heap = BinaryHeap::new();
    let key = |c: f32| (c * 1000.0) as u64;
    best.insert((start, layer), 0.0);
    heap.push(Reverse((key(toward(start_dir)), (start, layer))));
    let mut nearest = ((start, layer), toward(start_dir));
    let mut expanded = 0;
    let mut reached = None;
    while let Some(Reverse((_, state))) = heap.pop() {
        let (a, la) = state;
        // In the goal's cell, or beside it: a station or a bank is never
        // stood in, and the last metre is walked straight at the point.
        if a == goal || ground.neighbours(a).contains(&goal) {
            reached = Some(state);
            break;
        }
        expanded += 1;
        if expanded > MAX_EXPANDED {
            break;
        }
        let here = best[&state];
        let left = toward(ground.centre(a));
        if left < nearest.1 {
            nearest = (state, left);
        }
        for b in ground.neighbours(a) {
            if !inside(b) && b != goal {
                continue;
            }
            for (lb, digs, extra) in moves(ground, a, la, b) {
                let wet = ground.block(b, lb) == Block::Water;
                let walk = metres(a, b) * if wet { WADE } else { 1.0 };
                let cost = here + walk + extra;
                let next = (b, lb);
                if best.get(&next).is_none_or(|&c| cost < c) {
                    best.insert(next, cost);
                    from.insert(next, (state, digs));
                    let h = toward(ground.centre(b));
                    heap.push(Reverse((key(cost + h), next)));
                }
            }
        }
    }
    let (end, arrives) = match reached {
        Some(state) => (state, true),
        None => (nearest.0, false),
    };
    let mut steps = Vec::new();
    let mut at = end;
    while at != (start, layer) {
        let (prev, digs) = from.get(&at)?.clone();
        steps.push(Step {
            cell: at.0,
            layer: at.1,
            digs,
        });
        at = prev;
    }
    steps.reverse();
    Some(Plan { steps, arrives })
}

/// The moves from `(a, la)` into `b`: the layer stood in, the blocks dug
/// first, and the cost of the digs and the climb.
fn moves(
    ground: &dyn Ground,
    a: usize,
    la: usize,
    b: usize,
) -> Vec<(usize, Vec<(usize, usize)>, f32)> {
    let mut out = Vec::new();
    // Footing is ground that can be dug: unloaded ground and stations are
    // never stood on.
    let standable = |l: usize| l > 0 && matches!(ground.block(b, l - 1), Block::Dig(_));
    let open = |cell: usize, l: usize| matches!(ground.block(cell, l), Block::Open | Block::Water);
    // Head-room over the feet: water above the head is a swim, not a wade.
    let clear_head = |cell: usize, l: usize| ground.block(cell, l) == Block::Open;
    // Walk: up one, or down as far as three, through open blocks.
    for lb in la.saturating_sub(3)..=la + 1 {
        if !standable(lb) || !open(b, lb) || !clear_head(b, lb + 1) {
            continue;
        }
        // A step up needs head-room over the walker's own cell too.
        if lb > la && !clear_head(a, la + 2) {
            continue;
        }
        // Dropping: the blocks between are open (it falls through them).
        if lb < la && !(lb + 2..=la + 1).all(|l| open(b, l)) {
            continue;
        }
        let extra = if lb > la { STEP_UP_M } else { 0.0 };
        out.push((lb, Vec::new(), extra));
    }
    // Tunnel, stair up and stair down: dig what is solid where the walker
    // goes. Never through a block that is never dug.
    let dig = |blocks: &[(usize, usize)]| -> Option<(Vec<(usize, usize)>, f32)> {
        let mut digs = Vec::new();
        let mut cost = 0.0;
        for &(cell, l) in blocks {
            match ground.block(cell, l) {
                Block::Open => {}
                // Under water it floods: never dug.
                Block::Dig(_) if ground.block(cell, l + 1) == Block::Water => return None,
                Block::Dig(c) => {
                    digs.push((cell, l));
                    cost += c;
                }
                Block::Water | Block::Never => return None,
            }
        }
        (!digs.is_empty()).then_some((digs, cost))
    };
    let candidates: [(usize, Vec<(usize, usize)>, f32); 3] = [
        (la, vec![(b, la + 1), (b, la)], 0.0),
        (
            la + 1,
            vec![(a, la + 2), (b, la + 2), (b, la + 1)],
            STEP_UP_M,
        ),
        (
            la.saturating_sub(1),
            vec![(b, la + 1), (b, la), (b, la.saturating_sub(1))],
            0.0,
        ),
    ];
    for (lb, blocks, climb) in candidates {
        if !standable(lb) || out.iter().any(|(l, _, _)| *l == lb) {
            continue;
        }
        if let Some((digs, cost)) = dig(&blocks) {
            out.push((lb, digs, cost + climb));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A strip of cells along x, a metre apart, each a column of blocks.
    struct Strip {
        columns: Vec<Vec<Block>>,
    }

    impl Ground for Strip {
        fn neighbours(&self, cell: usize) -> Vec<usize> {
            let mut n = Vec::new();
            if cell > 0 {
                n.push(cell - 1);
            }
            if cell + 1 < self.columns.len() {
                n.push(cell + 1);
            }
            n
        }
        fn centre(&self, cell: usize) -> Vec3 {
            // A metre a cell on a sphere of radius 1000.
            let a = cell as f32 / 1000.0;
            Vec3::new(a.sin(), a.cos(), 0.0)
        }
        fn block(&self, cell: usize, layer: usize) -> Block {
            self.columns[cell]
                .get(layer)
                .copied()
                .unwrap_or(Block::Open)
        }
    }

    const SOIL: Block = Block::Dig(2.0);
    const STONE: Block = Block::Dig(4.0);

    /// A column of `ground` blocks of `fill` under open air.
    fn col(ground: usize, fill: Block) -> Vec<Block> {
        vec![fill; ground]
    }

    fn walk(strip: &Strip, start: usize, layer: usize, goal: usize) -> Plan {
        plan(strip, start, layer, goal, 1000.0).expect("a plan")
    }

    #[test]
    fn flat_ground_is_walked_with_nothing_dug() {
        let strip = Strip {
            columns: (0..10).map(|_| col(5, SOIL)).collect(),
        };
        let p = walk(&strip, 0, 5, 9);
        assert!(p.arrives);
        // Beside the goal is there: eight steps to the cell next to it.
        assert_eq!(p.steps.len(), 8);
        assert!(p.steps.iter().all(|s| s.digs.is_empty() && s.layer == 5));
    }

    #[test]
    fn a_pit_two_deep_is_left_by_a_stair_cut_up_its_wall() {
        // The walker at the bottom of a pit (ground 3) in ground 5.
        let mut columns: Vec<Vec<Block>> = (0..8).map(|_| col(5, STONE)).collect();
        columns[0] = col(3, STONE);
        let strip = Strip { columns };
        let p = walk(&strip, 0, 3, 7);
        assert!(p.arrives);
        // Up one layer per step, each step up digging into the wall.
        assert_eq!(p.steps[0].layer, 4);
        assert!(!p.steps[0].digs.is_empty());
        assert!(p.steps.iter().all(|s| s.layer <= 5));
        assert_eq!(p.steps.last().unwrap().layer, 5);
    }

    #[test]
    fn a_wall_is_tunnelled_through_with_its_blocks_dug() {
        let mut columns: Vec<Vec<Block>> = (0..6).map(|_| col(5, SOIL)).collect();
        columns[3] = col(9, STONE);
        let strip = Strip { columns };
        let p = walk(&strip, 0, 5, 5);
        assert!(p.arrives);
        let through = &p.steps[2];
        assert_eq!((through.cell, through.layer), (3, 5));
        assert_eq!(through.digs, vec![(3, 6), (3, 5)]);
    }

    #[test]
    fn a_drop_of_three_is_walked_off_and_four_is_not() {
        let mut columns: Vec<Vec<Block>> = (0..4).map(|_| col(8, SOIL)).collect();
        columns[2] = col(5, SOIL);
        columns[3] = col(5, SOIL);
        let p = walk(&Strip { columns }, 0, 8, 3);
        assert!(p.steps.iter().all(|s| s.digs.is_empty()));
        assert_eq!(p.steps[1].layer, 5);
        let mut columns: Vec<Vec<Block>> = (0..4).map(|_| col(8, SOIL)).collect();
        columns[2] = col(4, SOIL);
        columns[3] = col(4, SOIL);
        let p = walk(&Strip { columns }, 0, 8, 3);
        // Down by stairs instead: blocks dug on the way.
        assert!(p.steps.iter().any(|s| !s.digs.is_empty()));
    }

    #[test]
    fn a_station_in_the_way_is_never_dug() {
        let mut columns: Vec<Vec<Block>> = (0..5).map(|_| col(5, SOIL)).collect();
        columns[2] = col(5, SOIL);
        columns[2].push(Block::Never);
        columns[2].push(Block::Never);
        columns[2].push(Block::Never);
        let p = walk(&Strip { columns }, 0, 5, 4);
        // A strip has no way round: it never digs the station; it climbs
        // over by digging nothing of it, or stops short.
        assert!(
            p.steps
                .iter()
                .all(|s| s.digs.iter().all(|&(c, l)| !(c == 2 && l >= 5)))
        );
    }

    #[test]
    fn deep_water_is_not_walked_into() {
        let mut columns: Vec<Vec<Block>> = (0..5).map(|_| col(5, SOIL)).collect();
        columns[2] = col(3, SOIL);
        columns[2].extend([Block::Water; 4]);
        let p = walk(&Strip { columns }, 0, 5, 4);
        assert!(!p.arrives);
        assert!(p.steps.iter().all(|s| s.cell < 2));
    }

    #[test]
    fn shallow_water_is_waded() {
        let mut columns: Vec<Vec<Block>> = (0..5).map(|_| col(5, SOIL)).collect();
        columns[2] = col(5, SOIL);
        columns[2].push(Block::Water);
        let p = walk(&Strip { columns }, 0, 5, 4);
        assert!(p.arrives);
    }
}
