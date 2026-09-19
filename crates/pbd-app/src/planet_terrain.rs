//! Stable generation inputs shared by the flight clearance policy and GPU
//! upload. The GPU shades derived geometry; it never decides terrain heights.

use bevy::prelude::*;
use pbd_core::planet_gen::{self, Biome, TerrainConfig};
use pbd_core::terrain::Material;

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
pub const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;

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

/// What the record hands the shaders about a cell's surface: the material
/// index in the low byte and the biome in the next one. The shader's table is
/// the authority for what each material draws (0 seabed, 1 beach, 2 pasture,
/// 3 jungle, 4 desert, 5 stone, 6 snow, 7 marsh); the biome is what the
/// foliage pass reads for its density and the tree for its height, which is
/// how the reference keys both. Two facts in one word because they are
/// written and read together and a cell has exactly one of each.
/// The generator's river carve at a direction, for anything that needs to
/// find a watercourse rather than infer one from a height.
pub fn river_channel(cfg: &TerrainConfig, direction: Vec3) -> f32 {
    planet_gen::river_channel(cfg, direction.normalize_or(Vec3::Y))
}

pub fn surface_code(direction: Vec3, height: f32) -> u32 {
    let d = direction.normalize_or(Vec3::Y);
    let biome = planet_gen::biome_at(&TERRAIN, d, height);
    material_index(d, height, biome) | (biome as u32) << 8
}

/// Which of `planet_surface.wgsl`'s material codes a material is drawn in.
///
/// ONE table, because a cave wall of stone and a mountain top of stone are the
/// same rock and two tables would eventually disagree about it. What the surface
/// adds on top of this is the handful of decisions that need a BIOME or a sea
/// level, which a face underground has neither of.
pub fn render_code(material: Material) -> u32 {
    match material {
        Material::Sand | Material::Dirt => 1,
        Material::Stone | Material::Rock | Material::Ore => 5,
        Material::Snow => 6,
        Material::JungleGrass => 3,
        Material::Water | Material::Air => 0,
        // Soil, grass and dry grass: the shader's own default green.
        _ => 2,
    }
}

fn material_index(direction: Vec3, height: f32, biome: Biome) -> u32 {
    let material = planet_gen::top_material(&TERRAIN, direction, height);
    match (material, biome) {
        // Beach sand below the waterline is the seabed, which the water pass
        // tints; a desert dune and a swamp sward are their own tiles.
        (Material::Sand, _) if height < TERRAIN.sea_level_m => 0,
        (Material::Sand, Biome::Desert) => 4,
        (Material::Grass, Biome::Swamp) => 7,
        _ => render_code(material),
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
        // The deepest water within sight of the shore, rather than the depth
        // at one fixed distance: a coastline with real structure has inlets
        // and islands, so a single probe 100 m out can land back on dry
        // ground. What the swim needs is that water deep enough to submerge
        // in is reachable from the beach, which is what this measures.
        let mut deepest = (0.0_f32, 0.0_f32);
        let mut out = 0.0_f32;
        while out < 400.0 {
            out += 2.833;
            let depth = -surface_height(at(lon + out / PLANET_RADIUS));
            if depth > deepest.1 {
                deepest = (out, depth);
            }
        }
        deepest
    }

    /// What a capture preset is actually standing in, which is what decides
    /// how dense its trees are: the density table is per biome, so a frame
    /// that reads as closed canopy is a frame in a jungle rather than a
    /// scatter rule gone wrong. Run with `--ignored --nocapture`.
    #[test]
    #[ignore]
    fn biome_under_each_capture_preset() {
        for (name, direction) in [
            ("surface", Vec3::new(0.8776, 0.4794, 0.0)),
            ("coast", Vec3::new(0.8776, 0.4794, 0.0)),
            (
                "spawn",
                crate::flight_view::FlightViewConfig::default().spawn_direction,
            ),
        ] {
            let d = direction.normalize();
            let h = surface_height(d);
            let code = surface_code(d, h);
            println!(
                "{name:8} height {h:6.1} m  material {}  biome {:?}",
                code & 0xff,
                planet_gen::biome_at(&TERRAIN, d, h)
            );
        }
    }

    /// The generator is authored for THIS body: its land-scale fields derive
    /// their frequency from the radius it carries, so a config authored for a
    /// different one would put its hills at the wrong size. Two places hold
    /// the radius, so a test holds them together.
    #[test]
    fn the_generator_is_authored_for_this_bodys_radius() {
        assert_eq!(TERRAIN.radius_m, PLANET_RADIUS);
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
        // The floor band widened from -110 when the land bias landed. The bias
        // shifts the whole continent field down to break the supercontinent, so
        // the deepest basin goes down with it: measured -126 m against -110
        // before. That is the choice's own consequence rather than a drift, and
        // what this assertion is for is unchanged - a sea a walker can swim in
        // and a floor that is not absurd under a 150 m summit.
        assert!((-145.0..=-60.0).contains(&floor), "ocean floor {floor} m");
    }
}

#[cfg(test)]
mod landmass_report {
    use super::*;

