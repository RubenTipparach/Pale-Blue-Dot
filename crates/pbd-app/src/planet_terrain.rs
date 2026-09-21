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
    // The column's own rule, so a cap and the column top under it are one
    // number: `column::surface_m` floors to the layer, and the layer is the
    // elevation step.
    pbd_core::column::surface_m(&TERRAIN, d)
}
// The record's step and the column's layer are the same metre; if the step
// ever moves, `surface_height` has to quantise to it rather than to the layer.
const _: () = assert!(ELEVATION_STEP == 1.0);

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
        Material::Sand => 1,
        Material::Stone | Material::Rock | Material::Ore => 5,
        Material::Snow => 6,
        Material::JungleGrass => 3,
        Material::Water | Material::Air => 0,
        // Earth. It was drawn as the sward for as long as it shared a code
        // with grass, so a dirt layer under the turf and a cave wall cut
        // through one both came out green; and the material actually named
        // dirt shared the SAND code, so a mud flat was drawn as beach.
        Material::Soil | Material::Dirt => DIRT,
        // Grass and dry grass: the shader's own default green.
        _ => 2,
    }
}

/// The tilesets in the order `atlas.png` holds them, which is the sorted file
/// name. Both this and `tools/build_tileset_atlas.py` DERIVE the order from
/// the names rather than storing a manifest, so there is no third file to fall
/// out of step; `the_atlas_holds_the_tilesets_this_names` reads the real
/// directory and the real PNG and holds them together.
pub const TILESETS: [&str; 14] = [
    "asteroid",
    "basalt_wastes",
    "beach",
    "desert",
    "fields",
    "frozen_wastes",
    "jungle",
    "lunar_regolith",
    "mountains",
    "ocean",
    "redwood_forest",
    "sporewood",
    "swamp",
    "tundra",
];

/// Which tileset a biome is drawn from. Every sheet keeps the same layout -
/// (0,0) its own ground, (1,0) that ground fading into the earth under it,
/// (2,0) the earth, (3,0) the stone - so a biome is a slot and nothing else
/// about the drawing changes.
pub fn tileset_slot(biome: Biome) -> u32 {
    let name = match biome {
        Biome::Ocean => "ocean",
        Biome::Beach => "beach",
        Biome::Fields => "fields",
        Biome::Desert => "desert",
        Biome::Jungle => "jungle",
        Biome::Swamp => "swamp",
        Biome::Mountains => "mountains",
        Biome::Tundra => "tundra",
    };
    TILESETS
        .iter()
        .position(|&t| t == name)
        .expect("every biome names a shipped tileset") as u32
}

/// Snow is a MATERIAL rather than a biome: it caps a field above the snow line
/// and a pole at any height, so its side cannot come from the cell's own sheet
/// or a snowy meadow would fade into summer grass. It takes the tundra sheet,
/// whose (1,0) is snow over earth - Tenebris's `dirt_snow`, authored.
pub fn snow_slot() -> u32 {
    tileset_slot(Biome::Tundra)
}

