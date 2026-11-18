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
