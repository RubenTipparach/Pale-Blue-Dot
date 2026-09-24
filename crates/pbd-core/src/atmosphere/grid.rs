//! The cells the atmosphere and the ocean live on, and the finite-volume
//! operators both fluids use.
//!
//! The same Goldberg dual the terrain is built on (`crate::topology`), at a
//! coarse level. Every operator is written for any per-cell field, so the air
//! and the sea share one gradient, one divergence and one transport rather than
//! two copies that could drift apart.

use crate::topology::{DualCell, dual_sphere};
use glam::Vec3;

/// The most sides a cell has. Pentagons fill the sixth slot with themselves
/// and a zero-length edge, so every loop can run to six.
pub const SIDES: usize = 6;

/// The cells, their neighbours, and the geometry the operators need.
#[derive(Clone, Debug)]
pub struct Grid {
    pub level: u32,
    /// Planet radius, metres.
    pub radius: f32,
    /// Unit direction of each cell's centre.
    pub centre: Vec<Vec3>,
    /// Neighbour across each side, in ring order; a pentagon's sixth is itself.
    pub neighbour: Vec<[u32; SIDES]>,
    /// How many sides a cell really has: 5 or 6.
    pub sides: Vec<u8>,
    /// Length of each side, metres. Zero for a pentagon's sixth.
    pub edge: Vec<[f32; SIDES]>,
    /// Unit tangent at the centre pointing toward each neighbour: the outward
    /// normal of that side.
    pub normal: Vec<[Vec3; SIDES]>,
    /// Distance to each neighbour's centre, metres.
    pub span: Vec<[f32; SIDES]>,
    /// Cell area, square metres.
    pub area: Vec<f32>,
    /// For each side, which side of the neighbour faces back: so an edge's
    /// flux is worked out once and its neighbour takes exactly the negative.
    pub opposite: Vec<[u8; SIDES]>,
    /// Least-squares gradient weights: the gradient of a field at a cell is
    /// the sum over sides of (neighbour minus cell) times this, per metre.
    pub slope: Vec<[Vec3; SIDES]>,
    /// A coarse direction -> cell table that starts a point search close to its
    /// answer: `LOOKUP` x `LOOKUP` per cube face.
    lookup: Vec<u32>,
}

const LOOKUP: usize = 32;

impl Grid {
    /// The cells at this subdivision level on a sphere of this radius.
    pub fn new(level: u32, radius: f32) -> Self {
        let cells = dual_sphere(level);
        let n = cells.len();
        let mut grid = Grid {
            level,
            radius,
            centre: Vec::with_capacity(n),
            neighbour: Vec::with_capacity(n),
            sides: Vec::with_capacity(n),
            edge: Vec::with_capacity(n),
            normal: Vec::with_capacity(n),
            span: Vec::with_capacity(n),
            area: Vec::with_capacity(n),
            slope: Vec::with_capacity(n),
            opposite: Vec::with_capacity(n),
            lookup: Vec::new(),
        };
        for (index, cell) in cells.iter().enumerate() {
            grid.push_cell(index, cell, &cells);
        }
        for i in 0..n {
            let mut back = [0u8; SIDES];
            for (side, slot) in back.iter_mut().enumerate().take(grid.sides[i] as usize) {
                let k = grid.neighbour[i][side] as usize;
                *slot = (0..grid.sides[k] as usize)
                    .find(|&j| grid.neighbour[k][j] as usize == i)
                    .unwrap_or(0) as u8;
            }
            grid.opposite.push(back);
        }
        grid.lookup = grid.build_lookup();
        grid
    }

