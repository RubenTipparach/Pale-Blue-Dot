//! Versioned deterministic CPU terrain reference for one planar hex patch.
//! This is a baseline sampler, not the eventual spherical multibiome generator.
//! Integer-coordinate hashes make chunk order irrelevant. Never change this
//! algorithm for an existing save without bumping GENERATOR_VERSION.

use crate::hex::{Hex, Voxel};

/// Version 2 is the spherical generator in [`crate::planet_gen`]; version 3
/// splits its fields into planet-scale and land-scale, which moves every
/// height on the body.
pub const GENERATOR_VERSION: u32 = 4;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Soil = 2,
    Grass = 3,
    Water = 4,
    Ore = 5,
    /// The top blocks the spherical generator hands out beyond the planar
    /// sampler's own: a beach or a desert, a yellow-green pasture, the deep
    /// green of a jungle, a snowfield, a rocky outcrop, bare dirt.
    Sand = 6,
    DryGrass = 7,
    JungleGrass = 8,
    Snow = 9,
    Rock = 10,
    Dirt = 11,
}

#[derive(Clone, Copy, Debug)]
pub struct TerrainGenerator {
    pub seed: u64,
    pub sea_level: i32,
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl TerrainGenerator {
    fn hash(self, q: i32, r: i32, layer: i32, channel: u64) -> u64 {
        let mut value = mix(self.seed ^ channel ^ GENERATOR_VERSION as u64);
        for coordinate in [q, r, layer] {
            value = mix(value ^ (coordinate as i64 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        }
        value
    }

    fn lattice(self, q: i32, r: i32, scale: i32) -> f64 {
        let a = q.div_euclid(scale);
        let b = r.div_euclid(scale);
        let u = q.rem_euclid(scale) as f64 / scale as f64;
        let v = r.rem_euclid(scale) as f64 / scale as f64;
        let u = u * u * (3.0 - 2.0 * u);
        let v = v * v * (3.0 - 2.0 * v);
        let sample = |q, r| {
            ((self.hash(q, r, 0, scale as u64) >> 40) as f64 / ((1_u64 << 24) - 1) as f64) * 2.0
                - 1.0
        };
        let lower = sample(a, b) * (1.0 - u) + sample(a + 1, b) * u;
        let upper = sample(a, b + 1) * (1.0 - u) + sample(a + 1, b + 1) * u;
        lower * (1.0 - v) + upper * v
    }

    pub fn height(self, hex: Hex) -> i32 {
        (self.lattice(hex.q, hex.r, 64) * 26.0 + self.lattice(hex.q, hex.r, 16) * 7.0 + 8.0).floor()
            as i32
    }

    pub fn sample(self, voxel: Voxel) -> Material {
        let height = self.height(voxel.hex);
        if voxel.layer > height {
            if voxel.layer <= self.sea_level {
                Material::Water
            } else {
                Material::Air
            }
        } else if voxel.layer == height && height > self.sea_level {
            Material::Grass
        } else if voxel.layer >= height - 3 {
            Material::Soil
        } else if self
            .hash(voxel.hex.q, voxel.hex.r, voxel.layer, 0x0a_e1)
            .is_multiple_of(127)
        {
            Material::Ore
        } else {
            Material::Stone
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_is_independent_of_visit_order_and_changes_with_seed() {
        let generator = TerrainGenerator {
            seed: 42,
            sea_level: 0,
        };
        let voxels: Vec<_> = (-100..100)
            .map(|q| Voxel {
                hex: Hex { q, r: -q / 3 },
                layer: 5,
            })
            .collect();
        let forwards: Vec<_> = voxels
            .iter()
            .map(|&voxel| generator.sample(voxel))
            .collect();
        let mut backwards: Vec<_> = voxels
            .iter()
            .rev()
            .map(|&voxel| generator.sample(voxel))
            .collect();
        backwards.reverse();
        assert_eq!(forwards, backwards);
        let different = TerrainGenerator {
            seed: 43,
            ..generator
        };
        assert!(
            voxels
                .iter()
                .any(|&voxel| generator.sample(voxel) != different.sample(voxel))
        );
    }

    #[test]
    fn chunk_boundaries_reconstruct_the_same_sample_and_water_is_bounded() {
        let generator = TerrainGenerator {
            seed: 1234,
            sea_level: 0,
        };
        for q in -65..65 {
            for layer in [-64, -33, -1, 0, 1, 31, 32, 64] {
                let voxel = Voxel {
                    hex: Hex { q, r: -q },
                    layer,
                };
                let (chunk, local) = voxel.split();
                let material = generator.sample(voxel);
                assert_eq!(material, generator.sample(Voxel::join(chunk, local)));
                if material == Material::Water {
                    assert!(layer <= generator.sea_level);
                }
                if layer == 64 {
                    assert_eq!(material, Material::Air);
                }
            }
        }
    }

    #[test]
    fn adjacent_columns_remain_continuous_across_negative_chunk_edges() {
        let generator = TerrainGenerator {
            seed: 9,
            sea_level: 0,
        };
        for q in -200..200 {
            let height = generator.height(Hex { q, r: -32 });
            let neighbor = generator.height(Hex { q: q + 1, r: -32 });
            assert!((height - neighbor).abs() <= 3);
        }
    }
}
