//! The river walk (`river-cache` design 8): a recording instrument's path
//! and measures. The walker walks a whole river from its source to the sea
//! on one bank; this module holds what that walk steers by and what it
//! logs, read off the generator's drainage map (`rivers::RiverMap::course`)
//! and never off rendered state, so it is the same on every run.

use bevy::math::Vec3;

use crate::planet::{PLANET_RADIUS, terrain_config};

/// How far off the river's line the bank path runs past the river's own
/// half width, m: the valley's slope (`bank_m`, 7 m) and a little more, so
/// the walker is on the bank's top a few metres off the water.
const BANK_OFF_M: f32 = 9.0;
/// How far ahead along the path the walker steers for, m.
pub const STEER_AHEAD_M: f32 = 24.0;
/// How far ahead along the river the drawn water is measured, m.
pub const AHEAD_REACH_M: f32 = 600.0;
/// The measure's step along the river, m.
const AHEAD_STEP_M: f32 = 4.0;

/// A point `metres` along the ground from the unit direction `here` toward
/// the tangent `toward`.
fn along(here: Vec3, toward: Vec3, metres: f32) -> Vec3 {
    let a = metres / PLANET_RADIUS;
    (here * a.cos() + toward * a.sin()).normalize()
}

/// The unit tangent at `a` toward `b`.
fn tangent(a: Vec3, b: Vec3) -> Vec3 {
    (b - a * b.dot(a)).normalize_or_zero()
}

/// A whole river as a walker follows it.
#[derive(Clone, Debug)]
pub struct RiverPath {
    /// The river's line: its cells' centres, source first, the sea's cell
    /// last; unit directions.
    pub line: Vec<Vec3>,
    /// The bank the walker keeps to, one point per line point.
    pub bank: Vec<Vec3>,
    /// Metres along the line to each point.
    pub at_m: Vec<f32>,
    /// The river's level at each point, metres over the sea.
    pub level: Vec<i32>,
}

impl RiverPath {
    /// The river through the cell nearest `start`, with its bank on `side`
    /// (+1 the left looking downstream). None with no river within two cells.
    pub fn through(start: Vec3, side: f32) -> Option<Self> {
        let cfg = terrain_config();
        let settings = cfg.rivers.as_ref()?;
        let course = pbd_core::rivers::map(cfg, settings).course(start, 2);
        if course.len() < 2 {
            return None;
        }
        let line: Vec<Vec3> = course.iter().map(|s| s.centre).collect();
        let n = line.len();
        let bank = (0..n)
            .map(|i| {
                let up = line[i];
                // The way the river runs here: the mean of the reaches either
                // side, so a bend's bank point sits off the bend.
                let into = if i > 0 {
                    tangent(up, line[i - 1]) * -1.0
                } else {
                    Vec3::ZERO
                };
                let out = if i + 1 < n {
                    tangent(up, line[i + 1])
                } else {
                    Vec3::ZERO
                };
                let runs = (into + out).normalize_or(out);
                let left = up.cross(runs).normalize_or_zero() * side;
                along(up, left, course[i].half_width_m + BANK_OFF_M)
            })
            .collect();
        let mut at_m = vec![0.0];
        for w in line.windows(2) {
            at_m.push(at_m.last().unwrap() + w[0].angle_between(w[1]) * PLANET_RADIUS);
        }
        Some(Self {
            line,
            bank,
            at_m,
            level: course.iter().map(|s| s.level).collect(),
        })
    }

    /// The river's length, m: source to the sea's cell.
    pub fn length_m(&self) -> f32 {
        *self.at_m.last().unwrap_or(&0.0)
    }