    fn push_cell(&mut self, index: usize, cell: &DualCell, cells: &[DualCell]) {
        let c = cell.direction;
        let sides = cell.neighbors.len();
        let mut neighbour = [index as u32; SIDES];
        let mut edge = [0.0; SIDES];
        let mut normal = [Vec3::ZERO; SIDES];
        let mut span = [1.0; SIDES];
        let mut area = 0.0;
        for side in 0..sides {
            let a = cell.corners[side];
            let b = cell.corners[(side + 1) % sides];
            // The corner ring and the neighbour list run in the same order:
            // the side from corner `side` to `side + 1` faces `neighbors[side]`.
            let k = cell.neighbors[side];
            neighbour[side] = k as u32;
            edge[side] = a.angle_between(b) * self.radius;
            let toward = cells[k].direction;
            // The side's own outward normal, at its midpoint: the corners are
            // triangle centroids, so a side is not quite square to the line
            // between centres, and Green-Gauss wants the side's normal.
            let middle = (a + b).normalize();
            let mut n = (b - a).cross(middle).normalize_or_zero();
            if n.dot(toward - c) < 0.0 {
                n = -n;
            }
            normal[side] = (n - c * c.dot(n)).normalize_or_zero();
            span[side] = c.angle_between(toward) * self.radius;
            // The fan triangle centre-a-b, on the tangent plane: area enough
            // for a cell a few hundred metres across on a 4.8 km sphere.
            area += 0.5 * (a - c).cross(b - c).length() * self.radius * self.radius;
        }
        self.slope
            .push(least_squares(c, &cell.neighbors, cells, self.radius));
        self.centre.push(c);
        self.neighbour.push(neighbour);
        self.sides.push(sides as u8);
        self.edge.push(edge);
        self.normal.push(normal);
        self.span.push(span);
        self.area.push(area);
    }

    pub fn len(&self) -> usize {
        self.centre.len()
    }

    pub fn is_empty(&self) -> bool {
        self.centre.is_empty()
    }

    /// Mean distance between neighbouring centres, metres.
    pub fn spacing(&self) -> f32 {
        let (mut sum, mut count) = (0.0f64, 0u32);
        for (spans, &sides) in self.span.iter().zip(&self.sides) {
            for span in &spans[..sides as usize] {
                sum += *span as f64;
                count += 1;
            }
        }
        (sum / count.max(1) as f64) as f32
    }

    /// Gradient of a scalar at a cell, per metre, in the tangent plane: the
    /// least-squares plane through the cell and its neighbours, which is exact
    /// for a field that varies linearly. `blocked` says which neighbours are
    /// walls; a wall reads as the cell's own value, so a coast exerts no
    /// pressure.
    pub fn gradient(&self, field: &[f32], cell: usize, blocked: impl Fn(usize) -> bool) -> Vec3 {
        let here = field[cell];
        let mut sum = Vec3::ZERO;
        for side in 0..self.sides[cell] as usize {
            let k = self.neighbour[cell][side] as usize;
            if !blocked(k) {
                sum += self.slope[cell][side] * (field[k] - here);
            }
        }
        sum
    }

    /// Divergence of a tangent vector field at a cell, per second (flux out
    /// through each side over the area). No flux crosses a wall.
    pub fn divergence(&self, field: &[Vec3], cell: usize, blocked: impl Fn(usize) -> bool) -> f32 {
        let mut flux = 0.0;
        for side in 0..self.sides[cell] as usize {
            let k = self.neighbour[cell][side] as usize;
            if blocked(k) {
                continue;
            }
            let face = (field[cell] + field[k]) * 0.5;
            flux += self.edge[cell][side] * face.dot(self.normal[cell][side]);
        }
        flux / self.area[cell]
    }

    /// The volume flux out through each side of each cell for a velocity
    /// field, m^2/s: side length times the face velocity along the side's
    /// normal. Zero through a wall and through a pentagon's missing sixth.
    /// Worked out once per velocity and shared by every field it carries.
    pub fn fluxes(&self, velocity: &[Vec3], blocked: impl Fn(usize) -> bool) -> Vec<[f32; SIDES]> {
        let mut out = vec![[0.0f32; SIDES]; self.len()];
        for i in 0..self.len() {
            if blocked(i) {
                continue;
            }
            for side in 0..self.sides[i] as usize {
                let k = self.neighbour[i][side] as usize;
                // Each edge once, from its lower-numbered cell; the other
                // takes the exact negative, so what leaves one cell is what
                // arrives in the next, to the bit.
                if k <= i || blocked(k) {
                    continue;
                }
                let face = (velocity[i] + velocity[k]) * 0.5;
                let flux = self.edge[i][side] * face.dot(self.normal[i][side]);
                out[i][side] = flux;
                out[k][self.opposite[i][side] as usize] = -flux;
            }
        }
        out
    }

