//! The sea's cells (`ocean-environments` design B24): reef rock on a reef,
//! basalt and mussel beds at a vent, and sea ice standing on polar water, as
//! cells a player digs, made by generator version 14 and later.
//!
//! Where each place is comes from [`super::places`], the one function the
//! pieces, the spawner and the map read; what is here is which cell a place
//! is made of. Every rule reads a smooth field (a place, a patch noise, the
//! depth), never a neighbour's cell, so a patch is a place with edges.
//! Derived: nothing here is saved.

use crate::fauna::Environment;
use crate::planet_gen::{self, TerrainConfig};
use crate::terrain::Material;
use glam::Vec3;

/// The rules, with their numbers. Part of a generator version: never edited
/// once a world is made with them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeaCells {
    /// Reef rock where the reef's patch field (`reef_patch_m` across) is
    /// over `reef_above`; the reef's sand elsewhere.
    pub reef_patch_m: f32,
    pub reef_above: f32,
    /// A vent's ground is basalt, with mussel beds where its patch field
    /// (`mussel_patch_m` across) is over `mussel_above`.
    pub mussel_patch_m: f32,
    pub mussel_above: f32,
    /// Sea ice on polar water where the floe field (`floe_patch_m` across)
    /// is over `floe_above`, over water at least `floe_depth_m` deep, so a
    /// floe never stands on a beach.
    pub floe_patch_m: f32,
    pub floe_above: f32,
    pub floe_depth_m: f32,
}

impl SeaCells {
    /// Version 14's (design B24).
    pub const V14: Self = Self {
        reef_patch_m: 7.0,
        reef_above: 0.5,
        mussel_patch_m: 6.0,
        mussel_above: 0.56,
        floe_patch_m: 9.0,
        floe_above: 0.6,
        floe_depth_m: 2.0,
    };

    /// Version 16's (design B30): version 14's, with the reef's rock in
    /// wider patches (10 m) over more of the reef (over 0.45 of its field,
    /// about two thirds of a reef's floor where 0.5 gave about a third).
    pub const V16: Self = Self {
        reef_patch_m: 10.0,
        reef_above: 0.45,
        ..Self::V14
    };
}

const REEF_SALT: u64 = 0x2EEF_0014;
const MUSSEL_SALT: u64 = 0x3055_0014;
const FLOE_SALT: u64 = 0xF10E_0014;

fn patch(cfg: &TerrainConfig, salt: u64, metres: f32, d: Vec3) -> f32 {
    planet_gen::noise01(cfg.seed ^ salt, d, cfg.scale_of(metres), 2)
}

/// The sea's place at a floored surface, as the cells read it: the map's
/// (without the two-cell coast, which no cell here depends on).
fn place(cfg: &TerrainConfig, d: Vec3, surface_m: f32) -> Option<Environment> {
    super::places::chart_at(cfg, d, surface_m)
}

/// The top cell of a column under the sea, where this version makes it one
/// of the sea's cells; `None` where the top is the generator's own (sand or
/// stone), and always on a world without sea cells.
pub fn top(cfg: &TerrainConfig, d: Vec3, surface_m: f32) -> Option<Material> {
    let cells = cfg.sea_cells.as_ref()?;
    // The beach's band stays sand, as the generator has it.
    if surface_m >= cfg.sea_level_m - cfg.beach_band_m {
        return None;
    }
    match place(cfg, d, surface_m)? {
        Environment::Reef => (patch(cfg, REEF_SALT, cells.reef_patch_m, d) > cells.reef_above)
            .then_some(Material::ReefRock),
        Environment::Vent => Some(
            if patch(cfg, MUSSEL_SALT, cells.mussel_patch_m, d) > cells.mussel_above {
                Material::MusselBed
            } else {
                Material::Basalt
            },
        ),
        _ => None,
    }
}

/// Whether a cell of sea ice stands on the sea over this column: its layer
/// is the one from the sea's level up a metre. Polar water at least
/// `floe_depth_m` deep, where the floe field is high.
pub fn sea_ice(cfg: &TerrainConfig, d: Vec3, surface_m: f32) -> bool {
    let Some(cells) = cfg.sea_cells.as_ref() else {
        return false;
    };
    if cfg.sea_level_m - surface_m < cells.floe_depth_m {
        return false;
    }
    if patch(cfg, FLOE_SALT, cells.floe_patch_m, d) <= cells.floe_above {
        return false;
    }
    place(cfg, d, surface_m) == Some(Environment::UnderIce)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(n: usize) -> impl Iterator<Item = Vec3> {
        (0..n).map(move |i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
            let r = (1.0 - y * y).sqrt();
            let phi = i as f32 * 2.399_963;
            Vec3::new(r * phi.cos(), y, r * phi.sin())
        })
    }

    /// Version 14 makes each of the sea's cells somewhere, only in its own
    /// place, and version 12 makes none of them.
    #[test]
    fn each_sea_cell_stands_only_in_its_own_place() {
        let v14 = TerrainConfig::TENEBRIS_V14;
        let v12 = TerrainConfig::TENEBRIS_V12;
        let (mut reef, mut basalt, mut mussel, mut ice) = (0, 0, 0, 0);
        let (mut reef_place, mut vent_place, mut polar_place) = (0, 0, 0);
        for d in sphere(400_000) {
            let surface = planet_gen::surface_altitude(&v14, d).floor();
            if surface >= v14.sea_level_m {
                continue;
            }
            let here = place(&v14, d, surface);
            match here {
                Some(Environment::Reef) => reef_place += 1,
                Some(Environment::Vent) => vent_place += 1,
                Some(Environment::UnderIce) => polar_place += 1,
                _ => {}
            }
            match top(&v14, d, surface) {
                Some(Material::ReefRock) => {
                    assert_eq!(here, Some(Environment::Reef));
                    reef += 1;
                }
                Some(Material::Basalt) => {
                    assert_eq!(here, Some(Environment::Vent));
                    basalt += 1;
                }
                Some(Material::MusselBed) => {
                    assert_eq!(here, Some(Environment::Vent));
                    mussel += 1;
                }
                Some(other) => panic!("{other:?} at {d}"),
                None => {}
            }
            if sea_ice(&v14, d, surface) {
                assert_eq!(here, Some(Environment::UnderIce));
                assert!(surface <= v14.sea_level_m - 2.0, "a floe over the shallows");
                ice += 1;
            }
            assert_eq!(top(&v12, d, surface), None);
            assert!(!sea_ice(&v12, d, surface));
        }
        eprintln!(
            "reef rock {reef} of {reef_place} reef, basalt {basalt} and mussel {mussel} \
             of {vent_place} vent, ice {ice} of {polar_place} polar"
        );
        assert!(reef > 0 && reef < reef_place, "reef rock in patches");
        assert_eq!(basalt + mussel, vent_place, "a vent is basalt or mussels");
        assert!(mussel > 0 && mussel * 2 < vent_place, "mussels in patches");
        assert!(ice > 0 && ice < polar_place, "floes, not a sheet");
    }
}
