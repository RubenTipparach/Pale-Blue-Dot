//! Local generation of the dual at any level, addressed by lattice point.
//!
//! The finest tier is a cap around the player and never a whole globe, so it
//! cannot be a prefix of anything resident. A vertex at level `L` on icosahedron
//! face `f` is the lattice point `(i, j)` with `i + j <= 2^L`; its position is
//! the recursive midpoint construction, add-then-normalise exactly as
//! `dual_sphere` does, so a point the two generators share is the same float.
//! The cells come from enumerating the level-`L` triangles that intersect a
//! cap and building the dual from them with the same construction the whole
//! sphere uses. Design: `openspec/changes/hexagon-lod/design.md`,
//! "Implementation decisions".

use super::topology::{DualCell, dual_from_triangles, icosahedron, midpoint};
use bevy::prelude::*;
use std::collections::HashMap;

/// A vertex of the subdivided icosahedron by address. The same point has one
/// address per face it lies on; positions agree because the construction only
/// ever consults the edge the point bisects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LatticePoint {
    pub face: u8,
    pub level: u8,
    pub i: u32,
    pub j: u32,
}

/// A cell of the local dual with its lattice identity and its owners at the
/// level below: the coarse cell it is centred on when `(i, j)` are both even,
/// otherwise the two coarse cells whose edge it bisects.
#[derive(Debug)]
pub struct LocalCell {
    pub cell: DualCell,
    pub point: LatticePoint,
    pub owners: [Vec3; 2],
}

pub struct Lattice {
    vertices: Vec<Vec3>,
    faces: Vec<[usize; 3]>,
    memo: HashMap<LatticePoint, Vec3>,
}

impl Default for Lattice {
    fn default() -> Self {
        let (vertices, faces) = icosahedron();
        Self {
            vertices,
            faces,
            memo: HashMap::new(),
        }
    }
}

impl Lattice {
    /// Position of a lattice point, memoised: every point is built once per
    /// generator however many triangles touch it.
    pub fn position(&mut self, point: LatticePoint) -> Vec3 {
        if let Some(&position) = self.memo.get(&point) {
            return position;
        }
        let LatticePoint { face, level, i, j } = point;
        let n = 1u32 << level;
        debug_assert!(i + j <= n, "lattice point outside its face");
        let position = if level == 0 {
            let [a, b, c] = self.faces[face as usize];
            match (i, j) {
                (0, 0) => self.vertices[a],
                (1, 0) => self.vertices[b],
                _ => self.vertices[c],
            }
        } else {
            let below = |i, j| LatticePoint {
                face,
                level: level - 1,
                i,
                j,
            };
            match (i % 2, j % 2) {
                (0, 0) => self.position(below(i / 2, j / 2)),
                (1, 0) => midpoint(
                    self.position(below((i - 1) / 2, j / 2)),
                    self.position(below((i + 1) / 2, j / 2)),
                ),
                (0, _) => midpoint(
                    self.position(below(i / 2, (j - 1) / 2)),
                    self.position(below(i / 2, (j + 1) / 2)),
                ),
                // Both odd: the midpoint of the diagonal edge `i + j = const`,
                // which is the only edge of the coarser lattice through it.
                _ => midpoint(
                    self.position(below((i + 1) / 2, (j - 1) / 2)),
                    self.position(below((i - 1) / 2, (j + 1) / 2)),
                ),
            }
        };
        self.memo.insert(point, position);
        position
    }

    /// The coarse cells a point belongs to at the level below.
    pub fn owners(&mut self, point: LatticePoint) -> [Vec3; 2] {
        let LatticePoint { face, level, i, j } = point;
        if level == 0 {
            let p = self.position(point);
            return [p, p];
        }
        let below = |i, j| LatticePoint {
            face,
            level: level - 1,
            i,
            j,
        };
        match (i % 2, j % 2) {
            (0, 0) => {
                let p = self.position(below(i / 2, j / 2));
                [p, p]
            }
            (1, 0) => [
                self.position(below((i - 1) / 2, j / 2)),
                self.position(below((i + 1) / 2, j / 2)),
            ],
            (0, _) => [
                self.position(below(i / 2, (j - 1) / 2)),
                self.position(below(i / 2, (j + 1) / 2)),
            ],
            _ => [
                self.position(below((i + 1) / 2, (j - 1) / 2)),
                self.position(below((i - 1) / 2, (j + 1) / 2)),
            ],
        }
    }