    /// Metres along the river of the point of its line nearest `here`.
    pub fn progress_m(&self, here: Vec3) -> f32 {
        let here = here.normalize_or_zero();
        let mut best = (f32::INFINITY, 0.0);
        for (i, w) in self.line.windows(2).enumerate() {
            let (a, b) = (w[0], w[1]);
            let seg = b - a;
            let t = ((here - a).dot(seg) / seg.length_squared().max(1e-12)).clamp(0.0, 1.0);
            let p = (a + seg * t).normalize();
            let gap = p.angle_between(here);
            if gap < best.0 {
                best = (gap, self.at_m[i] + t * (self.at_m[i + 1] - self.at_m[i]));
            }
        }
        best.1
    }

    /// The point `metres` along `points` (the line or the bank), measured by
    /// the line's own distances.
    fn point_on(&self, points: &[Vec3], metres: f32) -> Vec3 {
        let m = metres.clamp(0.0, self.length_m());
        let i = self
            .at_m
            .windows(2)
            .position(|w| m <= w[1])
            .unwrap_or(self.at_m.len().saturating_sub(2));
        let span = (self.at_m[i + 1] - self.at_m[i]).max(1e-3);
        points[i]
            .lerp(points[i + 1], (m - self.at_m[i]) / span)
            .normalize()
    }

    /// The bank point `metres` along the river.
    pub fn bank_at(&self, metres: f32) -> Vec3 {
        self.point_on(&self.bank, metres)
    }

    /// The river's line `metres` along it.
    pub fn line_at(&self, metres: f32) -> Vec3 {
        self.point_on(&self.line, metres)
    }

    /// Where the river is drawn to ahead (the measure of design 8): from
    /// `from_m` along the river, the metres ahead to the first point whose
    /// water is in the view (`in_view(point, radius)` true at its surface)
    /// but whose sheet `listed` says is not listed, out to `AHEAD_REACH_M`.
    /// Points within `tier_m` of the tier's `anchor` are the tier's water,
    /// drawn either way. `in_view` takes a unit direction and a radius from
    /// the body's centre, m.
    pub fn drawn_ahead_m(
        &self,
        from_m: f32,
        anchor: Vec3,
        tier_m: f32,
        in_view: &dyn Fn(Vec3, f32) -> bool,
        listed: &dyn Fn(Vec3, f32) -> bool,
    ) -> f32 {
        let cfg = terrain_config();
        let mut ahead = 0.0;
        while ahead < AHEAD_REACH_M && from_m + ahead < self.length_m() {
            let d = self.line_at(from_m + ahead);
            let tier = d.angle_between(anchor.normalize_or_zero()) * PLANET_RADIUS < tier_m;
            // The water's surface here: the river cell's own reach's level
            // where the line passes over it.
            let surface = pbd_core::rivers::at(cfg, d)
                .map_or(0.0, |h| cfg.sea_level_m + h.level.max(0) as f32);
            if !tier && in_view(d, PLANET_RADIUS + surface) && !listed(d, surface) {
                return ahead;
            }
            ahead += AHEAD_STEP_M;
        }
        ahead.min(AHEAD_REACH_M)
    }
}

/// The tallest rise between two metre steps the walker takes with a jump,
/// m: one is auto-stepped, two are a jump (`river-cache` design 12).
pub const JUMPABLE_M: f32 = 2.0;
/// Under this much ground covered in the last second the walker is stuck,
/// m: on foot it sprints 14 m a second, swimming half that.
pub const STUCK_LAND_M: f32 = 3.0;
pub const STUCK_SWIM_M: f32 = 1.5;
/// What a walker stands on along the river at `d`, metres over the sea: the
/// ground, or a river's water over it.
pub fn walked_on(d: Vec3) -> f32 {
    let cfg = terrain_config();
    let ground = crate::planet::surface_height(d);
    // The water standing in this column, as the column is built with it
    // (`rivers::water`: sources to the water's top here), not the reach's
    // level: a reach's water is held lower wherever its banks are, and
    // reading the reach's level made a stepped channel look flat to the
    // router, which then walked into its steps.
    let sea = cfg.sea_level_m.round() as i32;
    let water = pbd_core::rivers::water(cfg, d)
        .last()
        .map_or(0.0, |&(top, _)| (top - sea) as f32);
    ground.max(water).max(0.0)
}

