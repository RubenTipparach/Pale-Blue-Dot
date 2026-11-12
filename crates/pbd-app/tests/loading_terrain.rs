//! The loading screen builds the planet once, with the towns' ground already
//! installed (`loading-screen` decision 1). Before it, every open built the
//! planet without the ground and then rebuilt it with it. A saved world must
//! open to the same terrain either way, cell for cell.
//!
//! The towns' ground is one value for the whole process
//! (`settlement::ground::install`), read by every height. A unit test that
//! installed it would change the ground under every other test running beside
//! it, so this is a process of its own, and one test.

use pbd_app::config::{ColumnSettings, ScatterSettings};
use pbd_app::loading::{Inputs, Progress, prepare};
use pbd_app::planet::{PreparedPlanet, terrain_config};
use pbd_app::saves::WorldSave;
use pbd_app::towns::{Opened, TownAssets, grounds_of, read_back, read_or_lay};
use pbd_core::settlement::ground::{self, Ground};

#[test]
fn the_planet_built_once_is_the_planet_built_then_rebuilt() {
    let config = *terrain_config();
    let inputs = Inputs {
        rules: pbd_app::sites::load_rules(),
        assets: TownAssets::load(),
        columns: ColumnSettings::default(),
        scatter: ScatterSettings::default(),
        near: pbd_app::sites::list_spawn(),
    };

    // The new path: sites, towns and ground, then the planet once.
    let new = prepare(
        WorldSave::memory_only(),
        inputs.clone(),
        &Progress::default(),
    );
    let sites = new.sites.clone().expect("a fresh world gets its sites");
    assert!(
        ground::installed().is_some(),
        "the fresh world's towns' ground is installed"
    );
    let once = new.planet.digest();

    // The old path, as `keep_sites`, `build_towns` and `settle` ran it: the
    // planet built with no ground, then the same sites and towns stored, the
    // ground installed, and the planet built again round the same place.
    ground::install(None);
    let bare = PreparedPlanet::build(
        inputs.near,
        &inputs.columns,
        &inputs.scatter,
        &WorldSave::memory_only().edits,
    )
    .digest();
    let mut old = WorldSave::memory_only();
    let (old_sites, _) = pbd_app::sites::ensure(
        &mut old,
        &inputs.rules,
        &config,
        pbd_app::sites::list_spawn(),
        pbd_app::sites::threads(),
    )
    .expect("the old path's sites");
    assert_eq!(old_sites, sites, "both paths make the same sites");
    let Opened {
        mut held, waiting, ..
    } = read_or_lay(&mut old, &old_sites, &inputs.assets, &config);
    held.extend(read_back(&old, waiting));
    let (grounds, kept) = grounds_of(held, &inputs.assets, &config);
    assert!(!kept.is_empty(), "the fresh world has towns");
    ground::install(Some(Ground::new(config, grounds)));
    let twice =
        PreparedPlanet::build(inputs.near, &inputs.columns, &inputs.scatter, &old.edits).digest();
    ground::install(None);

    assert_ne!(
        bare, once,
        "the towns' ground changes the planet, so the comparison means something"
    );
    assert_eq!(
        once, twice,
        "base records, fine levels and column tier: built once equals built then rebuilt"
    );
    assert_eq!(
        new.save.records, old.records,
        "the loading path stores the records the systems stored"
    );
}