    /// Carry a field one step through those fluxes, first-order upwind:
    /// whatever crosses a side leaves with the value of the cell it came from.
    ///
    /// `conserve` picks the form. Conserved, a field is an amount per square
    /// metre (vapour, cloud) and the area-weighted total is kept to rounding:
    /// what leaves one cell arrives in the next. Otherwise it is a property of
    /// the air (temperature, charge), and converging air does not pile it up.
    /// Stable while no wind crosses a cell in a step.
    pub fn upwind(
        &self,
        field: &[f32],
        fluxes: &[[f32; SIDES]],
        dt: f32,
        conserve: bool,
    ) -> Vec<f32> {
        (0..self.len())
            .map(|i| {
                let here = field[i];
                let mut change = 0.0;
                for (side, &flux) in fluxes[i].iter().enumerate() {
                    if flux == 0.0 {
                        continue;
                    }
                    let carried = if flux > 0.0 {
                        here
                    } else {
                        field[self.neighbour[i][side] as usize]
                    };
                    change -= flux * if conserve { carried } else { carried - here };
                }
                here + change * dt / self.area[i]
            })
            .collect()
    }

    /// The same for a vector field, component by component, left on each
    /// cell's tangent plane.
    pub fn upwind_vec(&self, field: &[Vec3], fluxes: &[[f32; SIDES]], dt: f32) -> Vec<Vec3> {
        (0..self.len())
            .map(|i| {
                let here = field[i];
                let mut change = Vec3::ZERO;
                for (side, &flux) in fluxes[i].iter().enumerate() {
                    if flux < 0.0 {
                        change -= (field[self.neighbour[i][side] as usize] - here) * flux;
                    }
                }
                let v = here + change * (dt / self.area[i]);
                let c = self.centre[i];
                v - c * c.dot(v)
            })
            .collect()
    }

    /// Mean of a scalar over a cell's open neighbours minus the cell's own
    /// value: a Laplacian in units of the field, which is what smoothing wants.
    pub fn neighbour_excess(
        &self,
        field: &[f32],
        cell: usize,
        blocked: impl Fn(usize) -> bool,
    ) -> f32 {
        let (mut sum, mut count) = (0.0, 0.0);
        for side in 0..self.sides[cell] as usize {
            let k = self.neighbour[cell][side] as usize;
            if !blocked(k) {
                sum += field[k];
                count += 1.0;
            }
        }
        if count > 0.0 {
            sum / count - field[cell]
        } else {
            0.0
        }
    }

    /// The cell whose centre is nearest a direction, found by walking downhill
    /// in distance from `start`.
    pub fn nearest_from(&self, direction: Vec3, start: usize) -> usize {
        let mut best = start;
        let mut best_dot = self.centre[best].dot(direction);
        loop {
            let mut moved = false;
            for side in 0..self.sides[best] as usize {
                let k = self.neighbour[best][side] as usize;
                let d = self.centre[k].dot(direction);
                if d > best_dot {
                    best = k;
                    best_dot = d;
                    moved = true;
                }
            }
            if !moved {
                return best;
            }
        }
    }

    /// The cell whose centre is nearest a direction, from anywhere.
    pub fn nearest(&self, direction: Vec3) -> usize {
        let start = self.lookup[lookup_index(direction)] as usize;
        self.nearest_from(direction, start)
    }

    /// The three cells whose centres make the triangle a direction falls in,
    /// and its barycentric weights, starting the search at `start`. The same
    /// interpolant serves the transport, the sampling for the renderer and the
    /// overlays, so none of them can disagree about what lies between cells.
    pub fn locate_from(&self, direction: Vec3, start: usize) -> ([u32; 3], [f32; 3]) {
        let direction = direction.normalize_or(Vec3::Y);
        let c = self.nearest_from(direction, start);
        let a = self.centre[c];
        let sides = self.sides[c] as usize;
        let mut fallback = None;
        for side in 0..sides {
            let j = self.neighbour[c][side] as usize;
            let k = self.neighbour[c][(side + 1) % sides] as usize;
            let (b, e) = (self.centre[j], self.centre[k]);
            // Gnomonic barycentrics: each weight is the signed volume of the
            // point with the opposite edge.
            let wa = direction.dot(b.cross(e));
            let wb = a.dot(direction.cross(e));
            let we = a.dot(b.cross(direction));
            let total = wa + wb + we;
            if total.abs() < 1e-12 {
                continue;
            }
            let w = [wa / total, wb / total, we / total];
            if w.iter().all(|&x| x >= -1e-4) {
                return ([c as u32, j as u32, k as u32], clamp_weights(w));
            }
            let worst = w.iter().cloned().fold(f32::INFINITY, f32::min);
            if fallback.is_none_or(|(best, _, _)| worst > best) {
                fallback = Some((worst, [c as u32, j as u32, k as u32], w));
            }
        }
        // Only a direction on a seam to the float can miss every fan
        // triangle; the least-outside one is then right to rounding.
        match fallback {
            Some((_, cells, w)) => (cells, clamp_weights(w)),
            None => ([c as u32; 3], [1.0, 0.0, 0.0]),
        }
    }