/// A cell's ground and what is walked on there (its water's top where water
/// stands over it), m over the sea: from the live column tier where the cell
/// is in it, so the flowing water the rule settled (a fall's lip, a sheet
/// between reaches) is seen as the walker meets it; from the generator
/// otherwise.
pub fn cell_heights(contact: Option<&crate::planet::PlanetContact>, d: Vec3) -> (f32, f32) {
    let d = d.normalize_or(Vec3::Y);
    if let Some(contact) = contact {
        let high = d * (PLANET_RADIUS + 400.0);
        if contact.column_water(high).is_some() {
            let floor = contact.stand(high).floor_radius - PLANET_RADIUS;
            let top = match contact.column_water(d * (PLANET_RADIUS + floor + 0.1)) {
                Some(Some(water)) => water.max(floor),
                _ => floor,
            };
            return (floor, top.max(0.0));
        }
    }
    (crate::planet::surface_height(d), walked_on(d))
}

/// The ground from `from` to `to` (unit directions) in metre steps: the
/// tallest rise between two steps, m, and the share of steps in water.
pub fn profile(from: Vec3, to: Vec3) -> (f32, f32) {
    let metres = from.angle_between(to) * PLANET_RADIUS;
    let steps = (metres.ceil() as usize).max(1);
    let mut last = walked_on(from);
    let (mut rise, mut wet) = (0.0f32, 0usize);
    for i in 1..=steps {
        let d = from.slerp(to, i as f32 / steps as f32);
        let h = walked_on(d);
        rise = rise.max(h - last);
        if crate::coast_walk::in_river(d) || crate::planet::surface_height(d) < 0.0 {
            wet += 1;
        }
        last = h;
    }
    (rise, wet as f32 / steps as f32)
}

/// Ground covered over the last second from a trail of `(seconds, unit
/// direction)`, newest last: the arc over the planet (height left out, so a
/// hop or a slide down a bank is not progress). `None` until the trail
/// spans a second.
pub fn moved_in_last_second(trail: &[(f32, Vec3)]) -> Option<f32> {
    let &(now, here) = trail.last()?;
    let &(_, then) = trail.iter().rev().find(|(t, _)| now - t >= 1.0)?;
    Some(here.angle_between(then) * PLANET_RADIUS)
}

/// A route over the finest hex cells, centre to centre (owner, 2026-11-18:
/// "bot should have specifc hex by hex naviation, I think this would avoid it
/// trying to take shortcuts that cause it to bump into sides"). Planned on
/// the cells' own column heights, so every step it takes is one the walker
/// can make: a rise of one layer walked, two jumped, more never.
#[derive(Clone, Debug)]
pub struct HexRoute {
    /// Cell centres in walking order, unit directions; the first is the
    /// walker's own cell.
    pub cells: Vec<Vec3>,
    /// Each cell's ground (or water), m over the sea.
    pub heights: Vec<f32>,
    /// The cell being walked to.
    pub next: usize,
}

/// How deep water must be over a cell's bed for a swimmer at the surface to
/// float over it, m: a swimmer is never grounded, so it steps up nothing,
/// and its feet hang about this far under the surface.
pub const FLOAT_OVER_M: f32 = 2.0;
/// How far over the water's surface a swimmer climbs out onto a bank, m:
/// well inside the walker's own reach (`water_climb_reach_m`, 4.5 m), since
/// it has to rise to the surface by the bank first.
pub const CLIMB_OUT_M: f32 = 1.5;

