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
    let fish: Vec<&str> = fauna
        .bodies
        .values()
        .flatten()
        .map(|s| s.name.as_str())
        .collect();
    let animals: Vec<&str> = land
        .land
        .values()
        .flatten()
        .map(|s| s.name.as_str())
        .collect();
    Catalog::new(book, fish, animals)
}

fn shipped_file() -> MissionFile {
    ron::from_str(include_str!("../../../../assets/config/missions.ron")).unwrap()
}

fn objective(kind: Kind, what: &str, count: u32) -> ObjectiveEntry {
    ObjectiveEntry {
        kind,
        what: what.into(),
        count,
        label: what.into(),
    }
}

fn mission(id: &str, needs: &[&str], objectives: Vec<ObjectiveEntry>) -> MissionEntry {
    MissionEntry {
        id: id.into(),
        title: id.into(),
        help: String::new(),
        needs: needs.iter().map(|n| n.to_string()).collect(),
        objectives,
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

    // The shipped chain and an old world (before tool crafting) that owns
    // all four tools at the stone tier and has a campfire placed: the
    // missions it holds close, and those before them with them (the axe's
    // logs too, since the rod after it is held), so it is told to go
    // fishing, not to gather by hand.
    let chain = shipped_file().resolve(&shipped_catalog(&book)).unwrap();
    let tools = ["tool:rod", "tool:shovel", "tool:pickaxe", "tool:axe"];
    let held = |o: &Objective| match o.kind {
        Kind::Craft => tools
            .iter()
            .chain(["block:campfire"].iter())
            .filter(|t| o.want.matches(Kind::Craft, t))
            .count() as u32,
        Kind::Place => u32::from(o.want.matches(Kind::Place, "block:campfire")),
        _ => 0,
    };
    let mut progress = Progress::default();
    let step = progress.settle(&chain, &held);
    for id in [
        "gather",
        "planks-rope",
        "crafting-table",
        "mining-tools",
        "axe",
        "rod",
        "fire",
    ] {
        assert!(progress.is_done(id), "{id} in {:?}", step.done);
    }
    assert_eq!(progress.line(&chain).as_deref(), Some("Go fishing 0/1"));
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
                vec![objective(Kind::Craft, "block:torch", 1)],
            ),
            mission("fell", &["pick"], vec![objective(Kind::Fell, "tree", 1)]),
            mission("cast", &["fell"], vec![objective(Kind::Cast, "water", 1)]),
        ],
    };
    let chain = file.resolve(&catalog).unwrap();
    let pending: Vec<&str> = chain.pending().map(|(id, _)| id).collect();
    assert_eq!(pending, ["planks", "pick", "fell"]);
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
    assert_eq!(read.line(&chain).as_deref(), Some("Catch fish 2/3"));
}

/// The shipped chain: every `what` names a known good, recipe, block,
/// species or action, pending rows excepted, and on this build (before
/// `tool-crafting`) it plays from the first mission to the last.
#[test]
fn the_shipped_chain_names_only_what_the_build_knows() {
    let book = shipped_book();
    let chain = shipped_file().resolve(&shipped_catalog(&book)).unwrap();
    assert_eq!(chain.missions.len(), 19);
    // Nothing is pending since `tool-crafting` merged: a new world starts
    // at the first mission.
    assert!(
        chain.pending().next().is_none(),
        "{:?}",
        chain.pending().collect::<Vec<_>>()
    );
    let progress = Progress::default();
    assert_eq!(
        progress.current(&chain).map(|m| m.id.as_str()),
        Some("gather")
    );
    // Every live objective resolved to something it can count.
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
    // Walk it from "Go fishing" to the end with the events the game raises.
    let mut progress = Progress::default();
    let events = [
        Happened::new(Kind::Gather, "block:dirt", 4),
        Happened::new(Kind::Gather, "fibre", 3),
        Happened::new(Kind::Gather, "log", 1),
        Happened::new(Kind::Craft, "planks", 4),
        Happened::new(Kind::Craft, "rope", 1),
        Happened::new(Kind::Craft, "block:workbench", 1),
        Happened::new(Kind::Place, "block:workbench", 1),
        Happened::new(Kind::Craft, "tool:pickaxe", 1),
        Happened::new(Kind::Craft, "tool:shovel", 1),
        Happened::new(Kind::Craft, "recipe:stone-axe", 1),
        Happened::new(Kind::Gather, "log", 3),
        Happened::new(Kind::Craft, "tool:rod", 1),
        Happened::new(Kind::Craft, "block:campfire", 1),
        Happened::new(Kind::Place, "block:campfire", 1),
        Happened::new(Kind::Cast, "water", 1),
        Happened::new(Kind::Catch, "minnow", 3),
        Happened::new(Kind::Cook, "grilled-fish", 3),
        Happened::new(Kind::Find, "wheat-seed", 2),
        Happened::new(Kind::Find, "turnip-seed", 2),
        Happened::new(Kind::Till, "any", 4),
        Happened::new(Kind::Plant, "wheat-seed", 4),
        Happened::new(Kind::Tame, "yard hen", 1),
        Happened::new(Kind::BuildPen, "any", 1),
        Happened::new(Kind::Pen, "yard hen", 1),
        Happened::new(Kind::Place, "block:forge", 1),
        Happened::new(Kind::Smelt, "copper-ingot", 2),
        Happened::new(Kind::Smelt, "iron-ingot", 1),
        Happened::new(Kind::Craft, "recipe:bronze-axe", 1),
        Happened::new(Kind::Collect, "egg", 1),
        Happened::new(Kind::Harvest, "wheat", 4),
        Happened::new(Kind::Cook, "flatbread", 1),
        Happened::new(Kind::Cook, "fried-egg", 1),
    ];
    let mut done = Vec::new();
    for event in &events {
        done.extend(progress.record(&chain, event, &none).done);
    }
    assert_eq!(
        done,
        [
            "gather",
            "planks-rope",
            "crafting-table",
            "mining-tools",
            "axe",
            "rod",
            "fire",
            "go-fishing",
            "catch-fish",
            "cook-fish",
            "find-seeds",
            "plant",
            "tame",
            "pen",
            "smelt",
            "metal-tool",
            "collect",
            "harvest",
            "farm-meal"
        ]
    );
    assert_eq!(progress.current(&chain), None);
}