    pub fn locate(&self, direction: Vec3) -> ([u32; 3], [f32; 3]) {
        let start = self.lookup[lookup_index(direction)] as usize;
        self.locate_from(direction, start)
    }

    /// A scalar field at any direction, interpolated.
    pub fn sample(&self, field: &[f32], direction: Vec3) -> f32 {
        let (cells, w) = self.locate(direction);
        weighted(field, cells, w)
    }

    fn build_lookup(&self) -> Vec<u32> {
        let mut table = vec![0u32; 6 * LOOKUP * LOOKUP];
        let mut start = 0usize;
        for (index, slot) in table.iter_mut().enumerate() {
            let direction = lookup_direction(index);
            start = self.nearest_from(direction, start);
            *slot = start as u32;
        }
        table
    }
}

/// The least-squares gradient weights for one cell: with the offsets to the
/// neighbours on the tangent plane, `M = sum d d^T`, and the weight for a
/// neighbour is `M^-1 d`, carried back into 3D.
fn least_squares(c: Vec3, neighbours: &[usize], cells: &[DualCell], radius: f32) -> [Vec3; SIDES] {
    let e1 = c.any_orthonormal_vector();
    let e2 = c.cross(e1);
    let offsets: Vec<(f32, f32)> = neighbours
        .iter()
        .map(|&k| {
            let t = cells[k].direction;
            let along = (t - c * c.dot(t)).normalize_or_zero() * (c.angle_between(t) * radius);
            (along.dot(e1), along.dot(e2))
        })
        .collect();
    let (mut xx, mut xy, mut yy) = (0.0f64, 0.0f64, 0.0f64);
    for &(x, y) in &offsets {
        xx += (x * x) as f64;
        xy += (x * y) as f64;
        yy += (y * y) as f64;
    }
    let det = xx * yy - xy * xy;
    let mut out = [Vec3::ZERO; SIDES];
    if det.abs() < 1e-12 {
        return out;
    }
    for (slot, &(x, y)) in out.iter_mut().zip(&offsets) {
        let gx = (yy * x as f64 - xy * y as f64) / det;
        let gy = (xx * y as f64 - xy * x as f64) / det;
        *slot = e1 * gx as f32 + e2 * gy as f32;
    }
    out
}

/// Interpolate a scalar at three cells.
pub fn weighted(field: &[f32], cells: [u32; 3], w: [f32; 3]) -> f32 {
    field[cells[0] as usize] * w[0]
        + field[cells[1] as usize] * w[1]
        + field[cells[2] as usize] * w[2]
}

/// Interpolate a vector at three cells.
pub fn weighted_vec(field: &[Vec3], cells: [u32; 3], w: [f32; 3]) -> Vec3 {
    field[cells[0] as usize] * w[0]
        + field[cells[1] as usize] * w[1]
        + field[cells[2] as usize] * w[2]
}

fn clamp_weights(w: [f32; 3]) -> [f32; 3] {
    let w = [w[0].max(0.0), w[1].max(0.0), w[2].max(0.0)];
    let total = (w[0] + w[1] + w[2]).max(1e-12);
    [w[0] / total, w[1] / total, w[2] / total]
}

