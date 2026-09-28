//! Opening a world of another generator switches the planet in place
//! (`bigger-biomes` decision 8, survey B5: "Should not restart").
//!
//! The chosen generator is one value for the whole process, read by every
//! system that asks what the ground is. A unit test that switched it would
//! pull the ground out from under every other test running beside it, so
//! the switch is tested here, in a process of its own, as one test.

use bevy::math::Vec3;
use pbd_app::planet::{
    generator_version, surface_code, surface_height, switch_generator, terrain_config,
    terrain_epoch,
};
use pbd_core::planet_gen::{TerrainConfig, biome_at};
use pbd_core::terrain::GENERATOR_VERSION;

/// Enough of the sphere that version 4's biomes and version 5's disagree
/// somewhere on it: they share their heights, and move a region's biome.
fn directions() -> impl Iterator<Item = Vec3> {
    let n = 2_000;
    let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());
    (0..n).map(move |i| {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).sqrt();
        let a = golden * i as f32;
        Vec3::new(r * a.cos(), y, r * a.sin())
    })
}

/// The ground as the process reads it now, against the ground a version's
/// own config makes.
fn reads_as(version: u32) -> bool {
    let cfg = TerrainConfig::for_version(version).expect("a carried version");
    directions().all(|d| {
        let height = surface_height(d);
        height == pbd_core::column::surface_m(&cfg, d)
            && (surface_code(d, height) >> 8) == biome_at(&cfg, d, height) as u32
    })
}

#[test]
fn a_load_switches_the_generator_in_place_and_back() {
    // A launch makes the newest version's planet.
    assert_eq!(generator_version(), GENERATOR_VERSION);
    assert_eq!(terrain_epoch(), 0);
    assert!(reads_as(GENERATOR_VERSION));

    // Opening a world of the version already chosen changes nothing, and
    // asks for no rebuild.
    assert_eq!(switch_generator(GENERATOR_VERSION), Ok(false));
    assert_eq!(terrain_epoch(), 0);

    // Opening an old world makes the planet its version's, whole: the
    // config every reader asks for, the heights and the biomes, and a new
    // epoch so that work begun on the old planet is not drawn on this one.
    assert_eq!(switch_generator(4), Ok(true));
    assert_eq!(generator_version(), 4);
    assert_eq!(*terrain_config(), TerrainConfig::TENEBRIS_V4);
    assert_eq!(terrain_epoch(), 1);
    assert!(reads_as(4));
    assert!(
        !reads_as(GENERATOR_VERSION),
        "the two versions must differ somewhere on these directions, or this \
         test cannot tell which planet it is on"
    );

    // A version this build does not carry is refused, and leaves the planet
    // as it was: a save from a newer build opens on nothing it cannot make.
    assert!(switch_generator(GENERATOR_VERSION + 1).is_err());
    assert!(switch_generator(0).is_err());
    assert_eq!(generator_version(), 4);
    assert_eq!(terrain_epoch(), 1);

    // And a new world after it makes the newest planet again.
    assert_eq!(switch_generator(GENERATOR_VERSION), Ok(true));
    assert_eq!(*terrain_config(), TerrainConfig::TENEBRIS_V5);
    assert_eq!(terrain_epoch(), 2);
    assert!(reads_as(GENERATOR_VERSION));
}
