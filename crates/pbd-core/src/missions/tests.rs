use super::*;
use crate::station::RecipeFile;

/// The shipped recipe book: the forge's and the kitchen's.
pub(super) fn shipped_book() -> RecipeBook {
    let mut book =
        ron::from_str::<RecipeFile>(include_str!("../../../../assets/config/recipes.ron"))
            .unwrap()
            .resolve()
            .unwrap();
    let kitchen = ron::from_str::<crate::cooking::CookingFile>(include_str!(
        "../../../../assets/config/cooking.ron"
    ))
    .unwrap()
    .resolve()
    .unwrap();
    book.recipes.extend(kitchen.recipes);
    book
}

/// The shipped rosters' names: every body's fish and land animals.
pub(super) fn shipped_catalog(book: &RecipeBook) -> Catalog {
    let fauna: crate::fauna::FaunaSettings =
        ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap();
    let land = crate::husbandry::HusbandrySettings::default();
    let fish: Vec<String> = fauna
        .bodies
        .values()
        .flatten()
        .map(|s| s.data_name())
        .collect();
    let animals: Vec<String> = land
        .land
        .values()
        .flatten()
        .map(|s| s.data_name())
        .collect();
    Catalog::new(
        book,
        fish.iter().map(String::as_str),
        animals.iter().map(String::as_str),
    )
}

fn shipped_file() -> MissionFile {
    ron::from_str(include_str!("../../../../assets/config/missions.ron")).unwrap()
}

fn objective(kind: Kind, what: &str, count: u32) -> ObjectiveEntry {
    ObjectiveEntry {
        kind,
        what: what.into(),
        count,
        label: Some(what.into()),
    }
}

fn mission(id: &str, needs: &[&str], objectives: Vec<ObjectiveEntry>) -> MissionEntry {
    MissionEntry {
        id: id.into(),
        title: Some(id.into()),
        help: None,
        needs: needs.iter().map(|n| n.to_string()).collect(),
        objectives,
        unlocks: Vec::new(),
        after: Vec::new(),
        branch: String::new(),
        icon: String::new(),
        coins: 0,
        mode: None,
    }
}

fn none(_: &Objective) -> u32 {
    0
}

/// A short chain walked by a list of events: each mission opens when the one
/// before it is done, counts only its own kind and thing, and the record
/// keeps counts only for open missions.
#[test]
fn a_chain_is_walked_by_its_events() {
    let book = shipped_book();
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![
            mission("cast", &[], vec![objective(Kind::Cast, "water", 1)]),
            mission("catch", &["cast"], vec![objective(Kind::Catch, "fish", 3)]),
            mission(
                "cook",
                &["catch"],
                vec![objective(Kind::Cook, "grilled-fish|chili-fish", 2)],
            ),
        ],
    };
    let chain = file.resolve(&shipped_catalog(&book)).unwrap();
    let mut progress = Progress::default();
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("cast")
    );
    assert_eq!(progress.line(&chain).as_deref(), Some("cast 0/1"));

    // A fish before its mission opened is not counted.
    let fish = Happened::new(Kind::Catch, "minnow", 1);
    let step = progress.record(&chain, &fish, &none);
    assert!(!step.changed && step.done.is_empty());

    let step = progress.record(&chain, &Happened::new(Kind::Cast, "water", 1), &none);
    assert_eq!(step.done, ["cast"]);
    assert_eq!(progress.line(&chain).as_deref(), Some("catch 0/3"));
    for n in 1..=2 {
        let step = progress.record(&chain, &fish, &none);
        assert!(step.changed && step.done.is_empty());
        assert_eq!(progress.line(&chain), Some(format!("catch {n}/3")));
    }
    // Another kind does not count.
    let other = Happened::new(Kind::Cook, "grilled-fish", 1);
    assert!(!progress.record(&chain, &other, &none).changed);
    assert_eq!(progress.record(&chain, &fish, &none).done, ["catch"]);
    // A dish not named does not count; a named one does, up to its count.
    let bread = Happened::new(Kind::Cook, "flatbread", 1);
    assert!(!progress.record(&chain, &bread, &none).changed);
    let step = progress.record(&chain, &Happened::new(Kind::Cook, "chili-fish", 5), &none);
    assert_eq!(step.done, ["cook"]);
    assert_eq!(progress.current(&chain), None);
    assert_eq!(progress.line(&chain), None);
    assert!(
        progress.counts.is_empty(),
        "no counts kept for closed missions"
    );
    assert_eq!(progress.done.len(), 3);
}

