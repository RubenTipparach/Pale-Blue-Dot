//! A new world starts where the same seed always started, under the same
//! sky, whether its towns stand before its walker or after
//! (`loading-screen`).
//!
//! Before the loading screen the walker was set down at `Startup` on a planet
//! built without the towns' ground, and the towns stood later. The loading
//! screen builds the planet once with their ground, and the start's search
//! for a level plain found Holbrook's terrace instead: 470 m off, under a
//! passing storm, so the perf suite's `walk` ran in rain it had never had.
//!
//! The towns' ground is one value for the whole process, so this is a
//! process of its own, and one test.

use pbd_app::atmosphere::Air;
use pbd_app::config::{ColumnSettings, ScatterSettings, WaterSettings};
use pbd_app::flight_view::FlightViewConfig;
use pbd_app::loading::{Inputs, Progress, prepare};
use pbd_app::planet::{PreparedPlanet, terrain_config};
use pbd_app::saves::WorldSave;
use pbd_app::towns::TownAssets;
use pbd_core::atmosphere::AtmosphereSettings;
use pbd_core::daylight::Clock;
use pbd_core::settlement::ground;

#[test]
fn a_new_world_starts_where_it_did_before_its_towns_stood() {
    let spawn = FlightViewConfig::default().spawn_direction;
    let sea = pbd_app::sea::Sea::new(&WaterSettings::default()).radius;

    // The path before the loading screen: the planet with no towns' ground.
    ground::install(None);
    let bare = PreparedPlanet::build(
        spawn,
        &ColumnSettings::default(),
        &ScatterSettings::default(),
        &WorldSave::memory_only().edits,
    );
    let before = bare.new_world_start(sea, spawn).expect("a start");

    // The loading screen's: a fresh world prepared, its towns' ground
    // installed, the planet built once with it.
    let prepared = prepare(
        WorldSave::memory_only(),
        Inputs {
            rules: pbd_app::sites::load_rules(),
            assets: TownAssets::load(),
            columns: ColumnSettings::default(),
            scatter: ScatterSettings::default(),
            near: spawn,
        },
        &Progress::default(),
    );
    assert!(
        ground::installed().is_some(),
        "the fresh world's towns' ground is installed, so the comparison means something"
    );
    let after = prepared
        .planet
        .new_world_start(sea, spawn)
        .expect("a start");
    assert_eq!(after, before, "the same seed starts in the same place");

    // And under the same sky: the perf suite's `walk` (`--time 10`, no
    // forcing) read "Weather here: cover 0.69, rain 0.00" at its first
    // sample before the loading screen.
    let air = Air::open(
        AtmosphereSettings::default(),
        terrain_config().seed,
        None,
        Clock::at(0, 10.0).seconds,
    );
    let cell = pbd_core::weather::cloud_cell(air.shown.to(), after);
    assert!(
        (cell.cover - 0.69).abs() < 0.015 && cell.rain == 0.0,
        "the start's weather at 10:00: cover {:.3}, rain {:.3}",
        cell.cover,
        cell.rain
    );
    ground::install(None);
}
