use super::*;
use crate::economy::coins::PURSE_SCHEMA;
use crate::missions::{
    Catalog, Happened, Kind, MissionEntry, MissionFile, Objective, ObjectiveEntry, Standing,
    recipe_open_in,
};
use crate::records::Record;
use crate::station::{RecipeBook, RecipeFile};

fn shipped_book() -> RecipeBook {
    let mut book =
        ron::from_str::<RecipeFile>(include_str!("../../../../../assets/config/recipes.ron"))
            .unwrap()
            .resolve()
            .unwrap();
    let kitchen = ron::from_str::<crate::cooking::CookingFile>(include_str!(
        "../../../../../assets/config/cooking.ron"
    ))
    .unwrap()
    .resolve()
    .unwrap();
    book.recipes.extend(kitchen.recipes);
    book
}

/// The shipped rosters' names: every body's fish and land animals.
fn catalog(book: &RecipeBook) -> Catalog {
    let fauna: crate::fauna::FaunaSettings =
        ron::from_str(include_str!("../../../../../assets/config/fauna.ron")).unwrap();
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

fn town_file() -> MissionFile {
    ron::from_str(include_str!(
        "../../../../../assets/config/town_missions.ron"
    ))
    .unwrap()
}

fn starter_file() -> MissionFile {
    ron::from_str(include_str!("../../../../../assets/config/missions.ron")).unwrap()
}

fn town_chain() -> Chain {
    town_file()
        .resolve_keyed(&catalog(&shipped_book()), "town_mission")
        .unwrap()
}

fn none(_: &Objective) -> u32 {
    0
}

/// Task 2.1: the shipped town tree loads, is M33's 25 missions in a strict
/// tree with one root, every unlock a recipe or one the file awaits, every
/// mission reachable from the root, and no mission of it named by the
/// starter tree, or the other way round.
#[test]
fn the_shipped_town_tree_is_a_strict_tree_of_its_own() {
    let book = shipped_book();
    let file = town_file();
    let chain = town_chain();
    assert_eq!(chain.missions.len(), 34);
    let roots: Vec<&str> = chain
        .missions
        .iter()
        .filter(|m| m.needs.is_empty())
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(roots, ["meet"]);
    for m in &chain.missions {
        assert!(m.needs.len() <= 1, "{} has one parent", m.id);
        assert!(m.after.is_empty(), "{}: no hidden gates", m.id);
        // Reachable: its parents lead to the root.
        let mut at = m;
        let mut steps = 0;
        while let Some(p) = at.needs.first() {
            at = chain.get(p).expect("a parent in the tree");
            steps += 1;
            assert!(steps < 30, "{} loops", m.id);
        }
        assert_eq!(at.id, "meet", "{} reaches the root", m.id);
        for code in &m.unlocks {
            assert!(
                book.get(code).is_some() || file.awaited.contains(code),
                "{}: unlocks {code}, neither a recipe nor awaited",
                m.id
            );
        }
        assert!(m.coins > 0, "{} pays", m.id);
    }
    assert_eq!(
        chain
            .missions
            .iter()
            .map(|m| u64::from(m.coins))
            .sum::<u64>(),
        640
    );
    // The two trees never name each other.
    let starter = starter_file();
    for m in &file.missions {
        for need in &m.needs {
            assert!(
                file.missions.iter().any(|n| &n.id == need),
                "{}: needs {need}, not in the town tree",
                m.id
            );
        }
    }
    for m in &starter.missions {
        for need in m.needs.iter().chain(&m.after) {
            assert!(
                starter.missions.iter().any(|n| &n.id == need),
                "starter {} names {need}",
                m.id
            );
        }
    }
}

/// What reads "not yet" in this build: the roads, each for what it waits
/// on (the carts are built, `land-transport` design 11). The rest is live,
/// and the pending ones block no live branch.
#[test]
fn the_roads_carts_and_lamps_wait_and_block_nothing_else() {
    let chain = town_chain();
    let pending: Vec<&str> = chain.pending().map(|(id, _)| id).collect();
    // The paving (`townsfolk` design 45) and the carts (`land-transport`
    // design 11) are built: nothing waits.
    assert!(pending.is_empty(), "{pending:?}");
    let live = [
        "meet",
        "first-trade",
        "first-job",
        "trade-more",
        "deliver",
        "goodwill",
        "three-jobs",
        "lost",
        "mend",
        "lamps",
        "house",
        "logs",
        "grinder",
        "crush",
        "mixer",
        "gravel",
        "mark",
        "pave",
        "join",
        "wheels",
        "hand-cart",
        "haul",
        "horse",
        "horse-cart",
        "road-trip",
    ];
    for id in live {
        assert!(chain.get(id).unwrap().pending.is_none(), "{id} is live");
    }
    // With the trunk done, every live branch opens.
    let mut progress = Progress::default();
    for id in ["meet", "first-trade", "first-job"] {
        progress.done.insert(id.into());
    }
    for id in ["trade-more", "three-jobs", "house"] {
        let m = chain.get(id).unwrap();
        assert_eq!(progress.standing(&chain, m), Standing::Open, "{id}");
    }
}

/// Every town mission's title, help and objective labels are keys in the
/// locale file, under `town_mission.`.
#[test]
fn every_town_mission_has_its_words() {
    let english = crate::locale::Locale::english();
    for m in &town_file().missions {
        for part in ["title", "help"] {
            let key = format!("town_mission.{}.{part}", m.id);
            assert!(english.get(&key).is_some(), "{key}");
        }
        for k in 0..m.objectives.len() {
            let key = format!("town_mission.{}.objective.{k}", m.id);
            assert!(english.get(&key).is_some(), "{key}");
        }
    }
}

/// Task 2.4: `any:` counts in every town; a talk counts a person once; a
/// job, a goodwill step, a house piece and a log at rest each count from
/// their event.
#[test]
fn each_town_kind_counts_from_its_event() {
    let chain = town_chain();
    let mut p = Progress::default();
    let talk =
        |p: &mut Progress, who: &str| p.record(&chain, &Happened::new(Kind::Talk, who, 1), &none);
    talk(&mut p, "21:0");
    talk(&mut p, "21:0");
    talk(&mut p, "21:1");
    assert_eq!(p.counts(chain.get("meet").unwrap()), [2], "one person once");
    let step = talk(&mut p, "34:0");
    assert_eq!(step.done, ["meet"]);
    // A sale in any town counts; a sale elsewhere than a town does not.
    for site in [21, 34, 7, 8, 9] {
        p.record(
            &chain,
            &Happened::new(Kind::Sell, format!("{site}:fish:perch"), 1),
            &none,
        );
    }
    p.record(&chain, &Happened::new(Kind::Sell, "fish", 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Buy, "21:flatbread", 1), &none);
    assert_eq!(step.done, ["first-trade"]);
    let step = p.record(&chain, &Happened::new(Kind::Job, "fish", 1), &none);
    assert_eq!(step.done, ["first-job"]);
    // The house: walls, a door and a roof.
    for _ in 0..6 {
        p.record(&chain, &Happened::new(Kind::Build, "wall", 1), &none);
    }
    p.record(&chain, &Happened::new(Kind::Build, "floor", 1), &none);
    p.record(&chain, &Happened::new(Kind::Build, "door", 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Build, "roof", 1), &none);
    assert_eq!(step.done, ["house"]);
    for _ in 0..4 {
        p.record(&chain, &Happened::new(Kind::Float, "log", 1), &none);
    }
    assert!(p.is_done("logs"));
    // Trade, a parcel and goodwill.
    p.record(&chain, &Happened::new(Kind::Sell, "21:stone", 20), &none);
    p.record(&chain, &Happened::new(Kind::Deliver, "34:parcel", 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Goodwill, "34:step", 1), &none);
    assert_eq!(step.done, ["goodwill"]);
    // Jobs, a lost villager, a mended wall and lamps lit at dusk.
    p.record(&chain, &Happened::new(Kind::Job, "deliver", 3), &none);
    p.record(
        &chain,
        &Happened::new(Kind::Escort, "21:villager", 1),
        &none,
    );
    let step = p.record(&chain, &Happened::new(Kind::Repair, "21:piece", 1), &none);
    assert_eq!(step.done, ["mend"]);
    for site in [21, 21, 34, 34] {
        p.record(
            &chain,
            &Happened::new(Kind::Light, format!("{site}:lamp"), 1),
            &none,
        );
    }
    assert!(p.is_done("lamps"));
    // The roads (`townsfolk` design 45): the stations made and set down,
    // stone crushed, gravel mixed, a plan staked and marked, paved, and a
    // paved cell beside a town.
    for (kind, what, count) in [
        (Kind::Craft, "block:grinder", 1),
        (Kind::Place, "block:grinder", 1),
        (Kind::Craft, "recipe:crushed-stone", 9),
        (Kind::Craft, "block:mixer", 1),
        (Kind::Place, "block:mixer", 1),
        (Kind::Craft, "recipe:mixed-gravel", 8),
        (Kind::Stake, "stake", 2),
        (Kind::Mark, "cell", 8),
        (Kind::Pave, "gravel", 8),
        (Kind::Pave, "joined", 1),
    ] {
        p.record(&chain, &Happened::new(kind, what, count), &none);
    }
    assert!(p.is_done("join"));
    // The carts (`land-transport` design 11): wheels, a hand cart set down
    // and hauling, a horse bought and hitched, and a drive on the road.
    p.record(&chain, &Happened::new(Kind::Craft, "wheel", 2), &none);
    p.record(&chain, &Happened::new(Kind::Craft, "hand-cart", 1), &none);
    p.record(&chain, &Happened::new(Kind::Place, "hand-cart", 1), &none);
    p.record(&chain, &Happened::new(Kind::Haul, "cart", 100), &none);
    p.record(&chain, &Happened::new(Kind::Buy, "21:horse", 1), &none);
    p.record(&chain, &Happened::new(Kind::Craft, "horse-cart", 1), &none);
    p.record(&chain, &Happened::new(Kind::Hitch, "horse", 1), &none);
    let step = p.record(
        &chain,
        &Happened::new(Kind::Travel, "cart-road", 500),
        &none,
    );
    assert_eq!(step.done, ["road-trip"]);
    // Carpentry (`furniture` design 12): the bench, a chair, a seat, a
    // table, a bed, a lie-down, a chest, four pieces placed, a town chest.
    let bench = "block:carpenter's bench";
    p.record(&chain, &Happened::new(Kind::Craft, bench, 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Place, bench, 1), &none);
    assert_eq!(step.done, ["carpenter"]);
    p.record(&chain, &Happened::new(Kind::Craft, "chair", 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Sit, "stool", 1), &none);
    assert_eq!(step.done, ["sit"]);
    p.record(&chain, &Happened::new(Kind::Craft, "table", 1), &none);
    p.record(&chain, &Happened::new(Kind::Craft, "bed", 1), &none);
    let step = p.record(&chain, &Happened::new(Kind::Lie, "bed", 1), &none);
    assert_eq!(step.done, ["lie"]);
    p.record(&chain, &Happened::new(Kind::Craft, "chest", 1), &none);
    for _ in 0..4 {
        p.record(&chain, &Happened::new(Kind::Build, "furniture", 1), &none);
    }
    assert!(p.is_done("furnish"));
    let step = p.record(&chain, &Happened::new(Kind::Loot, "21:planks", 3), &none);
    assert_eq!(step.done, ["find"]);
    // Every mission is done, and nothing is open.
    assert_eq!(p.current(&chain), None);
    assert_eq!(p.done.len(), 34);
}

/// Task 2.3: a mission's coins are paid by the count that finishes it, and
/// a record read back (a reload) neither pays it again nor undoes it.
#[test]
fn coins_are_paid_once() {
    let chain = town_chain();
    let mut record = TownRecord {
        opened: 3,
        progress: Progress::default(),
    };
    let mut purse = Purse::new(7);
    let mut paid = 0;
    for who in ["1:0", "1:1", "1:2", "1:2", "1:3"] {
        let step = record
            .progress
            .record(&chain, &Happened::new(Kind::Talk, who, 1), &none);
        paid += pay(&chain, &step.done, &mut purse);
    }
    assert_eq!(paid, 5);
    assert_eq!(purse.coins, 12);
    // Stored and read back, as the save does.
    let line = Record::of(
        TOWN_MISSIONS_KIND,
        TOWN_MISSIONS_ID,
        TOWN_MISSIONS_SCHEMA,
        &record,
    );
    let purse_line = Record::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
    // A kind with an underscore reads back from its journal line, as the
    // town's other kinds (`town_jobs`, `lost_villager`) now do too.
    for kind in [TOWN_MISSIONS_KIND, JOBS_KIND, LOST_KIND, "town_market"] {
        let mut r = line.clone();
        r.kind = kind.into();
        let (_, parsed) = Record::parse_line(&r.line(&crate::records::Author::Player(0)))
            .unwrap_or_else(|| panic!("{kind} reads back"));
        assert_eq!(parsed, r);
    }
    let mut back: TownRecord = line.read().unwrap();
    let mut purse: Purse = purse_line.read().unwrap();
    assert_eq!(back, record);
    let step = back.progress.settle(&chain, &none);
    assert!(step.done.is_empty());
    let step = back
        .progress
        .record(&chain, &Happened::new(Kind::Talk, "1:9", 1), &none);
    assert_eq!(pay(&chain, &step.done, &mut purse), 0);
    assert_eq!(purse.coins, 12, "never paid twice");
    assert!(back.progress.is_done("meet"));
}

/// Task 2.5: a recipe a town mission names is locked until it is done,
/// asking both trees, and a world holding its output keeps it open.
#[test]
fn a_town_unlock_waits_for_its_mission_across_both_trees() {
    let mut book = shipped_book();
    let mut mixer = book.get("workbench").unwrap().clone();
    mixer.code = "mixer".into();
    book.recipes.push(mixer);
    let entry = |id: &str, needs: &[&str], unlocks: &[&str]| MissionEntry {
        id: id.into(),
        title: Some(id.into()),
        help: None,
        needs: needs.iter().map(|s| s.to_string()).collect(),
        objectives: vec![ObjectiveEntry {
            kind: Kind::Talk,
            what: "any".into(),
            count: 1,
            label: Some(id.into()),
        }],
        unlocks: unlocks.iter().map(|s| s.to_string()).collect(),
        after: vec![],
        branch: String::new(),
        icon: String::new(),
        coins: 10,
    };
    let town = MissionFile {
        crew: None,
        awaited: vec![],
        missions: vec![entry("crush", &[], &["mixer"])],
    }
    .resolve(&catalog(&book))
    .unwrap();
    let starter = starter_file().resolve(&catalog(&book)).unwrap();
    let (sp, mut tp) = (Progress::default(), Progress::default());
    let open =
        |tp: &Progress, holds| recipe_open_in(&[(&starter, &sp), (&town, tp)], "mixer", &[], holds);
    assert!(!open(&tp, false), "locked until Crush stone");
    assert!(open(&tp, true), "a world holding a mixer keeps it");
    // A recipe neither tree names is open; one the starter names follows
    // the starter tree alone.
    let neither = book
        .recipes
        .iter()
        .find(|r| !starter.names(&r.code) && !town.names(&r.code))
        .unwrap();
    assert!(recipe_open_in(
        &[(&starter, &sp), (&town, &tp)],
        &neither.code,
        &[],
        false
    ));
    assert!(!recipe_open_in(
        &[(&starter, &sp), (&town, &tp)],
        "stone-axe",
        &[],
        false
    ));
    tp.done.insert("crush".into());
    assert!(open(&tp, false), "open once Crush stone is done");
}

/// Task 2.6's rule: what a world holds that shows a town was visited.
#[test]
fn an_old_world_shows_a_town_visit() {
    let mut records = Records::new();
    assert!(!been_to_town(&records, &Progress::default()));
    let mut starter = Progress::default();
    starter.done.insert(STARTER_TOWN.into());
    assert!(been_to_town(&records, &starter));
    records.put(Record::of(
        PURSE_KIND,
        PURSE_ID,
        PURSE_SCHEMA,
        &Purse::new(0),
    ));
    assert!(been_to_town(&records, &Progress::default()));
    let mut records = Records::new();
    records.put(Record::of(
        GOODWILL_KIND,
        21,
        1,
        &crate::economy::goodwill::Goodwill { points: 2 },
    ));
    assert!(been_to_town(&records, &Progress::default()));
    let mut records = Records::new();
    records.put(Record::of(JOBS_KIND, JOBS_ID, 1, &TakenJobs::default()));
    assert!(
        !been_to_town(&records, &Progress::default()),
        "no job taken"
    );
}