/// A state-shaped objective is met by what the world already holds when its
/// mission opens: a world that owns its tools is not told to make them, and
/// the mission after opens at once.
#[test]
fn a_state_met_mission_completes_on_open() {
    // The workbench's stone tools (`tool-crafting`) are in the shipped book.
    let book = shipped_book();
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![
            mission(
                "mining-tools",
                &[],
                vec![
                    objective(Kind::Craft, "tool:pickaxe", 1),
                    objective(Kind::Craft, "tool:shovel", 1),
                ],
            ),
            mission(
                "cast",
                &["mining-tools"],
                vec![objective(Kind::Cast, "water", 1)],
            ),
        ],
    };
    let chain = file.resolve(&shipped_catalog(&book)).unwrap();
    assert!(chain.pending().next().is_none());
    // The world owns both tools: an old save, joining the chain on open.
    let owned = |o: &Objective| {
        u32::from(
            o.kind == Kind::Craft
                && (o.want.matches(Kind::Craft, "tool:pickaxe")
                    || o.want.matches(Kind::Craft, "tool:shovel")),
        )
    };
    let mut progress = Progress::default();
    let step = progress.settle(&chain, &owned);
    assert_eq!(step.done, ["mining-tools"]);
    assert!(step.changed);
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("cast")
    );
    // Settling again changes nothing.
    assert_eq!(progress.settle(&chain, &owned), Step::default());

    // A world that owns only the pickaxe has half the mission done.
    let pick = |o: &Objective| u32::from(o.want.matches(Kind::Craft, "tool:pickaxe"));
    let mut progress = Progress::default();
    assert!(progress.settle(&chain, &pick).done.is_empty());
    assert_eq!(
        progress.line(&chain).as_deref(),
        Some("mining-tools: tool:pickaxe 1/1, tool:shovel 0/1")
    );

    // The shipped tree and an old world (before tool crafting) that owns
    // all four tools at the stone tier: the missions it holds close, and
    // those before them with them, so the trunk is behind it and it is told
    // to go fishing, not to gather by hand.
    let chain = shipped_file().resolve(&shipped_catalog(&book)).unwrap();
    let tools = ["tool:rod", "tool:shovel", "tool:pickaxe", "tool:axe"];
    let held = |o: &Objective| match o.kind {
        Kind::Craft => tools
            .iter()
            .filter(|t| o.want.matches(Kind::Craft, t))
            .count() as u32,
        _ => 0,
    };
    let mut progress = Progress::default();
    let step = progress.settle(&chain, &held);
    for id in [
        "gather",
        "axe",
        "wood",
        "planks-rope",
        "crafting-table",
        "stone-tools",
        "fire",
        "rod",
    ] {
        assert!(progress.is_done(id), "{id} in {:?}", step.done);
    }
    // After the campfire the trunk ends with the chest (`storage-chests` 9).
    assert_eq!(
        progress.line(&chain).as_deref(),
        Some("A chest: Make a chest 0/1, Set it down 0/1, Store a stack 0/1")
    );
}

/// A name the build lacks that the file awaits makes its mission pending,
/// and the missions after it still open; any other unknown name is refused.
#[test]
fn an_awaited_name_makes_a_mission_pending() {
    let book = shipped_book();
    let catalog = shipped_catalog(&book);
    let file = MissionFile {
        crew: None,
        awaited: vec!["oak-planks".into()],
        missions: vec![
            mission("planks", &[], vec![objective(Kind::Craft, "oak-planks", 4)]),
            // Known, but no recipe makes it: pending without being awaited.
            mission(
                "pick",
                &["planks"],
                vec![objective(Kind::Craft, "block:candle", 1)],
            ),
            mission("cast", &["pick"], vec![objective(Kind::Cast, "water", 1)]),
        ],
    };
    let chain = file.resolve(&catalog).unwrap();
    let pending: Vec<&str> = chain.pending().map(|(id, _)| id).collect();
    assert_eq!(pending, ["planks", "pick"]);
    let progress = Progress::default();
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("cast")
    );
    assert_eq!(
        progress.standing(&chain, chain.get("planks").unwrap()),
        Standing::Pending
    );
    // A pending mission counts nothing.
    let mut progress = Progress::default();
    let step = progress.record(&chain, &Happened::new(Kind::Craft, "oak-planks", 4), &none);
    assert!(!step.changed);

    // One alternative known is enough for a mission to play.
    let file = MissionFile {
        crew: None,
        awaited: vec!["bark".into()],
        missions: vec![mission(
            "gather",
            &[],
            vec![objective(Kind::Gather, "bark|block:dirt", 2)],
        )],
    };
    assert!(file.resolve(&catalog).unwrap().pending().next().is_none());

    for bad in [
        mission("x", &[], vec![objective(Kind::Craft, "unobtainium", 1)]),
        mission("x", &[], vec![objective(Kind::Catch, "kraken", 1)]),
        mission("x", &["nothing"], vec![objective(Kind::Cast, "water", 1)]),
        mission("x", &[], vec![objective(Kind::Cast, "water", 0)]),
        mission("x", &[], vec![]),
    ] {
        let file = MissionFile {
            crew: None,
            awaited: vec![],
            missions: vec![bad],
        };
        assert!(file.resolve(&catalog).is_err(), "{file:?}");
    }
}

/// The record's body reads back as it was written, and an id this build's
/// chain lacks (a newer build's mission) is kept rather than dropped.
#[test]
fn the_record_round_trips_and_keeps_what_it_does_not_know() {
    let mut progress = Progress::default();
    for id in [
        "gather",
        "planks-rope",
        "crafting-table",
        "mining-tools",
        "axe",
        "rod",
        "fire",
    ] {
        progress.done.insert(id.into());
    }
    progress.done.insert("go-fishing".into());
    progress.done.insert("a-later-mission".into());
    progress.counts.insert("catch-fish".into(), vec![2]);
    let record = crate::records::Record::of(MISSIONS_KIND, MISSIONS_ID, MISSIONS_SCHEMA, &progress);
    assert_eq!(record.read::<Progress>(), Some(progress.clone()));

    let chain = shipped_file()
        .resolve(&shipped_catalog(&shipped_book()))
        .unwrap();
    let mut read = record.read::<Progress>().unwrap();
    read.settle(&chain, &none);
    assert!(read.done.contains("a-later-mission"));
    // On the tree, the old line's done ids stay done, and a done mission's
    // parents are done with it (the fire's line back to the gathering:
    // `starter-missions`, "The first mission: open the pack"). The chest,
    // after the fire, is next.
    assert_eq!(
        read.line(&chain).as_deref(),
        Some("A chest: Make a chest 0/1, Set it down 0/1, Store a stack 0/1")
    );
}

