//! Where each water place is (`ocean-environments` design 4 and B10): one
//! function, [`environment`], read by the generator (the trench and the
//! pieces), the spawner (a species' `env` tags) and the map, so the three can
//! never disagree.
//!
//! It reads the ground (`planet_gen`), the plate field ([`super::plates`])
//! and the sea's yearly normals: the coldest and warmest daily-mean sea
//! temperature and the share of the year frozen, per atmosphere cell,
//! baked once by `examples/sea_normals.rs` and frozen as generator version
//! 7's input (`assets/climate/sea-normals-g7.bin`, compiled in).
//!
//! A world whose generator has no ocean places (versions 4 to 6) has only
//! the coast and the open sea, as it always had.

use super::plates;
use crate::atmosphere::grid::Grid;
use crate::fauna::Environment;
use crate::planet_gen::{self, TerrainConfig};
use glam::Vec3;
use std::sync::OnceLock;

/// The rules, with their numbers (design B10: decision 4's rules on this
/// planet's scale). Part of a generator version: never edited once a world
/// is made with them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OceanPlaces {
    /// Which frozen normals table: the generator version it was baked for.
    pub normals: u32,
    /// Reef: no deeper than this, m, where the coldest sea is above
    /// `reef_coldest_c`, and no river channel within `reef_clear_m`.
    pub reef_depth_m: f32,
    pub reef_coldest_c: f32,
    pub reef_clear_m: f32,
    /// Kelp: between these depths, m, on rock (`rockiness` above
    /// `kelp_rock`), where the warmest sea lies in `kelp_warmest_c`.
    pub kelp_depth_m: (f32, f32),
    pub kelp_rock: f32,
    pub kelp_warmest_c: (f32, f32),
    /// Polar: frozen on at least this share of the year's days.
    pub polar_frozen: f32,
    /// Vents: deeper than this, m, within `vent_reach_m` of a ridge.
    pub vent_depth_m: f32,
    pub vent_reach_m: f32,
    /// Trenches: deeper than this, m, within `trench_reach_m` of a trench
    /// line, where the floor is carved down toward `trench_floor_m`.
    pub trench_depth_m: f32,
    pub trench_reach_m: f32,
    pub trench_floor_m: f32,
    /// How fast two plates must part or close for a ridge or a trench.
    pub boundary_rate: f32,
    /// The coast: within this many cells of the shore.
    pub coast_cells: f32,
}

impl OceanPlaces {
    /// Version 7's rules.
    pub const V7: Self = Self {
        normals: 7,
        reef_depth_m: 6.0,
        // Decision 4 says 23 C; the settled planet's warmest coldest-day sea
        // is 21.9 C (sea_normals, measured), so 23 C would make no reef at
        // all. Real reefs live down to about 18 C; 20 C keeps them to the
        // warmest tropics (recommendation taken, ask only with screenshots).
        reef_coldest_c: 20.0,
        reef_clear_m: 60.0,
        kelp_depth_m: (6.0, 40.0),
        kelp_rock: 0.5,
        kelp_warmest_c: (5.0, 20.0),
        polar_frozen: 0.25,
        vent_depth_m: 90.0,
        vent_reach_m: 120.0,
        trench_depth_m: 60.0,
        trench_reach_m: 150.0,
        trench_floor_m: -140.0,
        boundary_rate: 0.2,
        coast_cells: 2.0,
    };
}

/// The sea's yearly normals, per atmosphere cell.
pub struct SeaNormals {
    pub grid: Grid,
    pub coldest_c: Vec<f32>,
    pub warmest_c: Vec<f32>,
    pub frozen: Vec<f32>,
}

const G7: &[u8] = include_bytes!("../../../../assets/climate/sea-normals-g7.bin");
const MAGIC: &[u8; 8] = b"PBDSEAN1";