/// Earth, on its own code at last. The two above it are faces rather than
/// materials - the picture a sod cell shows on its SIDE, which is that sod
/// fading into the earth under it - and nothing in a column is ever made of
/// them, so they are the shader's to derive and never a cell's material.
pub const DIRT: u32 = 10;
/// The side of a grass cell: `fields.png` tile (1,0), the shipped transition.
pub const GRASS_SIDE: u32 = 11;
/// The side of a snow cell: the same transition with its sod band taken to
/// snow, which is Tenebris's `dirt_snow` derived rather than authored. The
/// authored tile exists at (1,0) of `tundra.png` and `frozen_wastes.png` and
/// is out of reach until a body can bind its own tileset.
pub const SNOW_SIDE: u32 = 12;

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
    /// The three face codes are written here AND in `planet_surface.wgsl`,
    /// which is two copies of one fact. So this reads the REAL shader and
    /// holds it to these: a code that drifts is a wall drawn as something
    /// else, and nothing else in the build would notice.
    #[test]
    fn the_shader_agrees_about_the_face_codes() {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        for (name, code) in [
            ("DIRT_CODE", super::DIRT),
            ("GRASS_SIDE_CODE", super::GRASS_SIDE),
            ("SNOW_SIDE_CODE", super::SNOW_SIDE),
        ] {
            let line = format!("const {name} = {code}u;");
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should declare `{line}`"
            );
        }
    }

    /// The voxel light rule is written TWICE: in `pbd_core::light`, where it
    /// is tested, and in `planet_surface.wgsl`, where it actually runs -
    /// because every vertex in this renderer is generated in the shader, so
    /// there is no CPU mesh to bake a corner into. Two copies of one fact
    /// drift, so this reads the REAL shader and holds it to the core's
    /// numbers.
    ///
    /// What it proves is narrow and worth saying: that the CONSTANTS agree. It
    /// cannot prove the shader's arithmetic, and the captures are what check
    /// that. A ladder quietly changed in one file lights every crease in the
    /// world differently and nothing else in the build says a word.
    #[test]
    fn the_shader_carries_the_reference_light_constants() {
        use pbd_core::light;
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        for line in [
            format!("const LIGHT_MAX: f32 = {:.1};", light::MAX as f32),
            format!("const CONTACT_1: f32 = {:.2};", light::CONTACT[1]),
            format!("const CONTACT_2: f32 = {:.2};", light::CONTACT[2]),
            format!("const CONTACT_3: f32 = {:.2};", light::CONTACT[3]),
            format!("const LIGHT_LAYERS: u32 = {}u;", pbd_core::column::LAYERS),
            format!(
                "const LIGHT_WORDS: u32 = {}u;",
                crate::planet::column::LIGHT_WORDS
            ),
            format!(
                "const MATERIAL_WORDS: u32 = {}u;",
                crate::planet::column::MATERIAL_WORDS
            ),
            format!(
                "const MATERIAL_PER_WORD: u32 = {}u;",
                crate::planet::column::MATERIAL_PER_WORD
            ),
        ] {
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should declare `{line}`"
            );
        }
        // The step is one, and the shader relies on it: `corner_light` steps
        // up exactly one layer where it finds no air, and `sky_at` divides by
        // LIGHT_MAX with nothing else in the way.
        assert_eq!(
            light::STEP,
            1,
            "a step of anything else needs the shader to know"
        );
    }

    /// The shader's tile table, `if code==Nu { ... tile=vec2(x.,y.); }`, read
    /// off the shipped file: which tile of a sheet each material code draws.
    fn shader_tiles() -> Vec<(u32, (u32, u32))> {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        shader
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                let code = line
                    .strip_prefix("if code==")?
                    .split('u')
                    .next()?
                    .parse()
                    .ok()?;
                let tile = line.split("tile=vec2(").nth(1)?.split(')').next()?;
                let mut parts = tile.split(',').map(|p| p.trim().trim_end_matches('.'));
                Some((
                    code,
                    (parts.next()?.parse().ok()?, parts.next()?.parse().ok()?),
                ))
            })
            .collect()
    }

    /// A sheet's sixteen tile names, from `biomes.json`, in the row-major order
    /// the atlas holds them. A hand parse rather than a JSON crate: one array
    /// of strings under one id is not worth a dependency.
    fn sheet_tile_names(id: &str) -> Vec<String> {
        let json = include_str!("../../../assets/tilesets/biomes.json");
        let start = json
            .find(&format!("\"id\": \"{id}\""))
            .unwrap_or_else(|| panic!("biomes.json names the {id} sheet"));
        let tiles = &json[start..];
        let tiles = &tiles[tiles.find("\"tiles\"").expect("a tiles array")..];
        let open = tiles.find('[').expect("an array");
        let close = tiles.find(']').expect("a closed array");
        tiles[open + 1..close]
            .split(',')
            .map(|name| name.trim().trim_matches('"').to_owned())
            .collect()
    }

    /// Every material is drawn on a tile whose name in its sheet's own manifest
    /// says what it is. This is `tools/block_audit.py`'s text output as an
    /// assertion, and it is what would have caught the snow cap drawing the
    /// tundra sheet's "cold granite" and every sand cap its sheet's "packed
    /// path" - the tiles had only ever lent their brightness to a flat colour,
    /// and the day caps drew real colours they showed what they pointed at.
    #[test]
    fn the_shader_draws_each_material_on_a_tile_named_for_it() {
        let tiles = shader_tiles();
        let tile_of = |code: u32| {
            tiles
                .iter()
                .find(|(c, _)| *c == code)
                .map(|(_, tile)| *tile)
                .unwrap_or((0, 0))
        };
        let index = |(x, y): (u32, u32)| (y * 4 + x) as usize;
        for (code, sheet, word) in [
            (6, "tundra", "snow"),
            (0, "ocean", "sand"),
            (1, "beach", "sand"),
            (4, "desert", "sand"),
        ] {
            let names = sheet_tile_names(sheet);
            let name = &names[index(tile_of(code))];
            assert!(
                name.contains(word),
                "code {code} draws {sheet}'s tile #{} \"{name}\", which is not {word}",
                index(tile_of(code))
            );
        }
        // Stone is every sheet's #3 and the sward every sheet's #0.
        assert_eq!(index(tile_of(5)), 3, "stone is the sheet's stone");
        assert_eq!(index(tile_of(2)), 0, "grass is the sheet's ground");
        assert_eq!(&sheet_tile_names("fields")[3], "limestone");
        assert_eq!(&sheet_tile_names("fields")[0], "pasture grass");
    }

    /// The atlas is baked by `tools/build_tileset_atlas.py`, which derives its
    /// slot order by sorting the tileset file names. This names the same order
    /// and would be a silent lie if a tileset were added, so it reads the real
    /// directory and the real PNG's header.
    #[test]
    fn the_atlas_holds_the_tilesets_this_names() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/tilesets");
        let mut found: Vec<String> = std::fs::read_dir(&dir)
            .expect("the tilesets ship with the repository")
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let stem = path.file_stem()?.to_str()?.to_owned();
                (path.extension()? == "png" && stem != "atlas").then_some(stem)
            })
            .collect();
        found.sort();
        assert_eq!(found, super::TILESETS, "the slot order is the sorted name");

        let atlas = std::fs::read(dir.join("atlas.png")).expect("the atlas is committed");
        let width = u32::from_be_bytes(atlas[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(atlas[20..24].try_into().unwrap());
        // Four sheets across and down, four tiles each, 32 texels a tile:
        // the grid `pixel_tile` indexes.
        assert_eq!(
            (width, height),
            (512, 512),
            "atlas.png is 4x4 sheets of 4x4 tiles"
        );
    }

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
