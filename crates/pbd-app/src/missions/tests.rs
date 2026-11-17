use super::*;
use crate::config::{FaunaConfig, HusbandryConfig};
use pbd_core::farming::Crop;
use pbd_core::goods::Good;
use pbd_core::missions::{MissionEntry, ObjectiveEntry};

fn objective(kind: Kind, what: &str, count: u32) -> ObjectiveEntry {
    ObjectiveEntry {
        kind,
        what: what.into(),
        count,
        label: what.into(),
    }
}

/// An app with the missions plugin and the shipped tables, counting into a
/// memory-only world, with `chain` (or the shipped one).
fn app(chain: Option<MissionFile>) -> App {
    let (recipes, _) = crate::config::recipe_book();
    let fauna =
        FaunaConfig(ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap());
    let land = HusbandryConfig(Default::default());
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    if let Some(file) = chain {
        let fish = fauna.0.bodies.values().flatten().map(|s| s.name.as_str());
        let animals = land.0.land.values().flatten().map(|s| s.name.as_str());
        let catalog = Catalog::new(&recipes.0, fish, animals);
        app.insert_resource(MissionChain(file.resolve(&catalog).unwrap()));
    }
    app.insert_resource(recipes)
        .insert_resource(fauna)
        .insert_resource(land)
        .insert_resource(WorldSave::memory_only())
        .insert_resource(Hotbar::default())
        .add_plugins(MissionsPlugin);
    app.update();
    app
}

fn done(app: &App, id: &str) -> bool {
    app.world().resource::<Missions>().progress.is_done(id)
}

/// Every kind is counted from the message its system raises: the existing
/// ones (`Gained`, `Planted`, `Deed`, `Cooked`) and `Did`.
#[test]
fn every_kind_counts_from_its_message() {
    let rows = [
        ("find", objective(Kind::Find, "seed", 3)),
        (
            "gather",
            objective(Kind::Gather, "wheat-seed|block:dirt", 2),
        ),
        ("harvest", objective(Kind::Harvest, "crop", 2)),
        ("collect", objective(Kind::Collect, "egg", 1)),
        ("craft", objective(Kind::Craft, "recipe:mixed-ore", 1)),
        ("smelt", objective(Kind::Smelt, "ingot", 3)),
        ("plant", objective(Kind::Plant, "turnip-seed", 1)),
        ("tame", objective(Kind::Tame, "yard hen", 1)),
        ("pen", objective(Kind::Pen, "any", 1)),
        ("build-pen", objective(Kind::BuildPen, "any", 1)),
        ("cook", objective(Kind::Cook, "grilled-fish", 2)),
        ("cast", objective(Kind::Cast, "water", 1)),
        ("catch", objective(Kind::Catch, "fish", 1)),
        ("place", objective(Kind::Place, "block:torch", 1)),
        ("till", objective(Kind::Till, "any", 1)),
    ];
    let file = MissionFile {
        awaited: vec![],
        missions: rows
            .iter()
            .map(|(id, o)| MissionEntry {
                id: id.to_string(),
                title: id.to_string(),
                help: String::new(),
                needs: vec![],
                objectives: vec![o.clone()],
                unlocks: vec![],
                after: vec![],
                branch: String::new(),
            })
            .collect(),
    };
    let mut app = app(Some(file));
    for (id, _) in &rows {
        assert!(!done(&app, id), "{id} is not done before its message");
    }
    let world = app.world_mut();
    let gained = |item, count, how| Gained { item, count, how };
    world.write_message(gained(Item::Good(Good::WHEAT_SEED), 1, How::Found));
    world.write_message(gained(Item::Good(Good::TURNIP_SEED), 2, How::Picked));
    world.write_message(gained(Item::Good(Good::TURNIP), 2, How::Harvested));
    world.write_message(gained(Item::Good(Good::EGG), 1, How::Collected));
    world.write_message(gained(Item::Good(Good::COPPER_INGOT), 2, How::Made));
    world.write_message(gained(Item::Good(Good::IRON_INGOT), 1, How::Made));
    world.write_message(gained(Item::Block(Material::Dirt), 1, How::Found));
    world.write_message(Planted {
        crop: Crop::all().nth(1).unwrap(),
    });
    world.write_message(Deed::Tamed { species: 0 });
    world.write_message(Deed::Penned { species: 0 });
    world.write_message(Deed::PenBuilt { pen: 1, cells: 7 });
    world.write_message(Cooked {
        recipe: "grilled-fish".into(),
        dish: Item::Good(Good::by_code("grilled-fish").unwrap()),
        count: 2,
        fish: 2,
    });
    world.write_message(Did::new(Kind::Cast, "water", 1));
    world.write_message(Did::new(Kind::Catch, "minnow", 1));
    world.write_message(Did::block(Kind::Place, Material::Torch));
    world.write_message(Did::block(Kind::Till, Material::Grass));
    world.write_message(Did::new(Kind::Craft, "recipe:mixed-ore", 1));
    app.update();
    for (id, _) in &rows {
        assert!(done(&app, id), "{id} counted from its message");
    }
    // Everything done is in the save, as the player's record.
    let save = app.world().resource::<WorldSave>();
    let stored = progress_of(save);
    assert_eq!(stored.done.len(), rows.len());
    assert!(save.records.get(MISSIONS_KIND, MISSIONS_ID).is_some());
}