/// The shipped tree (`starter-missions` TS.1, M33 version 3): every `what`
/// names a known good, recipe, block, species or action, every `unlocks` a
/// recipe; it is a strict tree (one parent each, `after` gates naming
/// missions) reachable from `gather`; and played with the events the game
/// raises it closes every mission whose kind is live.
#[test]
fn the_shipped_chain_names_only_what_the_build_knows() {
    let book = shipped_book();
    // An Explorer world's tree, as it was before Survival's paths.
    let chain = shipped_file()
        .resolve(&shipped_catalog(&book))
        .unwrap()
        .for_mode(Mode::Explorer);
    assert_eq!(chain.missions.len(), 44);
    // One parent each, and only `gather` without one.
    for m in &chain.missions {
        assert!(m.needs.len() <= 1, "{}: {:?}", m.id, m.needs);
        assert_eq!(m.needs.is_empty(), m.id == "gather", "{}", m.id);
        for gate in &m.after {
            assert!(chain.get(gate).is_some(), "{}: after {gate}", m.id);
        }
    }
    // Reachable from `gather` by the parent lines.
    let mut reached = vec!["gather".to_string()];
    let mut at = 0;
    while at < reached.len() {
        let id = reached[at].clone();
        for m in chain.missions.iter().filter(|m| m.needs.contains(&id)) {
            reached.push(m.id.clone());
        }
        at += 1;
    }
    assert_eq!(reached.len(), chain.missions.len(), "{reached:?}");
    // The fire is in the trunk, and every branch hangs off it.
    for id in ["rod", "find-seeds", "fence", "dig-deep", "build"] {
        assert_eq!(chain.get(id).unwrap().needs, ["fire"], "{id}");
    }
    assert_eq!(chain.get("water-field").unwrap().after, ["go-fishing"]);
    // Nothing waits on the game: bait is a good now (G1).
    let pending: Vec<&str> = chain.pending().map(|(id, _)| id).collect();
    assert!(pending.is_empty(), "{pending:?}");
    let progress = Progress::default();
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("gather")
    );
    for mission in chain.missions.iter().filter(|m| m.pending.is_none()) {
        for objective in &mission.objectives {
            assert!(
                !matches!(&objective.want, Want::Names(n) if n.is_empty()),
                "{}: {:?}",
                mission.id,
                objective
            );
        }
    }
    for code in [
        "stone-axe",
        "planks",
        "workbench",
        "rod",
        "forge",
        "flatbread",
    ] {
        assert!(
            !chain.recipe_open(&progress, code, &[], false),
            "{code} open"
        );
    }
    let ev = Happened::new;
    let events = [
        ev(Kind::Gather, "fibre", 8),
        ev(Kind::Gather, "stone", 9),
        ev(Kind::Gather, "block:dirt", 6),
        ev(Kind::Open, "pack", 1),
        ev(Kind::Craft, "recipe:stone-axe", 1),
        ev(Kind::Fell, "tree", 2),
        ev(Kind::Gather, "log", 10),
        ev(Kind::Craft, "planks", 8),
        ev(Kind::Craft, "rope", 2),
        ev(Kind::Craft, "block:workbench", 1),
        ev(Kind::Place, "block:workbench", 1),
        ev(Kind::Craft, "tool:pickaxe", 1),
        ev(Kind::Craft, "tool:shovel", 1),
        ev(Kind::Craft, "block:campfire", 1),
        ev(Kind::Place, "block:campfire", 1),
        // The storage mission (`storage-chests` 9).
        ev(Kind::Craft, "block:chest", 1),
        ev(Kind::Place, "block:chest", 1),
        ev(Kind::Store, "block:dirt", 1),
        ev(Kind::Craft, "tool:rod", 1),
        ev(Kind::Cast, "water", 1),
        ev(Kind::Catch, "minnow", 3),
        ev(Kind::Gather, "bait", 3),
        ev(Kind::Baited, "minnow", 3),
        ev(Kind::Species, "minnow", 1),
        ev(Kind::Species, "minnow", 1),
        ev(Kind::Species, "perch", 1),
        ev(Kind::Species, "pike", 1),
        ev(Kind::Waters, "sea", 3),
        ev(Kind::Waters, "fresh", 3),
        ev(Kind::Open, "map", 1),
        ev(Kind::Reach, "marked", 1),
        ev(Kind::Swim, "water", 20),
        ev(Kind::Gather, "block:reef rock", 1),
        ev(Kind::Reach, "town", 1),
        ev(Kind::Board, "boat", 1),
        ev(Kind::Travel, "boat", 100),
        ev(Kind::Cook, "grilled-fish", 2),
        ev(Kind::Eat, "grilled-fish", 1),
        ev(Kind::Find, "wheat-seed", 10),
        ev(Kind::Till, "any", 6),
        ev(Kind::Plant, "wheat-seed", 6),
        ev(Kind::Pour, "water", 1),
        ev(Kind::Harvest, "wheat", 6),
        ev(Kind::Cook, "flatbread", 2),
        ev(Kind::Harvest, "turnip", 3),
        ev(Kind::Harvest, "chili", 3),
        ev(Kind::Harvest, "frost-kale", 3),
        ev(Kind::Cook, "roast-turnip", 1),
        ev(Kind::Craft, "recipe:fence", 6),
        ev(Kind::Craft, "recipe:gate", 1),
        ev(Kind::Tame, "yard hen", 1),
        ev(Kind::BuildPen, "any", 1),
        ev(Kind::Pen, "yard hen", 1),
        ev(Kind::Collect, "egg", 5),
        ev(Kind::Tame, "woolback", 1),
        ev(Kind::Collect, "wool", 3),
        ev(Kind::Cook, "fried-egg", 1),
        ev(Kind::Eat, "grilled-fish", 1),
        ev(Kind::Eat, "flatbread", 1),
        ev(Kind::Eat, "fried-egg", 1),
        ev(Kind::Gather, "block:stone", 10),
        ev(Kind::Gather, "block:ore", 4),
        ev(Kind::Gather, "coal", 3),
        ev(Kind::Craft, "block:forge", 1),
        ev(Kind::Place, "block:forge", 1),
        ev(Kind::Smelt, "bronze-ingot", 2),
        ev(Kind::Craft, "recipe:bronze-pickaxe", 1),
        ev(Kind::Gather, "block:stone", 5),
        ev(Kind::Smelt, "iron-ingot", 3),
        ev(Kind::Craft, "recipe:iron-pickaxe", 1),
        ev(Kind::Place, "block:dirt", 20),
        ev(Kind::Craft, "recipe:torch", 2),
        ev(Kind::Place, "block:torch", 4),
        ev(Kind::Night, "light", 1),
    ];
    let mut progress = Progress::default();
    for event in &events {
        progress.record(&chain, event, &none);
    }
    for m in chain.missions.iter().filter(|m| m.pending.is_none()) {
        assert!(progress.is_done(&m.id), "{} not done", m.id);
    }
    assert_eq!(progress.current(&chain), None);
    for m in &chain.missions {
        for code in &m.unlocks {
            assert!(chain.recipe_open(&progress, code, &[], false), "{code}");
        }
    }
}