    /// Group the land into connected masses on a level-`n` dual sphere, largest
    /// first, as a share of all land. Shared by the pinning test and the two
    /// reports so there is one flood fill rather than three.
    fn land_masses(level: u32) -> (f32, Vec<f32>) {
        let cells = super::super::topology::dual_sphere(level);
        let sea = TERRAIN.sea_level_m;
        let land: Vec<bool> = cells
            .iter()
            .map(|c| surface_height(c.direction) >= sea)
            .collect();
        let mut seen = vec![false; cells.len()];
        let mut sizes = Vec::new();
        for start in 0..cells.len() {
            if !land[start] || seen[start] {
                continue;
            }
            let mut stack = vec![start];
            seen[start] = true;
            let mut size = 0usize;
            while let Some(index) = stack.pop() {
                size += 1;
                for &next in &cells[index].neighbors {
                    if land[next] && !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
            sizes.push(size);
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let land_total: usize = sizes.iter().sum();
        let shares = sizes
            .iter()
            .map(|s| *s as f32 / land_total.max(1) as f32)
            .collect();
        (land_total as f32 / cells.len() as f32, shares)
    }

    /// The land is several continents and a tail of islands, not one mass.
    ///
    /// Both halves are pinned because both causes are real: the continent
    /// frequency cuts the field, and the land FRACTION decides whether the
    /// pieces touch. Above about 45% land the sphere percolates and joins up
    /// however finely it is cut - measured, at `continent_scale` 2.4 with no
    /// land bias the largest mass is still 90.8% of the land. So a later
    /// tuning that raises the land back over the band restores the
    /// supercontinent without touching the frequency at all, and this fails.
    #[test]
    fn the_land_is_several_continents_and_a_tail_of_islands() {
        let (land, shares) = land_masses(5);
        assert!(
            (0.30..0.45).contains(&land),
            "land fraction {land:.3} is outside the band that keeps the masses apart"
        );
        assert!(
            shares[0] < 0.55,
            "the largest mass holds {:.1}% of the land: that is a supercontinent",
            100.0 * shares[0]
        );
        // Several masses worth calling continents, rather than one and gravel.
        let continents = shares.iter().filter(|s| **s > 0.05).count();
        assert!(
            (3..=8).contains(&continents),
            "{continents} masses hold more than a twentieth of the land"
        );
        // And a real tail behind them: islands, not just two or three lumps.
        let islands = shares.iter().filter(|s| **s < 0.01).count();
        assert!(islands >= 10, "only {islands} islands");
    }

    /// Sweep the continent field and report what each setting makes, so the
    /// candidates worth rendering are chosen off numbers rather than guesses.
    #[test]
    #[ignore = "a report: cargo test -p pbd-app --lib continent_sweep -- --ignored --nocapture"]
    fn continent_sweep() {
        let cells = super::super::topology::dual_sphere(6);
        println!("scale  bias  land%  masses  biggest%  top5 share of land");
        for scale in [0.8f32, 1.6, 2.4, 3.2, 4.0, 5.0] {
            for bias in [0.0f32, -0.05, -0.10] {
                let cfg = TerrainConfig {
                    continent_scale: scale,
                    land_bias: bias,
                    ..TERRAIN
                };
                let sea = cfg.sea_level_m;
                let land: Vec<bool> = cells
                    .iter()
                    .map(|c| pbd_core::planet_gen::surface_altitude(&cfg, c.direction) >= sea)
                    .collect();
                let mut seen = vec![false; cells.len()];
                let mut sizes = Vec::new();
                for start in 0..cells.len() {
                    if !land[start] || seen[start] {
                        continue;
                    }
                    let mut stack = vec![start];
                    seen[start] = true;
                    let mut size = 0usize;
                    while let Some(index) = stack.pop() {
                        size += 1;
                        for &next in &cells[index].neighbors {
                            if land[next] && !seen[next] {
                                seen[next] = true;
                                stack.push(next);
                            }
                        }
                    }
                    sizes.push(size);
                }
                sizes.sort_unstable_by(|a, b| b.cmp(a));
                let land_total: usize = sizes.iter().sum::<usize>().max(1);
                let top5: usize = sizes.iter().take(5).sum();
                println!(
                    "{scale:>5.1} {bias:>5.2} {:>6.1} {:>7} {:>9.1} {:>8.1}",
                    100.0 * land_total as f32 / cells.len() as f32,
                    sizes.len(),
                    100.0 * sizes.first().copied().unwrap_or(0) as f32 / land_total as f32,
                    100.0 * top5 as f32 / land_total as f32,
                );
            }
        }
    }

    /// Flood-fill the land on a level-`n` dual sphere and report the connected
    /// components by share of the whole body. A measurement instrument: the
    /// question "is this one continent or several" is not answerable by looking
    /// at an orbit capture, because a land bridge one cell wide joins two
    /// masses that read as separate.
    #[test]
    #[ignore = "a report: cargo test -p pbd-app --lib landmass_report -- --ignored --nocapture"]
    fn landmass_report() {
        let cells = super::super::topology::dual_sphere(6);
        let sea = TERRAIN.sea_level_m;
        let land: Vec<bool> = cells
            .iter()
            .map(|c| surface_height(c.direction) >= sea)
            .collect();
        let mut seen = vec![false; cells.len()];
        let mut sizes = Vec::new();
        for start in 0..cells.len() {
            if !land[start] || seen[start] {
                continue;
            }
            let mut stack = vec![start];
            seen[start] = true;
            let mut size = 0usize;
            while let Some(index) = stack.pop() {
                size += 1;
                for &next in &cells[index].neighbors {
                    if land[next] && !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
            sizes.push(size);
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let total = cells.len() as f32;
        let land_total: usize = sizes.iter().sum();
        println!(
            "{} cells, land {:.1}% in {} masses",
            cells.len(),
            100.0 * land_total as f32 / total,
            sizes.len()
        );
        for (rank, size) in sizes.iter().take(12).enumerate() {
            println!(
                "  #{:<2} {:>7} cells  {:>5.2}% of the body  {:>5.1}% of the land",
                rank + 1,
                size,
                100.0 * *size as f32 / total,
                100.0 * *size as f32 / land_total as f32
            );
        }
        let islands = sizes.iter().filter(|s| **s < 40).count();
        println!("  masses under 40 cells (islands): {islands}");
    }
}
