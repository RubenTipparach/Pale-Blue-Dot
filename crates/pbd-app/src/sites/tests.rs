use super::*;
use crate::saves::{self, LOG, WorldSave};
use pbd_core::edits::Edit;
use pbd_core::inventory::{Item, Slots};
use pbd_core::terrain::Material;

fn temporary(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-sites-{name}-{}-{}",
        std::process::id(),
        saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// The shipped rules on a coarser grid, which is fast and makes the same
/// kind of list.
fn rules() -> SitesConfig {
    SitesConfig {
        level: 5,
        ..load_rules()
    }
}

fn start() -> Vec3 {
    FlightViewConfig::default().spawn_direction
}

/// "An old save gains its list": a world whose log predates sites is given
/// its list on its next open, written after what the player did, and read
/// back from the save; and a changed `sites.ron` does not change it (task
/// 3.1).
#[test]
fn an_old_save_gains_its_list_and_keeps_it_when_the_rules_change() {
    let root = temporary("keep");
    let slot = saves::create(&root, "Towns", 31).unwrap();
    let terrain = TerrainConfig::for_version(slot.identity.as_ref().unwrap().generator)
        .expect("this build's generator");
    let mut carried = Slots::new();
    carried.give(Item::Block(Material::Dirt), 1);
    let made = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        assert!(save.accept(
            Edit {
                cell: 77,
                layer: 200,
                material: Material::Air,
            },
            None,
            &carried
        ));
        assert!(stored(&save).is_none(), "no list yet");
        let (list, seq) = ensure(&mut save, &rules(), &terrain, start(), 4).expect("made");
        assert!(seq > 0, "queued to the disk");
        assert!(!list.is_empty());
        save.drain();
        assert!(save.committed() >= seq, "and on it");
        list
    };
    let path = root.join(&slot.id).join(LOG);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.lines().next().unwrap().starts_with("77 200 0"),
        "the dig first, untouched"
    );
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rec @c site "))
            .count(),
        made.len(),
        "a line per site, the creation's"
    );
    assert!(
        text.trim_end()
            .lines()
            .last()
            .unwrap()
            .starts_with("rec @c site-list 0 1 ")
    );
    let identity = saves::read_identity(&root.join(&slot.id)).expect("an identity");
    assert_eq!(identity.records.get("site"), Some(&1));
    assert_eq!(identity.records.get("site-list"), Some(&1));

    // Retuned rules: other counts, other jitter, the town pinned elsewhere.
    let mut retuned = rules();
    retuned.jitter = 0.9;
    for rule in &mut retuned.kinds {
        rule.count += 3;
    }
    let mut reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    assert_eq!(
        stored(&reopened).as_deref(),
        Some(made.as_slice()),
        "read back whole"
    );
    let (kept, seq) = ensure(&mut reopened, &retuned, &terrain, start(), 4).expect("kept");
    assert_eq!(kept, made, "the world keeps its towns");
    assert_eq!(seq, 0, "and nothing is written");
    reopened.drain();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "byte for byte"
    );
    assert_eq!(reopened.edits.for_cell(77), &[(200, Material::Air)]);
    let _ = std::fs::remove_dir_all(&root);
}

/// The game's system reads a world's stored list and shows it at once; it
/// never remakes a list a save holds, whatever the rules now say.
#[test]
fn the_game_shows_the_list_its_save_holds() {
    let root = temporary("show");
    let slot = saves::create(&root, "Shown", 32).unwrap();
    let terrain = TerrainConfig::for_version(slot.identity.as_ref().unwrap().generator)
        .expect("this build's generator");
    let made = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let (list, _) = ensure(&mut save, &rules(), &terrain, start(), 4).expect("made");
        save.drain();
        list
    };
    let mut retuned = rules();
    retuned.jitter = 0.9;
    let mut app = App::new();
    app.insert_resource(SitesRules(retuned))
        .init_resource::<WorldSites>()
        .insert_resource(WorldSave::open(root.clone(), saves::list(&root)[0].clone()))
        .add_systems(Update, keep_sites);
    app.update();
    let sites = app.world().resource::<WorldSites>();
    assert_eq!(sites.ready(), Some(made.as_slice()));
    assert!(!sites.surveying());
    let _ = std::fs::remove_dir_all(&root);
}

/// The list a new world is given is the list the owner approved on the map
/// mockup (task 1.4): the game makes it from the spawn direction, as the
/// mockup's instrument did, so every site, name and place is the same. A new
/// version-6 player starts on the level ground 438 m off that direction, and
/// the small town is measured from there too: 566 m to its centre, a walk of
/// under two minutes (the design's finding 6, which this pins).
#[test]
fn a_new_worlds_list_is_the_one_the_owner_approved() {
    let view = FlightViewConfig::default();
    let list = sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    );
    let approved: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/mockups/world-map/sites.json"
        ))
        .expect("the mockup's list"),
    )
    .expect("json");
    let approved: Vec<(u64, String)> = approved["sites"]
        .as_array()
        .expect("sites")
        .iter()
        .map(|s| {
            (
                s["id"].as_u64().unwrap(),
                s["name"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let made: Vec<(u64, String)> = list
        .sites
        .iter()
        .map(|s| (u64::from(s.id), s.name.clone()))
        .collect();
    assert_eq!(made, approved);
    let ground = crate::planet::PlanetContact::test_planet(crate::planet::BASE_LEVEL.into());
    let sea = crate::sea::Sea::new(&crate::config::WaterSettings::default()).radius;
    let start = crate::walking::new_world_start(&ground, sea, view.spawn_direction);
    let home = list.sites.iter().find(|s| s.home).expect("a home town");
    let metres = sites::arc_m(
        home.direction,
        start,
        crate::planet::terrain_config().radius_m,
    );
    assert!(
        (500.0..600.0).contains(&metres),
        "{} is {metres:.0} m from the level start; finding 6 says 566",
        home.name
    );
}