/// Whether the walker can go from cell `a` to its neighbour `b`, each as
/// (ground, what is walked on, wet), the way the walker's physics allows
/// (`walking::sweep`): on foot a rise of one layer is stepped and up to
/// [`JUMPABLE_M`] jumped; a swimmer, never grounded, steps up nothing, so it
/// goes on only where the water is deep enough to float over or the bed does
/// not rise; out of the water, onto ground no more than [`CLIMB_OUT_M`] over
/// the surface.
pub fn step_allowed(a: (f32, f32, bool), b: (f32, f32, bool)) -> bool {
    let ((ground_a, top_a, wet_a), (ground_b, top_b, wet_b)) = (a, b);
    match (wet_a, wet_b) {
        (false, false) | (false, true) => ground_b - ground_a <= JUMPABLE_M,
        (true, true) => top_b - ground_b >= FLOAT_OVER_M || ground_b <= ground_a + 0.05,
        (true, false) => ground_b - top_a <= CLIMB_OUT_M,
    }
}

/// The deepest water a walker keeping to dry ground wades through, m.
pub const WADE_M: f32 = 0.8;

/// A cell's key: its centre's bits, the same whatever the anchor.
pub fn cell_key(d: Vec3) -> [u32; 3] {
    d.to_array().map(f32::to_bits)
}

impl HexRoute {
    /// The cheapest walk over the finest cells from the walker's cell to the
    /// cell nearest `goal`, through the cells within the ellipse round the
    /// two; `None` if no walkable route joins them. A step costs its length,
    /// more on land than in the river (the run keeps to the water), and a two-layer jump four
    /// cells more (the trial walk stuck at them: taken only where needed);
    /// a rise past [`JUMPABLE_M`] (from water, past one layer and a half) and
    /// a `blocked` cell are never taken.
    pub fn plan(
        here: Vec3,
        goal: Vec3,
        blocked: &[[u32; 3]],
        contact: Option<&crate::planet::PlanetContact>,
    ) -> Option<Self> {
        Self::plan_where(here, goal, contact, false, &|d: Vec3| {
            blocked.contains(&cell_key(d))
        })
    }

