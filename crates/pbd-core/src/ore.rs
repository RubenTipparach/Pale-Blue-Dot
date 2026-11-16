//! Ore veins in the ground (`flora-v2` design 12.8, generator version 10).
//!
//! Space is cut into cubes in the body's frame; a cube holds a vein on its
//! own hash, at a share that falls with depth, and inside a vein cube each
//! metre cell is ore on its own hash. So ore comes in clumps a few cells
//! across, often just under a hillside's soil. Derived from the seed and the
//! cell, never saved; a version without veins (`TerrainConfig::ore` `None`)
//! has none. The ore is the game's one ore block, mixed ore, which digs to
//! coal or mixed ore (`station::ore_yield`) and smelts to copper, tin and
//! iron.

use crate::fauna::hash2;
use glam::Vec3;

/// The veins' numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OreVeins {
    /// The side of a vein cube, m.
    pub cube_m: f32,
    /// Within this depth of the surface, m, the shallow share holds.
    pub shallow_m: f32,
    /// The share of cubes holding a vein, shallow and deeper.
    pub shallow_share: f32,
    pub deep_share: f32,
    /// The share of a vein cube's cells that are ore.
    pub fill: f32,
}

impl OreVeins {
    /// Version 10's veins (design 12.8).
    pub const V10: Self = Self {
        cube_m: 3.0,
        shallow_m: 14.0,
        shallow_share: 0.18,
        deep_share: 0.07,
        fill: 0.55,
    };

    /// Whether the stone at `altitude` (m over the body's radius) under
    /// `direction`, `depth_m` below the surface, is ore.
    pub fn is_ore(
        &self,
        seed: u64,
        radius_m: f32,
        direction: Vec3,
        altitude: f32,
        depth_m: f32,
    ) -> bool {
        let p = direction.normalize_or(Vec3::Y) * (radius_m + altitude);
        let grid = |size: f32| -> u64 {
            let q = (p / size).floor();
            let (x, y, z) = (q.x as i64 as u64, q.y as i64 as u64, q.z as i64 as u64);
            hash2(hash2(seed ^ 0x0e_7e1d, x), hash2(y, z))
        };
        let share = if depth_m <= self.shallow_m {
            self.shallow_share
        } else {
            self.deep_share
        };
        let unit = |h: u64| (h >> 11) as f32 / (1u64 << 53) as f32;
        if unit(grid(self.cube_m)) >= share {
            return false;
        }
        unit(hash2(grid(1.0), 0x0005_ea0e)) < self.fill
    }
}