/// A hidden gate holds a mission shut until it is done, and is not its
/// parent; a second parent is refused (design "One parent per mission").
#[test]
fn an_after_gate_holds_a_mission_shut() {
    let book = shipped_book();
    let catalog = shipped_catalog(&book);
    let mut gated = mission("b", &["a"], vec![objective(Kind::Cast, "water", 1)]);
    gated.after = vec!["c".into()];
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![
            mission("a", &[], vec![objective(Kind::Till, "any", 1)]),
            gated,
            mission("c", &[], vec![objective(Kind::Craft, "planks", 1)]),
        ],
    };
    let chain = file.resolve(&catalog).unwrap();
    let mut progress = Progress::default();
    progress.record(&chain, &Happened::new(Kind::Till, "any", 1), &none);
    let b = chain.get("b").unwrap();
    assert_eq!(progress.standing(&chain, b), Standing::Locked);
    progress.record(&chain, &Happened::new(Kind::Craft, "planks", 1), &none);
    assert_eq!(progress.standing(&chain, b), Standing::Open);
    let two = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![
            mission("a", &[], vec![objective(Kind::Till, "any", 1)]),
            mission("c", &[], vec![objective(Kind::Till, "any", 1)]),
            mission("b", &["a", "c"], vec![objective(Kind::Till, "any", 1)]),
        ],
    };
    assert!(two.resolve(&catalog).is_err());
    let mut bad = file.clone();
    bad.missions[1].after = vec!["nowhere".into()];
    assert!(bad.resolve(&catalog).is_err());
}

/// The Homestead's kinds (`starter-missions` H.1): a felling, a dish eaten
/// and a night by the lights each count once per deed, and only what they
/// name.
#[test]
fn eat_fell_and_night_count_once_per_deed() {
    let book = shipped_book();
    let catalog = shipped_catalog(&book);
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![
            mission("fell", &[], vec![objective(Kind::Fell, "tree", 2)]),
            mission(
                "eat",
                &["fell"],
                vec![
                    objective(Kind::Eat, "grilled-fish", 1),
                    objective(Kind::Eat, "flatbread", 1),
                ],
            ),
            mission("night", &["eat"], vec![objective(Kind::Night, "light", 1)]),
        ],
    };
    let chain = file.resolve(&catalog).unwrap();
    assert!(chain.pending().next().is_none(), "every kind is live");
    let mut progress = Progress::default();
    progress.record(&chain, &Happened::new(Kind::Fell, "tree", 1), &none);
    assert_eq!(progress.line(&chain).as_deref(), Some("fell 1/2"));
    progress.record(&chain, &Happened::new(Kind::Fell, "tree", 1), &none);
    assert!(progress.is_done("fell"));
    // The same dish twice is one objective, not two.
    progress.record(&chain, &Happened::new(Kind::Eat, "grilled-fish", 1), &none);
    progress.record(&chain, &Happened::new(Kind::Eat, "grilled-fish", 1), &none);
    assert!(!progress.is_done("eat"));
    progress.record(&chain, &Happened::new(Kind::Eat, "flatbread", 1), &none);
    assert!(progress.is_done("eat"));
    progress.record(&chain, &Happened::new(Kind::Night, "light", 1), &none);
    assert!(progress.is_done("night"));
    // A name the kind does not know is refused.
    let bad = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![mission("x", &[], vec![objective(Kind::Night, "moon", 1)])],
    };
    assert!(bad.resolve(&catalog).is_err());
}

/// An old world's missions record reads unchanged with the new kinds in
/// the build: the record names ids and counts, never kinds.
#[test]
fn an_old_missions_record_reads_unchanged() {
    let old = r#"(done: ["gather", "planks-rope"], counts: {"crafting-table": [1, 0]})"#;
    let progress: Progress = ron::from_str(old).unwrap();
    assert!(progress.is_done("gather"));
    assert_eq!(progress.counts.get("crafting-table"), Some(&vec![1, 0]));
    let chain = shipped_file()
        .resolve(&shipped_catalog(&shipped_book()))
        .unwrap();
    // On the tree: the pack after the gathering is next (owner, 2026-11-22).
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("pack")
    );
}

/// A world from before the tree keeps every tool it could make (T.3): the
/// line's done ids stay done, and what it holds keeps its recipe open.
#[test]
fn an_old_world_keeps_what_it_could_make() {
    let book = shipped_book();
    let chain = shipped_file().resolve(&shipped_catalog(&book)).unwrap();
    // The old line done to the rod, and the tools it made held.
    let old = r#"(done: ["gather", "planks-rope", "crafting-table", "mining-tools", "axe", "rod"], counts: {})"#;
    let mut progress: Progress = ron::from_str(old).unwrap();
    let tools = ["tool:rod", "tool:shovel", "tool:pickaxe", "tool:axe"];
    let held = |o: &Objective| match o.kind {
        Kind::Craft => tools
            .iter()
            .filter(|t| o.want.matches(Kind::Craft, t))
            .count() as u32,
        _ => 0,
    };
    progress.settle(&chain, &held);
    let book = shipped_book();
    for code in [
        "stone-axe",
        "planks",
        "rope",
        "workbench",
        "stone-pickaxe",
        "stone-shovel",
        "rod",
    ] {
        let recipe = book.get(code).unwrap();
        let makes: Vec<String> = recipe
            .outputs
            .iter()
            .map(|(i, _)| item_code(*i))
            .chain(
                recipe
                    .tools
                    .iter()
                    .map(|(t, _)| item_code(crate::inventory::Item::Tool(*t))),
            )
            .collect();
        assert!(chain.recipe_open(&progress, code, &makes, false), "{code}");
    }
}

