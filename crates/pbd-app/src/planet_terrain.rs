//! Stable generation inputs shared by the flight clearance policy and GPU
//! upload. The GPU shades derived geometry; it never decides terrain heights.

use bevy::prelude::*;
use pbd_core::planet_gen::{self, Biome, TerrainConfig};

/// Sea-level radius in metres. It sits on the gold-standard ladder
/// `R = 300 m * 2^(L - 7)`: level 11 underfoot gives the 2.833 m Tenebris tile
/// (see `lod::tile_width_m` and the CLAUDE.md rule on hex size).
pub const PLANET_RADIUS: f32 = 4_800.0;

/// Vertical quantum of the surface, in metres: one column cap sits this far
/// above the next. It is the world's height resolution, so it belongs beside
/// the radius rather than inline in the generator. One metre is the Tenebris
/// cell height, and the walker's step is sized off it.
pub const ELEVATION_STEP: f32 = 1.0;

/// The generator: Tenebris's, ported into `pbd_core::planet_gen` with the
/// heights authored for this body. One config, one source of defaults; a
/// second body is a second value of it.
pub(super) const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;

/// Quantized terrain elevation above sea level, in metres, on a unit ray.
/// Normalizing here also makes the collision query safe for arbitrary poses.
pub fn surface_height(direction: Vec3) -> f32 {
    let d = direction.normalize_or(Vec3::Y);
    let height = planet_gen::surface_altitude(&TERRAIN, d);
    (height / ELEVATION_STEP).floor() * ELEVATION_STEP
}

/// Solid terrain or water surface radius for assisted-flight clearance.
pub fn terrain_radius(direction: Vec3) -> f32 {
    PLANET_RADIUS + surface_height(direction).max(0.)
}

/// The material index the surface shader reads, off the generator's top
/// block and biome. The shader's table is the authority for what each index
/// draws (0 seabed, 1 beach, 2 pasture, 3 jungle, 4 desert, 5 stone, 6 snow,
/// 7 marsh) and for which of them carry trees.
pub(super) fn biome(direction: Vec3, height: f32) -> u32 {
    use pbd_core::terrain::Material;
    let d = direction.normalize_or(Vec3::Y);
    let biome = planet_gen::biome_at(&TERRAIN, d, height);
    match (planet_gen::top_material(&TERRAIN, d, height), biome) {
        (Material::Sand, _) if height < TERRAIN.sea_level_m => 0,
        (Material::Sand, Biome::Desert) => 4,
        (Material::Sand, _) => 1,
        (Material::Stone, _) | (Material::Rock, _) => 5,
        (Material::Snow, _) => 6,
        (Material::JungleGrass, _) => 3,
        (Material::Grass, Biome::Swamp) => 7,
        (Material::Dirt, _) => 1,
        (Material::Water, _) => 0,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearance_is_finite_quantized_and_covers_land_and_ocean() {
        let mut land = 0;
        let mut ocean = 0;
        for cell in super::super::topology::dual_sphere(3) {
            let h = surface_height(cell.direction);
            assert!(h.is_finite());
            assert_eq!(h % ELEVATION_STEP, 0.);
            assert_eq!(terrain_radius(cell.direction), PLANET_RADIUS + h.max(0.));
            assert_eq!(h, surface_height(cell.direction));
            if h < 0. {
                ocean += 1;
            } else {
                land += 1;
            }
        }
        assert!(
            land > 100 && ocean > 100,
            "a world needs substantial oceans and continents"
        );
        assert!(surface_height(Vec3::ZERO).is_finite());
    }

    /// The relief is authored for a walker: summits near 150 m rather than
    /// the +432 m the 4,000 m preview carried. The sea keeps more of its
    /// range than the land, because a shelf a walker cannot submerge in is
    /// not a sea. Measured over a Fibonacci sample of the sphere so no seam
    /// or pole is favoured.
    /// The walker's eye above its feet, which is the depth that decides
    /// whether walking out to sea ever becomes swimming.
    const EYE_PLUS_FEET: f32 = 2.5;

    /// How deep the sea is a hundred metres out from the shoreline the capture
    /// presets walk, which is well inside what a standing player can see.
    fn shore_profile() -> (f32, f32) {
        let lat = 72_f32.to_radians();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 2.833 / PLANET_RADIUS;
        let mut lon = 0.0_f32;
        while surface_height(at(lon)) < 0.0 && lon < std::f32::consts::TAU {
            lon += step;
        }
        while surface_height(at(lon)) >= 0.0 && lon < 2.0 * std::f32::consts::TAU {
            lon += step;
        }
        let out = 100.0;
        (out, -surface_height(at(lon + out / PLANET_RADIUS)))
    }

    #[test]
    fn relief_is_cut_to_climbable_summits_over_a_sea_deep_enough_to_swim_in() {
        let samples = 200_000;
        let golden = std::f32::consts::PI * (3. - 5_f32.sqrt());
        let (mut peak, mut floor) = (f32::MIN, f32::MAX);
        for i in 0..samples {
            let y = 1. - 2. * (i as f32 + 0.5) / samples as f32;
            let r = (1. - y * y).max(0.).sqrt();
            let a = golden * i as f32;
            let h = surface_height(Vec3::new(r * a.cos(), y, r * a.sin()));
            peak = peak.max(h);
            floor = floor.min(h);
        }
        assert!((120.0..=180.0).contains(&peak), "summit {peak} m");
        // A shelf a walker cannot submerge in is not a sea: the water within
        // sight of a standing player has to be deeper than their eye.
        let shelf = shore_profile();
        assert!(
            shelf.1 > EYE_PLUS_FEET,
            "the sea is {:.1} m deep {:.0} m out, which a walker wades rather than swims",
            shelf.1,
            shelf.0
        );
        assert!((-110.0..=-60.0).contains(&floor), "ocean floor {floor} m");
    }
}