    /// [`Self::plan`] for a walker that keeps to dry ground (`dry`: water
    /// costs six times land, and water too deep to wade is never entered),
    /// round the cells `avoid` says (`mission-playthrough`: the showcase
    /// bot's trees, pits and stations).
    pub fn plan_where(
        here: Vec3,
        goal: Vec3,
        contact: Option<&crate::planet::PlanetContact>,
        dry: bool,
        avoid: &dyn Fn(Vec3) -> bool,
    ) -> Option<Self> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;
        let here = here.normalize_or(Vec3::Y);
        let goal = goal.normalize_or(Vec3::Y);
        // Never more than 60 m at a time: a route is a few hundred cells, a
        // couple of milliseconds, and planned again as it runs out.
        let far = here.angle_between(goal) * PLANET_RADIUS;
        let goal = if far > 60.0 {
            here.slerp(goal, 60.0 / far)
        } else {
            goal
        };
        let span = here.angle_between(goal) * PLANET_RADIUS;
        let mid = here.slerp(goal, 0.5);
        let radius = (span * 0.5 + 18.0) / PLANET_RADIUS;
        let mut lattice = crate::planet::lattice::Lattice::default();
        let cells = lattice.cells_in_band(crate::planet::FINEST_LEVEL, mid, 0.0, radius);
        if cells.is_empty() {
            return None;
        }
        let dirs: Vec<Vec3> = cells.iter().map(|c| c.cell.direction).collect();
        let (ground, heights): (Vec<f32>, Vec<f32>) =
            dirs.iter().map(|&d| cell_heights(contact, d)).unzip();
        // Wet where water stands over the ground: the column's own water.
        let wet: Vec<bool> = (0..dirs.len())
            .map(|i| heights[i] > ground[i] + 0.01)
            .collect();
        let nearest = |p: Vec3| {
            (0..dirs.len())
                .max_by(|&a, &b| dirs[a].dot(p).total_cmp(&dirs[b].dot(p)))
                .expect("cells")
        };
        let (start, end) = (nearest(here), nearest(goal));
        // Whether a cell is never taken, asked only of the cells the search
        // reaches (`bot-path-planner` 13: asking every cell in the disc first
        // was most of a 50 to 100 ms route). The same answers, so the same
        // route.
        let mut off: Vec<Option<bool>> = vec![None; dirs.len()];
        let mut is_off = |i: usize| {
            *off[i].get_or_insert_with(|| {
                i != start
                    && i != end
                    && ((dry && wet[i] && heights[i] - ground[i] > WADE_M) || avoid(dirs[i]))
            })
        };
        let metres = |a: usize, b: usize| dirs[a].angle_between(dirs[b]) * PLANET_RADIUS;
        let mut cost = vec![f32::INFINITY; dirs.len()];
        let mut from = vec![usize::MAX; dirs.len()];
        let mut heap = BinaryHeap::new();
        let key = |c: f32| (c * 1000.0) as u64;
        cost[start] = 0.0;
        heap.push(Reverse((key(metres(start, end)), start)));
        while let Some(Reverse((_, a))) = heap.pop() {
            if a == end {
                break;
            }
            for &b in &cells[a].cell.neighbors {
                if b == usize::MAX || b >= dirs.len() || is_off(b) {
                    continue;
                }
                let rise = heights[b] - heights[a];
                if !step_allowed(
                    (ground[a], heights[a], wet[a]),
                    (ground[b], heights[b], wet[b]),
                ) {
                    continue;
                }
                let step = metres(a, b);
                let (wet_cost, dry_cost) = if dry { (6.0, 1.0) } else { (1.0, 1.6) };
                let c = cost[a]
                    + step * if wet[b] { wet_cost } else { dry_cost }
                    + if rise > 1.1 { 12.0 } else { 0.0 }
                    // Out of the river only where it has to: a climb out is
                    // where a swimmer is slowest to get going.
                    + if wet[a] && !wet[b] { 8.0 } else { 0.0 };
                if c < cost[b] {
                    cost[b] = c;
                    from[b] = a;
                    heap.push(Reverse((key(c + metres(b, end)), b)));
                }
            }
        }
        if !cost[end].is_finite() {
            return None;
        }
        let mut order = vec![end];
        while let Some(&last) = order.last() {
            if last == start {
                break;
            }
            order.push(from[last]);
        }
        order.reverse();
        Some(Self {
            cells: order.iter().map(|&i| dirs[i]).collect(),
            heights: order.iter().map(|&i| heights[i]).collect(),
            next: 1.min(order.len() - 1),
        })
    }

    /// The cell centre to walk to from `here`: on past a cell once within
    /// 1.2 m of its centre, or once the cell after it is nearer.
    pub fn aim(&mut self, here: Vec3) -> Vec3 {
        let here = here.normalize_or(Vec3::Y);
        let gap = |d: Vec3| d.angle_between(here) * PLANET_RADIUS;
        while self.next + 1 < self.cells.len()
            && (gap(self.cells[self.next]) < 1.2
                || gap(self.cells[self.next + 1]) < gap(self.cells[self.next]))
        {
            self.next += 1;
        }
        self.cells[self.next]
    }

    /// The rise to the cell being walked to from the one before it, m.
    pub fn rise_ahead(&self) -> f32 {
        if self.next == 0 {
            return 0.0;
        }
        self.heights[self.next] - self.heights[self.next - 1]
    }

    /// The route's length still to walk from the cell being walked to, m.
    pub fn left_m(&self) -> f32 {
        self.cells[self.next.min(self.cells.len() - 1)..]
            .windows(2)
            .map(|w| w[0].angle_between(w[1]) * PLANET_RADIUS)
            .sum()
    }

    /// How far `here` is off the route's current leg, m.
    pub fn off_m(&self, here: Vec3) -> f32 {
        let here = here.normalize_or(Vec3::Y);
        let a = self.cells[self.next.saturating_sub(1)];
        let b = self.cells[self.next];
        let seg = b - a;
        let t = ((here - a).dot(seg) / seg.length_squared().max(1e-12)).clamp(0.0, 1.0);
        (a + seg * t).normalize().angle_between(here) * PLANET_RADIUS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::geo;

    fn at(lat: f32, lon: f32) -> Vec3 {
        geo::direction(geo::LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        })
    }

    /// The owner's river (`--at 20.885 -6.393`): a whole river, source to
    /// sea, with a bank that keeps off its line by more than the river's
    /// width and stays near it; progress runs from nought to its length.
    #[test]
    fn the_owner_s_river_is_walkable_on_its_bank() {
        if terrain_config().rivers.is_none() {
            return;
        }
        let path = RiverPath::through(at(20.885, -6.393), 1.0).expect("a river");
        let length = path.length_m();
        assert!((1_400.0..1_800.0).contains(&length), "{length} m");
        for (line, bank) in path.line.iter().zip(&path.bank) {
            let off = line.angle_between(*bank) * PLANET_RADIUS;
            assert!((9.0..25.0).contains(&off), "the bank {off} m off the line");
        }
        assert!(path.progress_m(path.line[0]) < 1.0);
        let end = path.progress_m(*path.line.last().unwrap());
        assert!((end - length).abs() < 1.0, "{end} of {length}");
        let mid = path.progress_m(path.bank_at(length * 0.5));
        assert!((mid - length * 0.5).abs() < 30.0, "{mid}");
    }

    /// Stuck is ground covered over a second, not height: a walker hopping
    /// on the spot is stuck, one sprinting is not, and a trail shorter than a
    /// second says nothing yet.
    #[test]
    fn stuck_is_ground_covered_in_a_second() {
        let a = at(20.0, -6.0);
        let east = Vec3::Y.cross(a).normalize();
        let step = |m: f32| along(a, east, m);
        let short = [(0.0, a), (0.5, step(0.2))];
        assert_eq!(moved_in_last_second(&short), None);
        let hopping: Vec<(f32, Vec3)> = (0..=12)
            .map(|i| (i as f32 * 0.1, step(0.1 * (i % 2) as f32)))
            .collect();
        assert!(moved_in_last_second(&hopping).unwrap() < STUCK_LAND_M);
        let sprint: Vec<(f32, Vec3)> = (0..=12)
            .map(|i| (i as f32 * 0.1, step(1.4 * i as f32)))
            .collect();
        assert!(moved_in_last_second(&sprint).unwrap() > 13.0);
    }

    /// A hex route down the owner's river from bank point to bank point 40 m
    /// on: found all the way, every step a neighbour's (no shortcut over a
    /// cell), and no step rising past a jump.
    #[test]
    fn a_hex_route_runs_down_the_owner_s_river() {
        if terrain_config().rivers.is_none() {
            return;
        }
        let path = RiverPath::through(at(20.885, -6.393), -1.0).expect("a river");
        let (mut found, mut n) = (0, 0);
        let mut m = 0.0;
        while m < path.length_m() - 45.0 {
            let here = path.bank_at(m);
            let route = HexRoute::plan(here, path.bank_at(m + 40.0), &[], None)
                .or_else(|| HexRoute::plan(here, path.line_at(m + 40.0), &[], None));
            n += 1;
            if let Some(route) = route {
                found += 1;
                for (w, h) in route.cells.windows(2).zip(route.heights.windows(2)) {
                    let step = w[0].angle_between(w[1]) * PLANET_RADIUS;
                    // A neighbour is 2.6 to 3.8 m off (the dual's spread);
                    // two cells across is 4.9 m or more.
                    assert!(
                        step < 4.4,
                        "{m} m: a step of {step} m is not to a neighbour"
                    );
                    assert!(
                        h[1] - h[0] <= JUMPABLE_M,
                        "{m} m: a rise of {}",
                        h[1] - h[0]
                    );
                }
            }
            m += 40.0;
        }
        assert!(found * 10 >= n * 9, "routes found {found} of {n}");
    }

    /// The router's step rule is the walker's: a swimmer passes over deep
    /// water or a bed that does not rise, never up a shallow bed's step; it
    /// climbs out near the surface; on foot it steps one layer and jumps two.
    #[test]
    fn the_step_rule_is_the_walker_s() {
        // (ground, walked on, wet)
        let deep = |bed: f32| (bed, bed + 2.0, true);
        let shallow = |bed: f32| (bed, bed + 0.5, true);
        let dry = |g: f32| (g, g, false);
        assert!(
            step_allowed(deep(60.0), deep(61.0)),
            "floats over deep water"
        );
        assert!(
            step_allowed(deep(60.0), shallow(60.0)),
            "a bed that does not rise"
        );
        assert!(
            !step_allowed(deep(60.0), shallow(61.0)),
            "a shallow bed's step stops it"
        );
        assert!(
            step_allowed(deep(60.0), dry(63.0)),
            "climbs out at the surface"
        );
        assert!(
            !step_allowed(deep(60.0), dry(64.0)),
            "not a bank over its reach"
        );
        assert!(step_allowed(dry(60.0), dry(62.0)) && !step_allowed(dry(60.0), dry(63.0)));
    }

    /// The measure: with every sheet listed the water is drawn to the
    /// reach; with none, to the first point in view past the tier.
    #[test]
    fn the_drawn_water_ahead_stops_at_the_first_culled_point() {
        if terrain_config().rivers.is_none() {
            return;
        }
        let path = RiverPath::through(at(20.885, -6.393), 1.0).expect("a river");
        let all = |_: Vec3, _: f32| true;
        let none = |_: Vec3, _: f32| false;
        let start = path.line_at(0.0);
        assert_eq!(
            path.drawn_ahead_m(0.0, start, 0.0, &all, &all),
            AHEAD_REACH_M
        );
        assert_eq!(path.drawn_ahead_m(0.0, start, 0.0, &all, &none), 0.0);
        let tier = path.drawn_ahead_m(0.0, start, 90.0, &all, &none);
        assert!((88.0..AHEAD_REACH_M).contains(&tier), "{tier}");
    }
}

