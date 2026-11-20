use super::*;
use crate::station::RecipeFile;

/// The shipped recipe book: the forge's and the kitchen's.
fn shipped_book() -> RecipeBook {
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
fn shipped_catalog(book: &RecipeBook) -> Catalog {
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
        Some("mining-tools: tool:shovel 0/1")
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
    assert_eq!(
        progress.line(&chain).as_deref(),
        Some("Go fishing: cast 0/1")
    );
}

/// A name the build lacks that the file awaits makes its mission pending,
/// and the missions after it still open; any other unknown name is refused.
#[test]
fn an_awaited_name_makes_a_mission_pending() {
    let book = shipped_book();
    let catalog = shipped_catalog(&book);
    let file = MissionFile {
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
    // On the tree, the old line's done ids stay done; the trunk's new step
    // after the axe is next.
    assert_eq!(read.line(&chain).as_deref(), Some("Gather wood: trees 0/2"));
}

/// The shipped tree (`starter-missions` TS.1, M33 version 3): every `what`
/// names a known good, recipe, block, species or action, every `unlocks` a
/// recipe; it is a strict tree (one parent each, `after` gates naming
/// missions) reachable from `gather`; and played with the events the game
/// raises it closes every mission whose kind is live.
#[test]
fn the_shipped_chain_names_only_what_the_build_knows() {
    let book = shipped_book();
    let chain = shipped_file().resolve(&shipped_catalog(&book)).unwrap();
    assert_eq!(chain.missions.len(), 42);
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
        ev(Kind::Reach, "reef", 1),
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
    // On the tree: the axe after the gathering is next.
    assert_eq!(progress.current(&chain).map(|m| m.id.as_str()), Some("axe"));
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