/// Which cube face a direction is on and where on it, as a table index.
fn lookup_index(direction: Vec3) -> usize {
    let a = direction.abs();
    let (face, u, v) = if a.x >= a.y && a.x >= a.z {
        (
            if direction.x > 0.0 { 0 } else { 1 },
            direction.y / a.x,
            direction.z / a.x,
        )
    } else if a.y >= a.z {
        (
            if direction.y > 0.0 { 2 } else { 3 },
            direction.x / a.y,
            direction.z / a.y,
        )
    } else {
        (
            if direction.z > 0.0 { 4 } else { 5 },
            direction.x / a.z,
            direction.y / a.z,
        )
    };
    let cell = |t: f32| (((t + 1.0) * 0.5 * LOOKUP as f32) as usize).min(LOOKUP - 1);
    if !(u.is_finite() && v.is_finite()) {
        return 0;
    }
    face * LOOKUP * LOOKUP + cell(v) * LOOKUP + cell(u)
}

fn lookup_direction(index: usize) -> Vec3 {
    let face = index / (LOOKUP * LOOKUP);
    let row = (index / LOOKUP) % LOOKUP;
    let column = index % LOOKUP;
    let u = (column as f32 + 0.5) / LOOKUP as f32 * 2.0 - 1.0;
    let v = (row as f32 + 0.5) / LOOKUP as f32 * 2.0 - 1.0;
    let d = match face {
        0 => Vec3::new(1.0, u, v),
        1 => Vec3::new(-1.0, u, v),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, v),
        4 => Vec3::new(u, v, 1.0),
        _ => Vec3::new(u, v, -1.0),
    };
    d.normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Grid {
        Grid::new(4, 4_800.0)
    }

    #[test]
    fn the_cells_cover_the_sphere() {
        let g = grid();
        let total: f64 = g.area.iter().map(|&a| a as f64).sum();
        let sphere = 4.0 * std::f64::consts::PI * 4_800.0f64.powi(2);
        assert!(
            (total / sphere - 1.0).abs() < 0.01,
            "areas sum to {total} of {sphere}"
        );
        assert_eq!(g.sides.iter().filter(|&&s| s == 5).count(), 12);
    }

    /// The gradient of a field that rises steadily along one axis is that
    /// slope, projected on each cell's tangent plane.
    #[test]
    fn the_gradient_of_a_linear_field_is_its_slope() {
        let g = grid();
        let slope = Vec3::new(0.3, -0.7, 0.2);
        let field: Vec<f32> = g.centre.iter().map(|c| c.dot(slope) * g.radius).collect();
        let mut worst = 0.0f32;
        for cell in 0..g.len() {
            let c = g.centre[cell];
            let want = slope - c * c.dot(slope);
            let got = g.gradient(&field, cell, |_| false);
            worst = worst.max((got - want).length() / slope.length());
        }
        assert!(worst < 0.05, "worst relative error {worst}");
    }

    /// A rigid rotation of the sphere has no divergence.
    #[test]
    fn a_rigid_rotation_does_not_diverge() {
        let g = grid();
        let axis = Vec3::new(0.2, 1.0, -0.1).normalize();
        let field: Vec<Vec3> = g.centre.iter().map(|c| axis.cross(*c) * 10.0).collect();
        let worst = (0..g.len())
            .map(|cell| g.divergence(&field, cell, |_| false).abs())
            .fold(0.0f32, f32::max);
        // Ten metres a second over a 360 m cell is 0.03 per second; the
        // residual must be a small fraction of that.
        assert!(worst < 2e-4, "worst divergence {worst}");
    }

    #[test]
    fn a_located_direction_interpolates_back_to_itself() {
        let g = grid();
        let field: Vec<f32> = g
            .centre
            .iter()
            .map(|c| c.x * 3.0 + c.y - c.z * 2.0)
            .collect();
        for i in 0..500 {
            let t = i as f32 * 2.399_963;
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / 500.0;
            let r = (1.0 - y * y).sqrt();
            let d = Vec3::new(r * t.cos(), y, r * t.sin());
            let (cells, w) = g.locate(d);
            assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-4);
            assert!(w.iter().all(|&x| x >= 0.0));
            let back: Vec3 = cells
                .iter()
                .zip(w)
                .map(|(&c, w)| g.centre[c as usize] * w)
                .sum();
            assert!(
                back.normalize().dot(d) > 0.9995,
                "{d:?} came back as {back:?}"
            );
            let want = d.x * 3.0 + d.y - d.z * 2.0;
            assert!((g.sample(&field, d) - want).abs() < 0.05, "{d:?}");
        }
    }
}
