//! The detailed trees' kit on the GPU (`flora-v2` design 10.2): the near
//! models, the anchors, the kit's own tiles and the baked impostors,
//! built once by `pbd_core::tree_kit` behind the loading screen and pulled
//! by `planet_surface.wgsl` from one read-only storage buffer.

use std::sync::LazyLock;
use std::time::Instant;

use bevy::log::info;
use pbd_core::tree_kit::{self, Kit, Textures};

/// Near-tree list entries per species (design 10.5): the most finest cells
/// inside `tree_near_m`'s 60 m ceiling, twice over for a landing's
/// dissolve, is about 3,300. `NEAR_CAP` in both shaders.
pub const NEAR_CAP: u32 = 4096;
/// The near draws' first vertex: `NEAR_TREE_FIRST_VERTEX` in the surface
/// shader, the species in the bits over 16.
pub const NEAR_FIRST_VERTEX: u32 = 1 << 20;
/// Vertices a near tree carries for its fruit after its model: four
/// hanging, two crossed cards each, and two fallen.
pub const FRUIT_VERTICES: u32 = 4 * 12 + 2 * 6;

fn assets() -> std::path::PathBuf {
    crate::config::config_dir().join("..")
}

fn read_textures() -> Textures {
    let atlas = image::open(assets().join("tilesets/atlas.png"))
        .expect("assets/tilesets/atlas.png decodes")
        .to_rgb8();
    let size = atlas.width();
    let tiles = tree_kit::KIT_TILES
        .iter()
        .map(|path| {
            let tile = image::open(assets().join(path))
                .unwrap_or_else(|e| panic!("assets/{path}: {e}"))
                .to_rgba8();
            assert_eq!(
                tile.dimensions(),
                (32, 32),
                "assets/{path} is a 32 x 32 tile"
            );
            tile.into_raw()
        })
        .collect();
    Textures {
        atlas: atlas.into_raw(),
        atlas_size: size,
        tiles,
    }
}

static KIT: LazyLock<Kit> = LazyLock::new(|| {
    let start = Instant::now();
    let kit = tree_kit::build_kit(&read_textures());
    info!(
        "tree kit: {} models, {} KiB, built in {:.0} ms; near vertices per species {:?}",
        kit.models.len(),
        kit.words.len() * 4 / 1024,
        start.elapsed().as_secs_f64() * 1000.0,
        tree_kit::SPECIES
            .iter()
            .zip(kit.vertex_max)
            .map(|(s, n)| format!("{} {}", s.name(), n + FRUIT_VERTICES))
            .collect::<Vec<_>>()
    );
    kit
});

/// Start building the kit on a thread of its own, so it is ready by the
/// time the planet's buffers are made (the loading screen covers it).
pub fn prepare() {
    std::thread::Builder::new()
        .name("tree kit".into())
        .spawn(|| {
            LazyLock::force(&KIT);
        })
        .expect("the tree kit thread");
}

/// The built kit, waiting for its thread if it is not done.
pub fn kit() -> &'static Kit {
    &KIT
}

