//! What the world map shows at a point on the planet: the ground's own top
//! block and biome, at the altitude the terrain draws.
//!
//! One function the map mockup's rasters and, later, the game's map both
//! call (`world-map` decision 1), so the map cannot show a planet the ground
//! does not have. It is the column's rule, not a second copy of it: the
//! altitude is [`column::surface_m`], floored to the layer as every column
//! is, and the biome and top block are read at that altitude, as the
//! terrain's surface code reads them.
//!
//! Colour is not decided here. What a top block looks like is the renderer's
//! business, and the map takes it from the same tiles.

use crate::column;
use crate::planet_gen::{Biome, TerrainConfig, biome_at, top_material};
use crate::terrain::Material;
use glam::Vec3;

/// One pixel of the base map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texel {
    /// The surface above sea level, in metres, floored to the layer as the
    /// column is. Negative is the sea floor.
    pub altitude_m: f32,
    /// The biome the terrain classifies the surface as.
    pub biome: Biome,
    /// The ground's top block, which is what the base map is coloured by
    /// (survey M3: "normal planet terrain like what it looks like on the
    /// world").
    pub top: Material,
    /// Under water: the surface is below the sea level, as a column floods.
    pub sea: bool,
}

/// What the map shows at `direction`, a direction from the planet's centre.
/// Any length but zero; it is normalised here.
pub fn base_texel(cfg: &TerrainConfig, direction: Vec3) -> Texel {
    let d = direction.normalize_or(Vec3::Y);
    let altitude_m = column::surface_m(cfg, d);
    Texel {
        altitude_m,
        biome: biome_at(cfg, d, altitude_m),
        top: top_material(cfg, d, altitude_m),
        sea: altitude_m < cfg.sea_level_m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet_gen::surface_altitude;

    /// Seeded directions spread over the sphere.
    fn directions(n: usize) -> impl Iterator<Item = Vec3> {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        std::iter::repeat_with(move || {
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
            };
            loop {
                let v = Vec3::new(next(), next(), next());
                let length = v.length();
                if length > 0.05 && length <= 1.0 {
                    return v / length;
                }
            }
        })
        .take(n)
    }

    /// The map shows what the ground is: the same altitude, biome, top block
    /// and water as the column the terrain builds there, on 10,000 seeded
    /// directions (`world-map` task 1.1).
    #[test]
    fn the_map_agrees_with_the_ground_on_ten_thousand_directions() {
        let cfg = TerrainConfig::TENEBRIS;
        let mut land = 0;
        for d in directions(10_000) {
            let texel = base_texel(&cfg, d);
            let surface = column::surface_m(&cfg, d);
            assert_eq!(texel.altitude_m, surface.floor(), "{d}");
            assert_eq!(texel.altitude_m, surface_altitude(&cfg, d).floor());
            assert_eq!(texel.biome, biome_at(&cfg, d, surface), "{d}");
            assert_eq!(texel.top, top_material(&cfg, d, surface), "{d}");
            // The column floods every layer from its surface up to the sea
            // level (`column::generate_solid`), so the surface is under water
            // exactly when it is below the sea level.
            assert_eq!(texel.sea, surface < cfg.sea_level_m, "{d}");
            land += usize::from(!texel.sea);
        }
        assert!(
            (2_000..8_000).contains(&land),
            "{land} of 10,000 on land: the sample is not all one thing"
        );
    }

    /// A direction's length does not matter.
    #[test]
    fn any_length_of_direction_is_the_same_place() {
        let cfg = TerrainConfig::TENEBRIS;
        for d in directions(100) {
            assert_eq!(base_texel(&cfg, d), base_texel(&cfg, d * 4_800.0));
        }
    }
}