impl SeaNormals {
    /// Read a normals file; `radius_m` is the body's, which the grid is
    /// built on.
    pub fn parse(bytes: &[u8], radius_m: f32) -> Result<Self, String> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err("not a sea normals file".into());
        }
        let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let (level, cells) = (word(8), word(12) as usize);
        if bytes.len() != 16 + cells * 12 {
            return Err(format!("{} bytes for {cells} cells", bytes.len()));
        }
        let grid = Grid::new(level, radius_m);
        if grid.len() != cells {
            return Err(format!(
                "level {level} has {} cells, not {cells}",
                grid.len()
            ));
        }
        let plane = |p: usize| -> Vec<f32> {
            (0..cells)
                .map(|i| {
                    let at = 16 + (p * cells + i) * 4;
                    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
                })
                .collect()
        };
        let normals = Self {
            grid,
            coldest_c: plane(0),
            warmest_c: plane(1),
            frozen: plane(2),
        };
        if normals
            .coldest_c
            .iter()
            .chain(&normals.warmest_c)
            .chain(&normals.frozen)
            .any(|v| !v.is_finite())
        {
            return Err("a value is not finite".into());
        }
        Ok(normals)
    }

    /// The frozen table a generator's ocean places name.
    pub fn of(places: &OceanPlaces, radius_m: f32) -> &'static SeaNormals {
        static G7_NORMALS: OnceLock<SeaNormals> = OnceLock::new();
        assert_eq!(
            places.normals, 7,
            "the only normals shipped are version 7's"
        );
        G7_NORMALS.get_or_init(|| SeaNormals::parse(G7, radius_m).expect("the shipped normals"))
    }

    /// The three normals at a direction, interpolated: coldest and warmest
    /// daily-mean sea temperature, C, and the share of the year frozen.
    pub fn at(&self, direction: Vec3) -> (f32, f32, f32) {
        let (cells, w) = self.grid.locate(direction);
        let f = |field: &[f32]| crate::atmosphere::grid::weighted(field, cells, w);
        (f(&self.coldest_c), f(&self.warmest_c), f(&self.frozen))
    }
}

/// How close a ridge (parting) or a trench (closing) line is, m, at a
/// direction: `None` where the nearest boundary is neither.
fn boundary(cfg: &TerrainConfig, places: &OceanPlaces, d: Vec3) -> (Option<f32>, Option<f32>) {
    let b = plates::at_with(&plates::plates(cfg.seed), cfg.seed, d);
    let metres = b.arc * cfg.radius_m;
    if b.closing > places.boundary_rate {
        (None, Some(metres))
    } else if b.closing < -places.boundary_rate {
        (Some(metres), None)
    } else {
        (None, None)
    }
}

/// The trench's carve, for the generator: the floor deepened toward
/// `trench_floor_m` along a trench line, eased out across `trench_reach_m`.
/// Called on sea floor deeper than `trench_depth_m` only.
pub fn carve_trench(cfg: &TerrainConfig, places: &OceanPlaces, d: Vec3, base: f32) -> f32 {
    if base >= cfg.sea_level_m - places.trench_depth_m {
        return base;
    }
    match boundary(cfg, places, d) {
        (_, Some(m)) if m < places.trench_reach_m => {
            let t = 1.0 - m / places.trench_reach_m;
            let t = t * t * (3.0 - 2.0 * t);
            base + (base.min(places.trench_floor_m) - base) * t
        }
        _ => base,
    }
}

/// Whether there is land within `metres` of a direction (six bearings).
fn land_within(cfg: &TerrainConfig, d: Vec3, metres: f32) -> bool {
    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
    let north = d.cross(east);
    let arc = metres / cfg.radius_m;
    (0..6).any(|k| {
        let a = k as f32 * std::f32::consts::TAU / 6.0;
        let at = (d + (east * a.cos() + north * a.sin()) * arc).normalize_or(d);
        planet_gen::surface_altitude(cfg, at) >= cfg.sea_level_m
    })
}

