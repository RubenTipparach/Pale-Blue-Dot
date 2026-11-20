//! Mountain ground (`mountain-ground`): what a Mountains cell's relief says
//! about it, and generator version 15's flats, snow line, scoured crests and
//! tree line.
//!
//! Two kinds of rule live here, kept apart by CLAUDE.md's "Saved games
//! survive every change":
//!
//! - [`relief`] reads a cell's six neighbours off the generator's smooth
//!   surface. It is pure and holds for every version, so the draw-time looks
//!   (scree at a cliff's foot, snow lingering on a shaded flat) read it on
//!   every world, old saves included, and change no cell.
//! - [`MountainGround`] is version 15's: which top a flat or a crest takes,
//!   and where trees stand, decided at generation. A config without it
//!   (versions 4 to 14) never calls it.

use glam::Vec3;

use crate::planet_gen::{self, Biome, TerrainConfig};
use crate::terrain::Material;

/// How far off a cell's neighbours stand, m: the gold standard's mean tile
/// width (CLAUDE.md, "Hex size is fixed").
pub const NEIGHBOUR_M: f32 = 2.833;

/// The snow line the draw-time looks read on every world, m: decision 1's
/// lowered line, about 170 m. Version 15 rags its own by a noise
/// ([`MountainGround::snow_line`]); a draw-time look takes the mean.
pub const DRAW_SNOW_LINE_M: f32 = 170.0;

/// What a cell's six neighbours say about it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Relief {
    /// No neighbour more than a metre above or below it.
    pub flat: bool,
    /// A neighbour stands three metres or more above it: the foot of a
    /// cliff, where scree gathers.
    pub scree: bool,
    /// It stands above at least five of its six neighbours: a crest.
    pub crest: bool,
}

/// The six points a cell's neighbours stand at, about [`NEIGHBOUR_M`] off,
/// on the body's own east and north. Not the lattice's own neighbours (a
/// hex's turn varies over the sphere), but six samples of the same smooth
/// surface the cells are cut from, at their spacing.
pub fn neighbour_directions(cfg: &TerrainConfig, direction: Vec3) -> [Vec3; 6] {
    let d = direction.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
    let north = d.cross(east);
    std::array::from_fn(|k| {
        let a = k as f32 * std::f32::consts::TAU / 6.0;
        (d + (east * a.cos() + north * a.sin()) * (NEIGHBOUR_M / cfg.radius_m)).normalize()
    })
}

/// A cell's relief: its floored surface against its six neighbours'. Six
/// `surface_altitude` calls; the caller asks only on a Mountains cell.
pub fn relief(cfg: &TerrainConfig, direction: Vec3, surface_m: f32) -> Relief {
    let h0 = surface_m.floor();
    let mut flat = true;
    let mut scree = false;
    let mut below = 0;
    for n in neighbour_directions(cfg, direction) {
        let h = planet_gen::surface_altitude(cfg, n).floor();
        flat &= (h - h0).abs() <= 1.0;
        scree |= h - h0 >= 3.0;
        below += usize::from(h < h0);
    }
    Relief {
        flat,
        scree,
        crest: below >= 5,
    }
}

/// Generator version 15's mountain ground (`mountain-ground` decision 1 and
/// design 6 to 12). Every height and every biome is the version under it;
/// only a Mountains cell's top, and whether a tree stands on it, moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MountainGround {
    /// The mean snow line, m, and how far a coarse noise rags it, m.
    pub snow_line_m: f32,
    pub ragged_m: f32,
    /// The ragging noise's feature size, m.
    pub ragged_scale_m: f32,
    /// The bare-soil patches in the meadow: a noise of this size, m, and the
    /// value over which a flat is dirt rather than turf.
    pub soil_patch_m: f32,
    pub soil_above: f32,
    /// The tree line (design 12): full forest below `forest_to_m`; the
    /// cover thins to `line_share` of it at `fir_end_m`, where the firs end,
    /// and to none at `krummholz_to_m`. Krummholz stands from
    /// `krummholz_from_m`.
    pub forest_to_m: f32,
    pub fir_end_m: f32,
    pub krummholz_from_m: f32,
    pub krummholz_to_m: f32,
    pub line_share: f32,
}

/// The salt of the snow line's ragging and of the soil patches.
const SNOW_LINE_SALT: u64 = 0x5_70E1_1AE;
const SOIL_PATCH_SALT: u64 = 0x50_11A7;