    /// Every level-`level` triangle whose bounding cap meets the cap of angular
    /// radius `radius` (radians) around `center`, as lattice points. A quadtree
    /// descent per face, pruned by the angle between caps, so the cost is the
    /// cap's own triangle count and not the sphere's.
    pub fn triangles_in_cap(
        &mut self,
        level: u8,
        center: Vec3,
        radius: f32,
    ) -> Vec<[LatticePoint; 3]> {
        let center = center.normalize_or(Vec3::Y);
        let mut out = Vec::new();
        for face in 0..self.faces.len() as u8 {
            self.descend(
                face,
                level,
                0,
                0,
                1 << level,
                true,
                center,
                radius,
                &mut out,
            );
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn descend(
        &mut self,
        face: u8,
        level: u8,
        i0: u32,
        j0: u32,
        size: u32,
        up: bool,
        center: Vec3,
        radius: f32,
        out: &mut Vec<[LatticePoint; 3]>,
    ) {
        let corners = if up {
            [(i0, j0), (i0 + size, j0), (i0, j0 + size)]
        } else {
            [(i0 + size, j0), (i0, j0 + size), (i0 + size, j0 + size)]
        };
        let points = corners.map(|(i, j)| LatticePoint { face, level, i, j });
        let positions = points.map(|p| self.position(p));
        let centroid = (positions[0] + positions[1] + positions[2]).normalize_or(Vec3::Y);
        let reach = positions
            .iter()
            .map(|p| p.dot(centroid).clamp(-1.0, 1.0).acos())
            .fold(0.0_f32, f32::max);
        let apart = centroid.dot(center).clamp(-1.0, 1.0).acos();
        if apart > radius + reach {
            return;
        }
        if size == 1 {
            out.push(points);
            return;
        }
        let half = size / 2;
        if up {
            self.descend(face, level, i0, j0, half, true, center, radius, out);
            self.descend(face, level, i0 + half, j0, half, true, center, radius, out);
            self.descend(face, level, i0, j0 + half, half, true, center, radius, out);
            self.descend(face, level, i0, j0, half, false, center, radius, out);
        } else {
            self.descend(face, level, i0 + half, j0, half, false, center, radius, out);
            self.descend(face, level, i0, j0 + half, half, false, center, radius, out);
            self.descend(
                face,
                level,
                i0 + half,
                j0 + half,
                half,
                false,
                center,
                radius,
                out,
            );
            self.descend(
                face,
                level,
                i0 + half,
                j0 + half,
                half,
                true,
                center,
                radius,
                out,
            );
        }
    }

    /// The complete cells of the level-`level` dual whose centres lie within
    /// `radius` radians of `center`. Triangles are enumerated out to two more
    /// tiles so every cell inside has its whole ring; cells whose ring is cut by
    /// the enumeration edge are dropped rather than returned incomplete.
    pub fn cells_in_cap(&mut self, level: u8, center: Vec3, radius: f32) -> Vec<LocalCell> {
        let center = center.normalize_or(Vec3::Y);
        let tile = 1.2087 / (1u32 << level) as f32;
        let triangles = self.triangles_in_cap(level, center, radius + 2.5 * tile);
        // Dedupe vertices on their bits: a point shared by two faces was built
        // from the same parents in the same order and is the same float.
        let mut index_of: HashMap<[u32; 3], usize> = HashMap::new();
        let mut vertices: Vec<Vec3> = Vec::new();
        let mut points: Vec<LatticePoint> = Vec::new();
        let mut indexed = Vec::with_capacity(triangles.len());
        for tri in &triangles {
            let mut ids = [0usize; 3];
            for (slot, point) in ids.iter_mut().zip(tri) {
                let position = self.position(*point);
                let key = position.to_array().map(f32::to_bits);
                *slot = *index_of.entry(key).or_insert_with(|| {
                    vertices.push(position);
                    points.push(*point);
                    vertices.len() - 1
                });
            }
            indexed.push(ids);
        }
        let cos_radius = radius.cos();
        let mut cells = Vec::new();
        for (index, cell) in dual_from_triangles(&vertices, &indexed)
            .into_iter()
            .enumerate()
        {
            let Some(cell) = cell else {
                continue;
            };
            if cell.direction.dot(center) < cos_radius {
                continue;
            }
            let point = points[index];
            let owners = self.owners(point);
            cells.push(LocalCell {
                cell,
                point,
                owners,
            });
        }
        // Neighbour indices from `dual_from_triangles` refer to the deduped
        // vertex list; remap them to this cell list, with `usize::MAX` where
        // the neighbour is outside the cap.
        let mut slot_of = vec![usize::MAX; vertices.len()];
        for (slot, local) in cells.iter().enumerate() {
            let key = local.cell.direction.to_array().map(f32::to_bits);
            slot_of[index_of[&key]] = slot;
        }
        for local in &mut cells {
            for neighbor in &mut local.cell.neighbors {
                *neighbor = slot_of[*neighbor];
            }
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::topology::dual_sphere;

    /// Every level-5 point the lattice can address is a vertex of
    /// `dual_sphere(5)`, at the same float, and the two generators agree on
    /// the corner ring and the neighbour directions of every cell.
    #[test]
    fn lattice_points_are_dual_sphere_vertices_bit_for_bit() {
        let sphere = dual_sphere(5);
        let by_bits: HashMap<[u32; 3], usize> = sphere
            .iter()
            .enumerate()
            .map(|(index, cell)| (cell.direction.to_array().map(f32::to_bits), index))
            .collect();
        let mut lattice = Lattice::default();
        let mut matched = 0;
        for face in 0..20u8 {
            for i in 0..=32u32 {
                for j in 0..=(32 - i) {
                    let point = LatticePoint {
                        face,
                        level: 5,
                        i,
                        j,
                    };
                    let key = lattice.position(point).to_array().map(f32::to_bits);
                    assert!(
                        by_bits.contains_key(&key),
                        "{point:?} is not a sphere vertex"
                    );
                    matched += 1;
                }
            }
        }
        assert_eq!(matched, 20 * 33 * 34 / 2);
        // The whole sphere as one cap: the same cells, corner for corner.
        let cells = lattice.cells_in_cap(5, Vec3::Y, std::f32::consts::PI);
        assert_eq!(cells.len(), sphere.len());
        for local in &cells {
            let reference = &sphere[by_bits[&local.cell.direction.to_array().map(f32::to_bits)]];
            // Corners are triangle centres, and the two generators sum a
            // triangle's vertices in a different order, so they agree to an
            // ulp rather than to the bit. Vertices, which are what levels
            // share, agree to the bit.
            assert_eq!(local.cell.corners.len(), reference.corners.len());
            for (a, b) in local.cell.corners.iter().zip(&reference.corners) {
                assert!(a.distance(*b) < 1e-6);
            }
            for (a, b) in local.cell.neighbors.iter().zip(&reference.neighbors) {
                assert_eq!(cells[*a].cell.direction, sphere[*b].direction);
            }
        }
    }

    /// A cap is complete inside and its owners are the level below: a
    /// vertex-centred cell owns itself, a midpoint cell is owned by the two
    /// coarse cells whose edge it bisects, and both are coarse vertices.
    #[test]
    fn a_cap_is_complete_and_owned_by_the_level_below() {
        let mut lattice = Lattice::default();
        let center = Vec3::new(0.3, 0.8, -0.5).normalize();
        let coarse: HashMap<[u32; 3], ()> = lattice
            .cells_in_cap(6, center, 0.2)
            .into_iter()
            .map(|c| (c.cell.direction.to_array().map(f32::to_bits), ()))
            .collect();
        let fine = lattice.cells_in_cap(7, center, 0.1);
        // A 0.1 rad cap holds pi * 0.01 sr over a level-7 cell's 7.7e-5 sr.
        assert!((350..470).contains(&fine.len()), "{} cells", fine.len());
        let mut vertex_centred = 0;
        for local in &fine {
            assert!([5, 6].contains(&local.cell.corners.len()));
            for &neighbor in &local.cell.neighbors {
                // Inside the cap by more than a tile, every neighbour is present.
                if local.cell.direction.dot(center) > (0.1 - 0.02_f32).cos() {
                    assert_ne!(neighbor, usize::MAX);
                }
            }
            let [a, b] = local.owners;
            assert!(coarse.contains_key(&a.to_array().map(f32::to_bits)));
            assert!(coarse.contains_key(&b.to_array().map(f32::to_bits)));
            if a == b {
                vertex_centred += 1;
                assert_eq!(a, local.cell.direction);
            } else {
                // The midpoint of its owners' edge, by the shared construction.
                assert_eq!(midpoint(a, b), local.cell.direction);
            }
        }
        // One vertex-centred cell per coarse cell, three midpoint cells each.
        assert!((fine.len() as f32 / vertex_centred as f32 - 4.0).abs() < 0.2);
    }

    fn mean_tile_width(cells: &[LocalCell], radius_m: f32) -> f32 {
        let (mut total, mut count) = (0.0f64, 0u32);
        for local in cells {
            for &neighbor in &local.cell.neighbors {
                if neighbor != usize::MAX {
                    total += (local
                        .cell
                        .direction
                        .distance(cells[neighbor].cell.direction)
                        * radius_m) as f64;
                    count += 1;
                }
            }
        }
        (total / f64::from(count.max(1))) as f32
    }

    /// The gold standard, measured on the tier a player stands on. One cap
    /// sits somewhere on the dual's own +/-9% distortion, so it must land in
    /// the measured spread; the mean over caps at the icosahedron's vertices,
    /// face centres and edge midpoints, which sample that pattern evenly, is
    /// the global 2.833 m. And a level-11 cap of 300 m is a few tens of
    /// thousands of cells, not forty-two million.
    #[test]
    fn a_level_eleven_cap_measures_the_gold_standard_tile() {
        let mut lattice = Lattice::default();
        let radius_m = 4_800.0_f32;
        let cells = lattice.cells_in_cap(11, Vec3::new(0.8776, 0.4794, 0.0), 300.0 / radius_m);
        assert!(
            (35_000..50_000).contains(&cells.len()),
            "{} cells in the cap",
            cells.len()
        );
        let local = mean_tile_width(&cells, radius_m);
        assert!(
            (2.595..=3.101).contains(&local),
            "one cap measured {local} m"
        );
        // Caps on a Fibonacci spiral sample the sphere's area evenly, which
        // the icosahedron's own vertices, face centres and edge midpoints do
        // not: those over-weight the face centres, where tiles are largest,
        // and read 2.4% high.
        let samples: Vec<Vec3> = (0..256)
            .map(|k| {
                let y = 1.0 - 2.0 * (k as f32 + 0.5) / 256.0;
                let r = (1.0 - y * y).sqrt();
                let angle = k as f32 * 2.399_963;
                Vec3::new(r * angle.cos(), y, r * angle.sin())
            })
            .collect();
        let mut total = 0.0;
        for center in &samples {
            let cap = lattice.cells_in_cap(11, *center, 40.0 / radius_m);
            total += mean_tile_width(&cap, radius_m);
        }
        let global = total / samples.len() as f32;
        assert!(
            (global - 2.833).abs() < 0.02,
            "global mean tile width {global} m"
        );
    }
}
