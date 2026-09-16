//! Deterministic dual of a subdivided icosahedron. Shared triangle centers are
//! the corners of the dual, so adjacent columns cannot develop seam cracks.

use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct DualCell {
    pub direction: Vec3,
    pub corners: Vec<Vec3>,
    pub neighbors: Vec<usize>,
}

pub(super) fn dual_sphere(level: u32) -> Vec<DualCell> {
    assert!(
        level <= 8,
        "preview topology has a bounded allocation budget"
    );
    let t = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let mut vertices = vec![
        Vec3::new(-1., t, 0.),
        Vec3::new(1., t, 0.),
        Vec3::new(-1., -t, 0.),
        Vec3::new(1., -t, 0.),
        Vec3::new(0., -1., t),
        Vec3::new(0., 1., t),
        Vec3::new(0., -1., -t),
        Vec3::new(0., 1., -t),
        Vec3::new(t, 0., -1.),
        Vec3::new(t, 0., 1.),
        Vec3::new(-t, 0., -1.),
        Vec3::new(-t, 0., 1.),
    ];
    for v in &mut vertices {
        *v = v.normalize();
    }
    let mut triangles = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    for _ in 0..level {
        let mut edges = BTreeMap::new();
        let mut subdivided = Vec::with_capacity(triangles.len() * 4);
        for [a, b, c] in triangles {
            let mut midpoint = |i: usize, j: usize| {
                *edges.entry((i.min(j), i.max(j))).or_insert_with(|| {
                    let index = vertices.len();
                    vertices.push((vertices[i] + vertices[j]).normalize());
                    index
                })
            };
            let ab = midpoint(a, b);
            let bc = midpoint(b, c);
            let ca = midpoint(c, a);
            subdivided.extend([[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        triangles = subdivided;
    }
    let centers: Vec<_> = triangles
        .iter()
        .map(|[a, b, c]| (vertices[*a] + vertices[*b] + vertices[*c]).normalize())
        .collect();
    let mut incident = vec![Vec::new(); vertices.len()];
    for (index, tri) in triangles.iter().enumerate() {
        for v in tri {
            incident[*v].push(index);
        }
    }
    vertices
        .iter()
        .enumerate()
        .map(|(index, direction)| {
            let tangent = direction.any_orthonormal_vector();
            let bitangent = direction.cross(tangent);
            incident[index].sort_by(|a, b| {
                let a = centers[*a];
                let b = centers[*b];
                a.dot(bitangent)
                    .atan2(a.dot(tangent))
                    .total_cmp(&b.dot(bitangent).atan2(b.dot(tangent)))
            });
            let ring = &incident[index];
            let neighbors = (0..ring.len())
                .map(|side| {
                    let a = triangles[ring[side]];
                    let b = triangles[ring[(side + 1) % ring.len()]];
                    *a.iter()
                        .find(|&&v| v != index && b.contains(&v))
                        .expect("adjacent dual corners share one primal edge")
                })
                .collect();
            DualCell {
                direction: *direction,
                corners: ring.iter().map(|&tri| centers[tri]).collect(),
                neighbors,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_topology_has_twelve_pentagons_and_shared_outward_edges() {
        for level in 0..=4 {
            let cells = dual_sphere(level);
            assert_eq!(cells.len(), 10 * 4_usize.pow(level) + 2);
            assert_eq!(cells.iter().filter(|c| c.corners.len() == 5).count(), 12);
            for (index, cell) in cells.iter().enumerate() {
                assert!([5, 6].contains(&cell.corners.len()));
                for side in 0..cell.corners.len() {
                    let a = cell.corners[side];
                    let b = cell.corners[(side + 1) % cell.corners.len()];
                    assert!(
                        (a - cell.direction)
                            .cross(b - cell.direction)
                            .dot(cell.direction)
                            > 0.
                    );
                    let other = &cells[cell.neighbors[side]];
                    let opposite = other.neighbors.iter().position(|&n| n == index).unwrap();
                    assert_eq!(a, other.corners[(opposite + 1) % other.corners.len()]);
                    assert_eq!(b, other.corners[opposite]);
                }
            }
        }
    }
}