/// The lock rule (`starter-missions` T.1): a recipe a mission names is
/// locked until that mission is done; one no mission names is open; a world
/// that already holds what a recipe makes keeps it (T.3); an `unlocks` that
/// names no recipe is refused.
#[test]
fn a_mission_unlocks_its_recipes() {
    let book = shipped_book();
    let catalog = shipped_catalog(&book);
    let mut first = mission("bench", &[], vec![objective(Kind::Craft, "planks", 4)]);
    first.unlocks = vec!["stone-pickaxe".into()];
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![first.clone()],
    };
    let chain = file.resolve(&catalog).unwrap();
    let mut progress = Progress::default();
    assert!(
        !chain.recipe_open(&progress, "stone-pickaxe", &[], false),
        "locked"
    );
    assert!(
        chain.recipe_open(&progress, "planks", &[], false),
        "named by none"
    );
    assert!(
        chain.recipe_open(&progress, "stone-pickaxe", &[], true),
        "held before the tree"
    );
    assert_eq!(
        chain.unlocked_by("stone-pickaxe").map(|m| m.id.as_str()),
        Some("bench")
    );
    progress.record(&chain, &Happened::new(Kind::Craft, "planks", 4), &none);
    assert!(progress.is_done("bench"));
    assert!(
        chain.recipe_open(&progress, "stone-pickaxe", &[], false),
        "unlocked"
    );
    first.unlocks = vec!["no-such-recipe".into()];
    let bad = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![first],
    };
    assert!(bad.resolve(&catalog).is_err());
}

/// "Three kinds of fish" (G4): a kind counts once in a mission however often
/// it is landed, the names go with the mission when it closes, and a record
/// written before `seen` still reads.
#[test]
fn a_kind_of_fish_counts_once_a_mission() {
    let file = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![mission(
            "kinds",
            &[],
            vec![objective(Kind::Species, "any", 3)],
        )],
    };
    let chain = file.resolve(&shipped_catalog(&shipped_book())).unwrap();
    let none = |_: &Objective| 0;
    let mut progress = Progress::default();
    for name in ["minnow", "minnow", "perch", "minnow"] {
        progress.record(&chain, &Happened::new(Kind::Species, name, 1), &none);
    }
    assert_eq!(progress.counts(chain.get("kinds").unwrap()), [2]);
    assert_eq!(progress.seen["kinds"].len(), 2);
    let step = progress.record(&chain, &Happened::new(Kind::Species, "pike", 1), &none);
    assert_eq!(step.done, ["kinds"]);
    assert!(progress.seen.is_empty(), "the names close with the mission");
    // A record from before `seen`.
    let old: Progress = ron::from_str(r#"(done: ["gather"], counts: {"kinds": [1]})"#).unwrap();
    assert!(old.seen.is_empty());
    assert_eq!(old.counts["kinds"], [1]);
}

/// `mission-playthrough` 3.5: one mission at a time. A mission picked on the
/// tree is the one the HUD shows; when it is done the focus goes on to the
/// next of its branch, not back to the tree's first open mission.
#[test]
fn the_focus_is_the_one_mission_shown_and_goes_on_down_its_branch() {
    let chain = shipped_file()
        .resolve(&shipped_catalog(&shipped_book()))
        .unwrap();
    let none = |_: &Objective| 0;
    let mut progress = Progress::default();
    for id in [
        "gather",
        "axe",
        "wood",
        "planks-rope",
        "crafting-table",
        "stone-tools",
        "fire",
    ] {
        progress.done.insert(id.to_string());
    }
    progress.settle(&chain, &none);
    // Unfocused: the tree's first open mission.
    let first = progress.current(&chain).unwrap().id.clone();
    assert_eq!(progress.focused(&chain).unwrap().id, first);
    // A locked mission cannot be picked; an open one can.
    assert!(!progress.focus_on(&chain, "smelt"));
    assert!(progress.focus_on(&chain, "dig-deep"));
    assert_eq!(progress.focused(&chain).unwrap().id, "dig-deep");
    assert_eq!(
        progress.line(&chain).as_deref(),
        Some("Dig into the hill 0/10")
    );
    // Done: on to the next of the mining branch.
    progress.record(
        &chain,
        &Happened::new(Kind::Gather, "block:stone", 10),
        &none,
    );
    assert!(progress.is_done("dig-deep"));
    progress.focus_past(&chain, "dig-deep");
    assert_eq!(progress.focused(&chain).unwrap().id, "ore");
    assert_eq!(
        progress.line(&chain).as_deref(),
        Some("Find ore: Dig ore 0/4, Dig coal 0/3")
    );
}

/// `mission-playthrough` 3.9: a record from before "Go fishing" lost its
/// cast keeps its fish.
#[test]
fn a_record_from_before_the_cast_was_dropped_keeps_its_fish() {
    let mut progress = Progress::default();
    progress.counts.insert("go-fishing".into(), vec![1, 2]);
    progress.migrate();
    assert_eq!(progress.counts["go-fishing"], vec![2]);
    // Read again: nothing more moves.
    progress.migrate();
    assert_eq!(progress.counts["go-fishing"], vec![2]);
}

// ---- Survival's missions and paths (`survival-mode` design 18).

fn shipped_chain() -> Chain {
    shipped_file()
        .resolve(&shipped_catalog(&shipped_book()))
        .unwrap()
}

/// The trunk's events, from bare hands to the fire.
fn to_the_fire() -> Vec<Happened> {
    let ev = Happened::new;
    vec![
        ev(Kind::Gather, "fibre", 8),
        ev(Kind::Gather, "stone", 9),
        ev(Kind::Gather, "block:dirt", 6),
        ev(Kind::Open, "pack", 1),
        ev(Kind::Craft, "recipe:stone-axe", 1),
        ev(Kind::Fell, "tree", 2),
        ev(Kind::Gather, "log", 10),
        ev(Kind::Craft, "planks", 8),
        ev(Kind::Craft, "rope", 2),
        ev(Kind::Craft, "block:workbench", 1),
        ev(Kind::Place, "block:workbench", 1),
        ev(Kind::Craft, "tool:pickaxe", 1),
        ev(Kind::Craft, "tool:shovel", 1),
        ev(Kind::Craft, "block:campfire", 1),
        ev(Kind::Place, "block:campfire", 1),
    ]
}

