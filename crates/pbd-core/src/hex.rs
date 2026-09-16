//! Pointy-top axial hexagonal prisms within ONE planet surface patch.
//! A sphere cannot have only hexagonal cells: face/edge ownership and the twelve
//! pentagonal defects of a spherical dual grid belong to the future topology layer.

use glam::DVec3;

pub const CHUNK_EDGE: i32 = 32;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    pub const DIRECTIONS: [Self; 6] = [
        Self { q: 1, r: 0 },
        Self { q: 1, r: -1 },
        Self { q: 0, r: -1 },
        Self { q: -1, r: 0 },
        Self { q: -1, r: 1 },
        Self { q: 0, r: 1 },
    ];

    /// Local planar coordinates; `radius` is centre-to-corner distance.
    pub fn center(self, height: f64, radius: f64) -> DVec3 {
        DVec3::new(
            3.0_f64.sqrt() * radius * (self.q as f64 + self.r as f64 * 0.5),
            height,
            1.5 * radius * self.r as f64,
        )
    }

    /// Nearest cell using cube-coordinate rounding, including negative positions.
    pub fn from_point(point: DVec3, radius: f64) -> Self {
        assert!(radius.is_finite() && radius > 0.0);
        assert!(point.is_finite());
        let q = (3.0_f64.sqrt() / 3.0 * point.x - point.z / 3.0) / radius;
        let r = (2.0 / 3.0 * point.z) / radius;
        let s = -q - r;
        let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
        if (rq - q).abs() > (rr - r).abs() && (rq - q).abs() > (rs - s).abs() {
            rq = -rr - rs;
        } else if (rr - r).abs() > (rs - s).abs() {
            rr = -rq - rs;
        }
        Self {
            q: rq as i32,
            r: rr as i32,
        }
    }

    pub fn distance(self, other: Self) -> u64 {
        let q = self.q as i64 - other.q as i64;
        let r = self.r as i64 - other.r as i64;
        (q.abs() + r.abs() + (q + r).abs()) as u64 / 2
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Voxel {
    pub hex: Hex,
    pub layer: i32,
}

/// A patch-local chunk address. Global cache keys also need planet + patch IDs.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Chunk {
    pub q: i32,
    pub r: i32,
    pub layer: i32,
}

impl Voxel {
    pub fn split(self) -> (Chunk, [u8; 3]) {
        (
            Chunk {
                q: self.hex.q.div_euclid(CHUNK_EDGE),
                r: self.hex.r.div_euclid(CHUNK_EDGE),
                layer: self.layer.div_euclid(CHUNK_EDGE),
            },
            [
                self.hex.q.rem_euclid(CHUNK_EDGE) as u8,
                self.hex.r.rem_euclid(CHUNK_EDGE) as u8,
                self.layer.rem_euclid(CHUNK_EDGE) as u8,
            ],
        )
    }

    pub fn join(chunk: Chunk, local: [u8; 3]) -> Self {
        assert!(local.iter().all(|&axis| axis < CHUNK_EDGE as u8));
        Self {
            hex: Hex {
                q: chunk.q * CHUNK_EDGE + local[0] as i32,
                r: chunk.r * CHUNK_EDGE + local[1] as i32,
            },
            layer: chunk.layer * CHUNK_EDGE + local[2] as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_round_trip_and_neighbors_are_one_cell_away() {
        for q in -64..64 {
            for r in -64..64 {
                let hex = Hex { q, r };
                assert_eq!(Hex::from_point(hex.center(8.0, 0.75), 0.75), hex);
                for step in Hex::DIRECTIONS {
                    assert_eq!(
                        hex.distance(Hex {
                            q: q + step.q,
                            r: r + step.r
                        }),
                        1
                    );
                }
            }
        }
    }

    #[test]
    fn negative_and_boundary_voxels_have_unique_chunk_ownership() {
        for n in [-65, -64, -33, -32, -31, -1, 0, 1, 31, 32, 33, 64] {
            let voxel = Voxel {
                hex: Hex { q: n, r: -n },
                layer: n,
            };
            let (chunk, local) = voxel.split();
            assert_eq!(Voxel::join(chunk, local), voxel);
        }
        assert_eq!(
            Voxel {
                hex: Hex { q: -1, r: 0 },
                layer: -33
            }
            .split(),
            (
                Chunk {
                    q: -1,
                    r: 0,
                    layer: -2
                },
                [31, 0, 31]
            )
        );
    }
}