/// Whether a river runs into the sea within `metres` of a direction: a
/// point the channel carves under the sea that would be land without it.
fn river_within(cfg: &TerrainConfig, d: Vec3, metres: f32) -> bool {
    let mut uncarved = *cfg;
    // The channel field never exceeds one, so no carve fires.
    uncarved.river_threshold = 2.0;
    uncarved.ocean = None;
    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
    let north = d.cross(east);
    let arc = metres / cfg.radius_m;
    std::iter::once(d)
        .chain((0..6).map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 6.0;
            (d + (east * a.cos() + north * a.sin()) * arc).normalize_or(d)
        }))
        .any(|at| {
            planet_gen::river_channel(cfg, at) > cfg.river_threshold
                && planet_gen::surface_altitude(&uncarved, at) >= cfg.sea_level_m
        })
}

/// The cell width, m, every body shares (CLAUDE.md, "Hex size is fixed").
const CELL_M: f32 = 2.833;

/// The water place at a direction, or `None` on land.
pub fn environment(cfg: &TerrainConfig, direction: Vec3) -> Option<Environment> {
    let d = direction.normalize_or(Vec3::Y);
    let altitude = planet_gen::surface_altitude(cfg, d);
    environment_at(cfg, d, altitude)
}

/// [`environment`] with the altitude already in hand.
pub fn environment_at(cfg: &TerrainConfig, d: Vec3, altitude: f32) -> Option<Environment> {
    place_at(cfg, d, altitude, true)
}

/// The places as the map's chart shows them: [`environment_at`] without the
/// coast, which is two cells wide and finer than any map pixel (the chart
/// shows open sea there), and without its six extra looks at the ground.
/// `None` on land and on a world made before the sea's places.
pub fn chart_at(cfg: &TerrainConfig, d: Vec3, altitude: f32) -> Option<Environment> {
    cfg.ocean.as_ref()?;
    place_at(cfg, d, altitude, false)
}