const SURVIVAL_IDS: [&str; 25] = [
    "drink",
    "forage",
    "cook-meal",
    "straw",
    "cactus",
    "still",
    "spear",
    "hunt",
    "cook-meat",
    "dress",
    "tan",
    "bow",
    "long-shot",
    "hunting-axe",
    "sword",
    "fend-off",
    "first-night",
    "warm-up",
    "dress-weather",
    "cool-down",
    "relight",
    "put-out",
    "long-swim",
    "hold-breath",
    "soft-landing",
];

/// 6.1: the shipped file's 25 survival missions are Survival's only, each
/// on its path, and an Explorer world's tree has none of them and counts
/// none of their deeds.
#[test]
fn an_explorer_world_shows_and_counts_no_survival_mission() {
    let full = shipped_chain();
    let survival: Vec<&Mission> = full
        .missions
        .iter()
        .filter(|m| m.mode == Some(Mode::Survival))
        .collect();
    let ids: Vec<&str> = survival.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, SURVIVAL_IDS);
    for m in &survival {
        assert!(
            ["eat-drink", "hunting", "shelter", "water"].contains(&m.branch.as_str()),
            "{}: {}",
            m.id,
            m.branch
        );
        // A path's head hangs off the fire; nothing hangs off the pack.
        assert_ne!(m.needs, ["pack"], "{}", m.id);
    }
    for head in ["drink", "spear", "first-night", "long-swim"] {
        assert_eq!(full.get(head).unwrap().needs, [PATHS_FROM], "{head}");
    }
    let explorer = full.for_mode(Mode::Explorer);
    assert_eq!(explorer.missions.len(), full.missions.len() - 25);
    assert!(explorer.missions.iter().all(|m| m.mode.is_none()));
    let survival_tree = full.for_mode(Mode::Survival);
    assert_eq!(survival_tree.missions.len(), full.missions.len());
    // Every deed of Survival's, in an Explorer world past the fire, counts
    // for nothing and opens nothing.
    let mut progress = Progress::default();
    for event in to_the_fire() {
        progress.record(&explorer, &event, &none);
    }
    let before = progress.clone();
    let ev = Happened::new;
    for event in [
        ev(Kind::Drink, "fresh", 3),
        ev(Kind::Kill, "yard hen", 1),
        ev(Kind::Dawn, "night", 1),
        ev(Kind::Dive, "water", 1),
        ev(Kind::Fall, "water", 1),
        ev(Kind::Warm, "fire", 1),
    ] {
        let step = progress.record(&explorer, &event, &none);
        assert!(step.done.is_empty(), "{event:?}");
    }
    assert_eq!(progress, before);
    for id in SURVIVAL_IDS {
        assert!(!progress.is_done(id) && !progress.counts.contains_key(id));
        assert!(explorer.get(id).is_none(), "{id}");
    }
    // And no picker: no cards, nothing to choose, no path taken.
    assert!(progress.path_cards(&explorer, Mode::Explorer).is_empty());
    assert!(!progress.choosing(&explorer, Mode::Explorer));
    assert!(!progress.choose_path(&explorer, Mode::Explorer, "fishing"));
    assert_eq!(progress.path, None);
}

/// 6.1: the pack leads to the axe in both modes, and no survival mission
/// opens before the fire.
#[test]
fn the_pack_leads_to_the_axe_in_both_modes() {
    let full = shipped_chain();
    for mode in [Mode::Explorer, Mode::Survival] {
        let chain = full.for_mode(mode);
        let mut progress = Progress::default();
        for event in &to_the_fire()[..4] {
            progress.record(&chain, event, &none);
        }
        assert!(progress.is_done("pack"), "{mode:?}");
        assert_eq!(
            progress.current(&chain).map(|m| m.id.as_str()),
            Some("axe"),
            "{mode:?}"
        );
        assert!(
            progress.open(&chain).all(|m| m.mode.is_none()),
            "{mode:?}: a survival mission before the fire"
        );
    }
}

/// 6.1: "Make a fire"'s ancestors hold every basic tool mission: the stone
/// axe, pickaxe and shovel are all made by the fire.
#[test]
fn the_fires_ancestors_hold_every_basic_tool() {
    let chain = shipped_chain().for_mode(Mode::Survival);
    let mut ancestors = Vec::new();
    let mut at = chain.get(PATHS_FROM).unwrap();
    while let Some(parent) = at.needs.first() {
        ancestors.push(parent.clone());
        at = chain.get(parent).unwrap();
    }
    let made: Vec<String> = ancestors
        .iter()
        .flat_map(|id| chain.get(id).unwrap().objectives.iter())
        .filter(|o| o.kind == Kind::Craft)
        .flat_map(|o| match &o.want {
            Want::Names(names) => names.clone(),
            _ => Vec::new(),
        })
        .collect();
    for tool in [
        "recipe:stone-axe",
        "recipe:stone-pickaxe",
        "recipe:stone-shovel",
    ] {
        assert!(made.iter().any(|n| n == tool), "{tool} not before the fire");
    }
}