#[cfg(test)]
mod uphill_probe {
    use super::*;
    use crate::planet::terrain_config;

    /// INSTRUMENT (ignored, `river-downhill`): the water's top every 2 m down
    /// the owner's river's line, and every place it rises going downstream:
    /// how much, and whether the reach's own level rose there (never, by the
    /// map's construction) or the bank clamp let it back up.
    #[test]
    #[ignore]
    fn where_the_river_runs_uphill() {
        let cfg = terrain_config();
        let src = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 19.25f32.to_radians(),
            lon: (-6.75f32).to_radians(),
        });
        let path = RiverPath::through(src, 1.0).expect("river");
        let sea = cfg.sea_level_m.round() as i32;
        let mut last: Option<(f32, i32, i32)> = None;
        let (mut rises, mut total, mut worst, mut points) = (0, 0, 0, 0);
        let mut m = 0.0;
        while m < path.length_m() {
            let d = path.line_at(m);
            let top = pbd_core::rivers::water(cfg, d).last().map(|w| w.0 - sea);
            let level = pbd_core::rivers::at(cfg, d).map(|h| h.level);
            if let (Some(top), Some(level)) = (top, level) {
                points += 1;
                if let Some((at, t0, l0)) = last
                    && top > t0
                {
                    rises += 1;
                    total += top - t0;
                    worst = worst.max(top - t0);
                    if rises <= 25 {
                        eprintln!(
                            "{at:5.0} -> {m:5.0} m: top {t0} -> {top} (+{}), reach level {l0} -> {level}",
                            top - t0
                        );
                    }
                }
                last = Some((m, top, level));
            }
            m += 2.0;
        }
        eprintln!(
            "{points} points with water; {rises} rises, {total} m in all, the worst {worst} m"
        );
    }
}