fn place_at(cfg: &TerrainConfig, d: Vec3, altitude: f32, with_coast: bool) -> Option<Environment> {
    let sea = cfg.sea_level_m;
    if altitude >= sea {
        return None;
    }
    let depth = sea - altitude;
    let coast = |places: Option<&OceanPlaces>| {
        let cells = places.map_or(2.0, |p| p.coast_cells);
        with_coast && land_within(cfg, d, cells * CELL_M)
    };
    let Some(places) = cfg.ocean.as_ref() else {
        return Some(if coast(None) {
            Environment::Coast
        } else {
            Environment::Open
        });
    };
    let normals = SeaNormals::of(places, cfg.radius_m);
    let (coldest, warmest, frozen) = normals.at(d);
    if frozen >= places.polar_frozen {
        return Some(Environment::UnderIce);
    }
    if depth > places.vent_depth_m.min(places.trench_depth_m) {
        let (ridge, trench) = boundary(cfg, places, d);
        if depth > places.vent_depth_m && ridge.is_some_and(|m| m < places.vent_reach_m) {
            return Some(Environment::Vent);
        }
        if trench.is_some_and(|m| m < places.trench_reach_m) {
            return Some(Environment::Trench);
        }
    }
    if depth <= places.reef_depth_m
        && coldest >= places.reef_coldest_c
        && !river_within(cfg, d, places.reef_clear_m)
    {
        return Some(Environment::Reef);
    }
    if depth >= places.kelp_depth_m.0
        && depth <= places.kelp_depth_m.1
        && planet_gen::rockiness(cfg, d) > places.kelp_rock
        && warmest >= places.kelp_warmest_c.0
        && warmest <= places.kelp_warmest_c.1
    {
        return Some(Environment::Kelp);
    }
    Some(if coast(Some(places)) {
        Environment::Coast
    } else {
        Environment::Open
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(n: usize) -> impl Iterator<Item = Vec3> {
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..n).map(move |i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let t = golden * i as f32;
            Vec3::new(r * t.cos(), y, r * t.sin())
        })
    }

    /// The shipped normals load, every value is finite, and they say what
    /// the planet is: warm tropics, frozen poles.
    #[test]
    fn the_shipped_normals_load_and_read_warm_tropics_and_frozen_poles() {
        let cfg = TerrainConfig::for_version(7).unwrap();
        let normals = SeaNormals::of(cfg.ocean.as_ref().unwrap(), cfg.radius_m);
        assert_eq!(normals.coldest_c.len(), normals.grid.len());
        let (cold_eq, warm_eq, ice_eq) = normals.at(Vec3::X);
        let (cold_pole, _, ice_pole) = normals.at(-Vec3::Y);
        assert!(
            cold_eq > cold_pole + 10.0,
            "{cold_eq} at the equator, {cold_pole} at the pole"
        );
        assert!(warm_eq >= cold_eq);
        assert!(
            ice_eq < 0.01 && ice_pole > 0.5,
            "ice {ice_eq} and {ice_pole}"
        );
        assert!(SeaNormals::parse(&G7[..20], cfg.radius_m).is_err());
    }

    /// On version 7 every place but marsh and river water occurs, each in a
    /// share that makes it a place and not the whole sea; an older version
    /// has only the coast and the open sea.
    #[test]
    fn version_7_has_every_sea_place_and_older_versions_only_coast_and_open() {
        let v7 = TerrainConfig::for_version(7).unwrap();
        let v6 = TerrainConfig::for_version(6).unwrap();
        let mut counts = std::collections::BTreeMap::new();
        let mut sea = 0;
        for d in sphere(40_000) {
            let Some(env) = environment(&v7, d) else {
                continue;
            };
            sea += 1;
            *counts.entry(format!("{env:?}")).or_insert(0usize) += 1;
            let old = environment(&v6, d);
            assert!(
                matches!(old, Some(Environment::Coast | Environment::Open) | None),
                "{old:?}"
            );
        }
        println!("{sea} sea samples: {counts:?}");
        for place in [
            "Open", "Coast", "Reef", "Kelp", "UnderIce", "Vent", "Trench",
        ] {
            let n = counts.get(place).copied().unwrap_or(0);
            assert!(n > 0, "no {place} anywhere: {counts:?}");
            if place != "Open" {
                assert!(n < sea / 4, "{place} is {n} of {sea}");
            }
        }
    }

    /// The chart agrees with the environment everywhere but the coast, which
    /// it shows as open sea.
    #[test]
    fn the_chart_is_the_environment_but_the_coast() {
        let v7 = TerrainConfig::for_version(7).unwrap();
        for d in sphere(20_000) {
            let h = planet_gen::surface_altitude(&v7, d);
            let env = environment_at(&v7, d, h);
            let chart = chart_at(&v7, d, h);
            let expect = if env == Some(Environment::Coast) {
                Some(Environment::Open)
            } else {
                env
            };
            assert_eq!(chart, expect, "{d}");
        }
        let v6 = TerrainConfig::for_version(6).unwrap();
        assert_eq!(chart_at(&v6, Vec3::X, -50.0), None);
    }

    /// The trench is carved inside the column, and only on sea floor.
    #[test]
    fn the_trench_stays_inside_the_column() {
        let v7 = TerrainConfig::for_version(7).unwrap();
        let mut deepest = 0.0f32;
        for d in sphere(40_000) {
            let h = planet_gen::surface_altitude(&v7, d);
            deepest = deepest.min(h);
            if environment_at(&v7, d, h) == Some(Environment::Trench) {
                assert!(h < v7.sea_level_m - v7.ocean.unwrap().trench_depth_m);
            }
        }
        assert!(
            deepest > crate::column::BASE_M as f32 + 2.0,
            "the deepest floor {deepest} m stays above the column's floor"
        );
        assert!(deepest < -130.0, "a trench reaches {deepest} m");
    }
}