/// 6.1: in a Survival world the fire opens every path's head: Survival's
/// four (or, while a head waits on goods this build lacks, the mission
/// after it) and the starter tree's own.
#[test]
fn a_survival_world_opens_the_paths_with_the_fire() {
    let chain = shipped_chain().for_mode(Mode::Survival);
    let mut progress = Progress::default();
    let events = to_the_fire();
    for event in &events[..events.len() - 1] {
        progress.record(&chain, event, &none);
    }
    assert!(progress.path_cards(&chain, Mode::Survival).is_empty());
    let last = events.last().unwrap();
    let step = progress.record(&chain, last, &none);
    assert!(step.done.iter().any(|d| d == PATHS_FROM));
    let open: Vec<&str> = progress.open(&chain).map(|m| m.id.as_str()).collect();
    for id in [
        "drink",
        "first-night",
        "long-swim",
        "rod",
        "find-seeds",
        "fence",
        "dig-deep",
        "build",
    ] {
        assert!(open.contains(&id), "{id} not open: {open:?}");
    }
    // The spear opens with them, unless its recipe is still to come; then
    // the hunt it leads to is open in its place.
    let spear = chain.get("spear").unwrap();
    assert!(
        open.contains(&"spear") || (spear.pending.is_some() && open.contains(&"hunt")),
        "{open:?}"
    );
    // A mission waits only on goods and recipes this build lacks, never on a
    // kind the game cannot raise.
    for (id, why) in chain.pending() {
        assert!(
            why.iter().all(|w| !w.ends_with("in the game")),
            "{id}: {why:?}"
        );
    }
    let cards = progress.path_cards(&chain, Mode::Survival);
    let branches: Vec<&str> = cards.iter().map(|c| c.branch.as_str()).collect();
    assert_eq!(
        branches,
        [
            "eat-drink",
            "hunting",
            "fishing",
            "farming",
            "shelter",
            "water",
            "herding",
            "mining",
            "light"
        ]
    );
    assert!(cards.iter().all(|c| c.first.is_some() && c.done == 0));
    assert_eq!(
        cards[0].first.as_deref(),
        Some("drink"),
        "Eat and drink starts with a drink"
    );
    assert!(progress.choosing(&chain, Mode::Survival));
    assert_eq!(
        progress.offer_after(&chain, Mode::Survival, &step.done),
        Some(Offer::FireLit)
    );
    assert_eq!(
        progress.offer_after(&chain, Mode::Explorer, &step.done),
        None
    );
}

/// 6.2: each of Survival's kinds counts its own deed and names, and nothing
/// else; a sip of salt water names nothing a mission may count.
#[test]
fn each_survival_kind_counts_its_own_deed() {
    let rows = [
        (Kind::Drink, "fresh|rain|bucket|still|cactus"),
        (Kind::Kill, "yard hen|long-shot"),
        (Kind::Dawn, "night"),
        (Kind::Cool, "heat"),
        (Kind::Warm, "fire"),
        (Kind::Wear, "wool"),
        (Kind::Dive, "water"),
        (Kind::Fall, "water"),
        (Kind::DriveOff, "predator"),
        (Kind::Relight, "torch"),
        (Kind::Douse, "fire"),
    ];
    for (kind, names) in rows {
        assert!(Kind::LIVE.contains(&kind), "{kind:?} not live");
        let file = MissionFile {
            missions: vec![mission("m", &[], vec![objective(kind, names, 2)])],
            ..Default::default()
        };
        let chain = file.resolve(&shipped_catalog(&shipped_book())).unwrap();
        assert!(chain.get("m").unwrap().pending.is_none(), "{kind:?}");
        let first = names.split('|').next().unwrap();
        let mut progress = Progress::default();
        // Another kind with the same name, and this kind with another name,
        // count nothing.
        let other = if kind == Kind::Dive {
            Kind::Swim
        } else {
            Kind::Dive
        };
        progress.record(&chain, &Happened::new(other, first, 1), &none);
        progress.record(&chain, &Happened::new(kind, "elsewhere", 1), &none);
        assert!(progress.counts.is_empty(), "{kind:?}");
        progress.record(&chain, &Happened::new(kind, first, 1), &none);
        assert_eq!(progress.counts["m"], [1], "{kind:?}");
        let step = progress.record(&chain, &Happened::new(kind, first, 1), &none);
        assert_eq!(step.done, ["m"], "{kind:?}");
    }
    // Salt water is never a drink: no objective may name it.
    let file = MissionFile {
        missions: vec![mission("sea", &[], vec![objective(Kind::Drink, "sea", 1)])],
        ..Default::default()
    };
    assert!(file.resolve(&shipped_catalog(&shipped_book())).is_err());
    // The drink mission counts fresh water, rain, a fresh bucket and the
    // still, and not a cactus (its own mission's) or anything salt.
    let chain = shipped_chain();
    let drink = &chain.get("drink").unwrap().objectives[0];
    for name in ["fresh", "rain", "bucket", "still"] {
        assert!(drink.want.matches(Kind::Drink, name), "{name}");
    }
    for name in ["cactus", "sea", "salt"] {
        assert!(!drink.want.matches(Kind::Drink, name), "{name}");
    }
    // The survival rules' water sources: salt names nothing, so no sip of it
    // is ever raised or counted.
    use crate::survival::water::WaterSource as W;
    for (source, named) in [
        (W::Fresh, Some("fresh")),
        (W::Bucket { salt: false }, Some("bucket")),
        (W::Still, Some("still")),
        (W::Cactus, Some("cactus")),
        (W::Sea, None),
        (W::Brackish, None),
        (W::Bucket { salt: true }, None),
    ] {
        assert_eq!(drink_name(source), named, "{source:?}");
    }
}

/// A tree of paths for the picker: the fire, then `a` (two missions), `b`
/// (whose second waits on `c`'s), and `c`.
fn path_chain() -> Chain {
    let mut rows = vec![
        mission(
            "fire",
            &[],
            vec![objective(Kind::Place, "block:campfire", 1)],
        ),
        mission("a1", &["fire"], vec![objective(Kind::Drink, "fresh", 1)]),
        mission("a2", &["a1"], vec![objective(Kind::Dawn, "night", 1)]),
        mission("b1", &["fire"], vec![objective(Kind::Dive, "water", 1)]),
        mission("b2", &["b1"], vec![objective(Kind::Fall, "water", 1)]),
        mission("c1", &["fire"], vec![objective(Kind::Douse, "fire", 1)]),
    ];
    rows[4].after = vec!["c1".into()];
    for (row, branch) in rows.iter_mut().zip(["trunk", "a", "a", "b", "b", "c"]) {
        row.branch = branch.into();
    }
    MissionFile {
        missions: rows,
        ..Default::default()
    }
    .resolve(&shipped_catalog(&shipped_book()))
    .unwrap()
}