/// The near draws' vertex counts, one per species: its largest variant and
/// the fruit.
pub fn near_vertices() -> [u32; 8] {
    kit().vertex_max.map(|n| n + FRUIT_VERTICES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::tree_kit::{BIOME_TREES, SPECIES, Species};

    const SURFACE: &str = include_str!("../../../assets/shaders/planet_surface.wgsl");
    const VISIBILITY: &str = include_str!("../../../assets/shaders/planet_visibility.wgsl");

    fn constant(source: &str, name: &str) -> u32 {
        let line = source
            .lines()
            .find(|l| l.starts_with(&format!("const {name}: u32 = ")))
            .unwrap_or_else(|| panic!("no const {name}"));
        line.split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_end_matches(';')
            .trim_end_matches('u')
            .parse()
            .unwrap()
    }

    #[test]
    fn the_shaders_read_the_kit_as_it_is_packed() {
        assert_eq!(constant(SURFACE, "NEAR_CAP"), NEAR_CAP);
        assert_eq!(constant(VISIBILITY, "NEAR_CAP"), NEAR_CAP);
        assert_eq!(
            constant(SURFACE, "NEAR_TREE_FIRST_VERTEX"),
            NEAR_FIRST_VERTEX
        );
        assert_eq!(
            constant(SURFACE, "KIT_SPECIES_WORDS"),
            tree_kit::SPECIES_WORDS
        );
        assert_eq!(
            constant(SURFACE, "KIT_VARIANT_BASE"),
            tree_kit::VARIANT_BASE
        );
        assert_eq!(
            constant(SURFACE, "KIT_VARIANT_WORDS"),
            tree_kit::VARIANT_WORDS
        );
        assert_eq!(constant(SURFACE, "KIT_TILE_BASE"), tree_kit::TILE_BASE);
        assert_eq!(
            constant(SURFACE, "KIT_VERTEX_WORDS"),
            tree_kit::VERTEX_WORDS
        );
        assert_eq!(constant(SURFACE, "TREE_VARIANTS"), tree_kit::VARIANTS);
        assert_eq!(
            constant(SURFACE, "IMPOSTOR_VIEW_W"),
            pbd_core::tree_bake::VIEW_W as u32
        );
        assert_eq!(
            constant(SURFACE, "IMPOSTOR_H"),
            pbd_core::tree_bake::VIEW_H as u32
        );
        assert_eq!(
            constant(SURFACE, "IMPOSTOR_W"),
            pbd_core::tree_bake::STRIP_W as u32
        );
        for (name, salt) in [
            ("SALT_SPECIES", tree_kit::SALT_SPECIES),
            ("SALT_VARIANT", tree_kit::SALT_VARIANT),
            ("SALT_TURN", tree_kit::SALT_TURN),
            ("SALT_SIZE", tree_kit::SALT_SIZE),
        ] {
            assert!(
                SURFACE.contains(&format!("const {name}: u32 = 0x{salt:x}u;")),
                "{name}"
            );
        }
        assert!(VISIBILITY.contains(&format!(
            "const SALT_CACTUS: u32 = 0x{:x}u;",
            tree_kit::SALT_CACTUS
        )));
        assert_eq!(
            constant(VISIBILITY, "CACTUS_DENSITY"),
            tree_kit::CACTUS_DENSITY
        );
    }

    /// Both shaders pick a tree's species with the same lines, and those
    /// lines are `BIOME_TREES` (a biome of one species returns it; of two,
    /// the first below its share).
    #[test]
    fn the_species_pick_is_the_cores_mix() {
        let index = |s: Species| SPECIES.iter().position(|x| *x == s).unwrap();
        for (biome, mix) in BIOME_TREES.iter().enumerate() {
            let line = match mix {
                [] => continue,
                [(one, _)] => format!("if biome == {biome}u {{ return {}u; }}", index(*one)),
                [(a, w), (b, _)] => format!(
                    "if biome == {biome}u {{ return select({}u, {}u, r < {w}); }}",
                    index(*b),
                    index(*a)
                ),
                _ => panic!("the shaders pick from at most two species"),
            };
            assert!(SURFACE.contains(&line), "surface: {line}");
            assert!(VISIBILITY.contains(&line), "visibility: {line}");
        }
        for id in 0..2000 {
            for biome in [1, 2, 3, 4, 5, 7] {
                let p = tree_kit::pick(id, biome).unwrap();
                let r = tree_kit::roll(id, tree_kit::SALT_SPECIES);
                let mix = BIOME_TREES[biome as usize];
                let expect = if mix.len() == 1 || r < mix[0].1 {
                    mix[0].0
                } else {
                    mix[1].0
                };
                assert_eq!(p.species, expect);
            }
        }
    }
}