impl MountainGround {
    /// Version 15's (`mountain-ground` decision 1, design 6 and 12).
    pub const V15: Self = Self {
        snow_line_m: 170.0,
        ragged_m: 15.0,
        ragged_scale_m: 160.0,
        soil_patch_m: 24.0,
        soil_above: 0.66,
        forest_to_m: 135.0,
        fir_end_m: 150.0,
        krummholz_from_m: 142.0,
        krummholz_to_m: 160.0,
        line_share: 0.35,
    };

    /// The ragged snow line at a direction, m.
    pub fn snow_line(&self, cfg: &TerrainConfig, direction: Vec3) -> f32 {
        let n = planet_gen::noise01(
            cfg.seed ^ SNOW_LINE_SALT,
            direction,
            cfg.scale_of(self.ragged_scale_m),
            2,
        );
        self.snow_line_m + self.ragged_m * (2.0 * n - 1.0)
    }

    /// A Mountains cell's top (decision 1, design 6): above the snow line a
    /// crest is scoured to stone, a flat holds snow, and a steep face is rock
    /// up to the old snowcap and snow over it; below the line a flat is
    /// alpine turf, or bare soil in patches, and a steep face is rock.
    pub fn top(&self, cfg: &TerrainConfig, direction: Vec3, surface_m: f32) -> Material {
        let r = relief(cfg, direction, surface_m);
        if surface_m > self.snow_line(cfg, direction) {
            if r.crest {
                Material::Stone
            } else if r.flat || surface_m > cfg.mountain_snowcap_elev_m {
                Material::Snow
            } else {
                Material::Stone
            }
        } else if r.flat {
            let patch = planet_gen::noise01(
                cfg.seed ^ SOIL_PATCH_SALT,
                direction,
                cfg.scale_of(self.soil_patch_m),
                2,
            );
            if patch > self.soil_above {
                Material::Dirt
            } else {
                Material::Grass
            }
        } else {
            Material::Stone
        }
    }

    /// The tree line at an altitude: how much of the biome's density is
    /// thinned away, in sixteenths less one (0 none, 15 all), and whether
    /// the tree that stands is krummholz rather than a fir.
    pub fn tree_line(&self, surface_m: f32) -> (u32, bool) {
        let a = surface_m;
        let share = if a < self.forest_to_m {
            1.0
        } else if a < self.fir_end_m {
            let t = (a - self.forest_to_m) / (self.fir_end_m - self.forest_to_m);
            1.0 + (self.line_share - 1.0) * t
        } else if a < self.krummholz_to_m {
            let t = (a - self.fir_end_m) / (self.krummholz_to_m - self.fir_end_m);
            self.line_share * (1.0 - t)
        } else {
            0.0
        };
        let thin = (15.0 * (1.0 - share)).round().clamp(0.0, 15.0) as u32;
        (thin, a >= self.krummholz_from_m && thin < 15)
    }
}

/// The tree line a record carries (design 12): the thinning and the
/// krummholz bit, from the world's config, on a Mountains cell of a world
/// that has version 15's ground; nought and false everywhere else.
pub fn tree_line(cfg: &TerrainConfig, surface_m: f32, biome: Biome) -> (u32, bool) {
    match (&cfg.mountains, biome) {
        (Some(m), Biome::Mountains) => m.tree_line(surface_m),
        _ => (0, false),
    }
}

/// The draw-time relief bits a record carries on a Mountains cell, every
/// world: scree at a cliff's foot, and a flat (where snow may linger).
/// Nought elsewhere, where nothing reads them.
pub fn draw_relief(cfg: &TerrainConfig, direction: Vec3, surface_m: f32, biome: Biome) -> Relief {
    if biome == Biome::Mountains {
        relief(cfg, direction, surface_m)
    } else {
        Relief::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tree_line_thins_to_none() {
        let m = MountainGround::V15;
        assert_eq!(m.tree_line(110.0), (0, false));
        assert_eq!(m.tree_line(134.9), (0, false));
        let (mid, k) = m.tree_line(142.5);
        assert!(mid > 0 && mid < 15 && k, "{mid}");
        let (high, k) = m.tree_line(155.0);
        assert!(high > mid && k);
        assert_eq!(m.tree_line(160.0), (15, false));
        assert_eq!(m.tree_line(250.0), (15, false));
        // It never thickens with altitude.
        let mut last = 0;
        for a in 100..170 {
            let (t, _) = m.tree_line(a as f32);
            assert!(t >= last);
            last = t;
        }
    }
}