/// 6.5: a chosen path's first open mission is Now and the HUD's goal; when
/// it is done Now moves down the path; a finished path leaves the picker
/// and offers it again; a path whose next waits keeps a card naming what it
/// waits on; with no path open Now falls back to the file's first open one.
#[test]
fn a_path_is_now_until_it_runs_out() {
    let chain = path_chain();
    let ev = Happened::new;
    let s = Mode::Survival;
    let mut p = Progress::default();
    let step = p.record(&chain, &ev(Kind::Place, "block:campfire", 1), &none);
    assert_eq!(p.offer_after(&chain, s, &step.done), Some(Offer::FireLit));
    // Before a pick: the first open mission, and a choice due.
    assert_eq!(p.focused(&chain).map(|m| m.id.as_str()), Some("a1"));
    assert!(p.choosing(&chain, s));
    // A path with nothing open cannot be picked.
    assert!(!p.choose_path(&chain, s, "nowhere"));
    // Picking `b` makes its first mission Now, over the file's order and
    // over a focus picked on the tree.
    p.focus = Some("a1".into());
    assert!(p.choose_path(&chain, s, "b"));
    assert_eq!(p.focus, None);
    assert_eq!(p.path_now(&chain).map(|m| m.id.as_str()), Some("b1"));
    assert_eq!(p.focused(&chain).map(|m| m.id.as_str()), Some("b1"));
    assert!(!p.choosing(&chain, s));
    // The others still count.
    p.record(&chain, &ev(Kind::Drink, "fresh", 1), &none);
    assert!(p.is_done("a1"));
    assert_eq!(p.focused(&chain).map(|m| m.id.as_str()), Some("b1"));
    // `b1` done: `b2` waits on `c1`, so the path has nothing open; the
    // picker is offered again and `b` keeps a card that names `c1`.
    let step = p.record(&chain, &ev(Kind::Dive, "water", 1), &none);
    assert_eq!(step.done, ["b1"]);
    assert_eq!(p.path_now(&chain), None);
    assert_eq!(
        p.offer_after(&chain, s, &step.done),
        Some(Offer::PathWaits("b".into(), "c1".into()))
    );
    let cards = p.path_cards(&chain, s);
    let b = cards.iter().find(|c| c.branch == "b").unwrap();
    assert_eq!(
        (b.first.as_deref(), b.waits_on.as_deref()),
        (None, Some("c1"))
    );
    assert_eq!((b.done, b.total), (1, 2));
    assert!(p.choosing(&chain, s));
    // Meanwhile Now falls back to the first open mission in the file.
    assert_eq!(p.focused(&chain).map(|m| m.id.as_str()), Some("a2"));
    // Pick `a`; finish it: the path is done, left off the picker.
    assert!(p.choose_path(&chain, s, "a"));
    let step = p.record(&chain, &ev(Kind::Dawn, "night", 1), &none);
    assert_eq!(
        p.offer_after(&chain, s, &step.done),
        Some(Offer::PathDone("a".into()))
    );
    let cards = p.path_cards(&chain, s);
    assert!(cards.iter().all(|c| c.branch != "a"), "{cards:?}");
    // `c1` done opens `b2` again; a deed off the chosen path offers nothing.
    let step = p.record(&chain, &ev(Kind::Douse, "fire", 1), &none);
    assert_eq!(step.done, ["c1"]);
    assert_eq!(p.offer_after(&chain, s, &step.done), None);
    assert!(p.choose_path(&chain, s, "b"));
    assert_eq!(p.focused(&chain).map(|m| m.id.as_str()), Some("b2"));
    // All done: no cards, nothing to choose, no goal.
    p.record(&chain, &ev(Kind::Fall, "water", 1), &none);
    assert!(p.path_cards(&chain, s).is_empty());
    assert!(!p.choosing(&chain, s));
    assert_eq!(p.focused(&chain), None);
}

/// 6.5: the chosen path is saved in the `missions` record (schema 2), and a
/// schema 1 record, which has none, reads as no path.
#[test]
fn the_path_is_saved_and_an_older_record_reads_none() {
    assert_eq!(MISSIONS_SCHEMA, 2);
    let old = r#"{"done":["gather","pack"],"counts":{"axe":[0]},"focus":"axe"}"#;
    let read: Progress = serde_json::from_str(old).unwrap();
    assert_eq!(read.path, None);
    assert_eq!(read.focus.as_deref(), Some("axe"));
    // A record with no path writes none, as schema 1 did.
    assert!(!serde_json::to_string(&read).unwrap().contains("path"));
    let mut chosen = read.clone();
    chosen.path = Some("hunting".into());
    let back: Progress = serde_json::from_str(&serde_json::to_string(&chosen).unwrap()).unwrap();
    assert_eq!(back.path.as_deref(), Some("hunting"));
}

/// A mission every world has may not wait on one of a single mode, so an
/// Explorer tree is whole without Survival's.
#[test]
fn a_mission_for_every_world_never_waits_on_one_mode() {
    let mut rows = vec![
        mission("a", &[], vec![objective(Kind::Dawn, "night", 1)]),
        mission("b", &["a"], vec![objective(Kind::Dawn, "night", 1)]),
    ];
    rows[0].mode = Some(Mode::Survival);
    let file = MissionFile {
        missions: rows,
        ..Default::default()
    };
    let err = file.resolve(&shipped_catalog(&shipped_book())).unwrap_err();
    assert!(err.contains("another mode"), "{err}");
}
#[test]
fn zz_print_pending() {
    for (id, why) in shipped_chain().pending() {
        println!("PENDING {id}: {why:?}");
    }
}