/// The shipped tree on a new world (which owns no tools since
/// `tool-crafting`) opens on "Gather by hand", and the HUD's line follows
/// the digging; three fish are counted one at a time once fishing opens.
#[test]
fn a_new_world_starts_at_the_first_mission_and_the_line_follows() {
    let mut app = app(None);
    app.insert_resource(crate::fish::ToolSlot(
        pbd_core::inventory::Equipment::new_world(),
    ));
    app.insert_resource(Hotbar::starting_kit());
    // A world just opened joins with what it holds.
    app.insert_resource(WorldSave::memory_only());
    app.update();
    let line = |app: &App| {
        let chain = &app.world().resource::<MissionChain>().0;
        app.world().resource::<Missions>().progress.line(chain)
    };
    assert_eq!(line(&app).as_deref(), Some("Gather by hand: fiber 0/8"));
    let world = app.world_mut();
    for (good, count) in [("fibre", 8), ("stone", 9)] {
        world.write_message(Gained {
            item: Item::Good(Good::by_code(good).unwrap()),
            count,
            how: How::Found,
        });
    }
    app.update();
    assert_eq!(line(&app).as_deref(), Some("Gather by hand: soil 0/6"));
    for _ in 0..6 {
        app.world_mut()
            .write_message(Did::block(Kind::Gather, Material::Dirt));
    }
    app.update();
    assert!(done(&app, "gather"));
    assert_eq!(line(&app).as_deref(), Some("Craft an axe 0/1"));
    let missions = app.world().resource::<Missions>();
    assert_eq!(
        missions.notice.as_ref().map(|(t, _)| t.as_str()),
        Some("Gather by hand")
    );
    // The axe was locked until the gathering was done.
    let locks = app.world().resource::<RecipeLocks>();
    assert_eq!(locks.locked("stone-axe"), None);
    assert_eq!(locks.locked("planks"), Some("Gather wood"));
    // Skip ahead to fishing: the record says the trunk is done.
    {
        let mut missions = app.world_mut().resource_mut::<Missions>();
        for id in [
            "axe",
            "wood",
            "planks-rope",
            "crafting-table",
            "stone-tools",
            "fire",
            "rod",
        ] {
            missions.progress.done.insert(id.into());
        }
    }
    app.world_mut()
        .write_message(Did::new(Kind::Cast, "water", 1));
    app.update();
    for n in 1..=3 {
        app.world_mut()
            .write_message(Did::new(Kind::Catch, "minnow", 1));
        app.update();
        if n < 3 {
            assert_eq!(line(&app), Some(format!("Go fishing: fish {n}/3")));
        }
    }
    assert!(done(&app, "go-fishing"));
    // Bait is a good now: digging it and landing fish on it is next (G1).
    assert_eq!(line(&app).as_deref(), Some("Dig for bait: dig bait 0/3"));
    app.world_mut().write_message(Gained {
        item: Item::Good(Good::BAIT),
        count: 3,
        how: How::Found,
    });
    for _ in 0..3 {
        app.world_mut()
            .write_message(Did::new(Kind::Baited, "minnow", 1));
    }
    app.update();
    assert!(done(&app, "bait"));
    // A kind counts once (G4): two minnows and a perch are two kinds.
    for kind in ["minnow", "minnow", "banded perch"] {
        app.world_mut()
            .write_message(Did::new(Kind::Species, kind, 1));
    }
    app.update();
    assert_eq!(line(&app).as_deref(), Some("Three kinds of fish 2/3"));
}

/// An old world, saved before missions and before tool crafting, opens with
/// the tree started and what it already holds done: its four tools (owned
/// at the stone tier, `Equipment::default`) close the trunk and the rod,
/// and nothing it could make is locked (T.3). A world with a record reads
/// it back.
#[test]
fn an_old_world_joins_with_what_it_holds_and_a_record_reads_back() {
    let mut app = app(None);
    app.insert_resource(crate::fish::ToolSlot::default());
    app.insert_resource(WorldSave::memory_only());
    app.update();
    for id in [
        "gather",
        "axe",
        "wood",
        "crafting-table",
        "stone-tools",
        "rod",
    ] {
        assert!(done(&app, id), "{id}: the tools it owns close the trunk");
    }
    let locks = app.world().resource::<RecipeLocks>();
    for code in ["stone-axe", "planks", "rope", "workbench", "rod"] {
        assert_eq!(locks.locked(code), None, "{code}");
    }
    assert!(!done(&app, "go-fishing"));

    // Count a cast, then reopen the world from its records.
    app.world_mut()
        .write_message(Did::new(Kind::Cast, "water", 1));
    app.update();
    let records = app.world().resource::<WorldSave>().records.clone();
    let mut reopened = WorldSave::memory_only();
    reopened.records = records;
    app.insert_resource(reopened);
    app.update();
    assert!(done(&app, "rod"));
    let chain = &app.world().resource::<MissionChain>().0;
    assert_eq!(
        app.world()
            .resource::<Missions>()
            .progress
            .line(chain)
            .as_deref(),
        Some("Go fishing: fish 0/3")
    );
}
