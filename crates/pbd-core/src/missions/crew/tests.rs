use super::*;
use crate::building::{PIECE_RECORD, Piece, PieceBody, Shape, placed, removed};
use crate::missions::tests::{shipped_book, shipped_catalog};
use crate::records::Author;

fn shipped_file() -> MissionFile {
    ron::from_str(include_str!("../../../../../assets/config/missions.ron")).unwrap()
}

/// The shipped crew chain, resolved.
fn house() -> CrewChain {
    let file = shipped_file();
    let crew = file.crew.as_ref().expect("missions.ron has a crew section");
    crew.resolve(&shipped_catalog(&shipped_book()), &file.awaited)
        .expect("the crew chain resolves")
}

fn both() -> Vec<Hand> {
    vec![
        Hand {
            player: 0,
            since_s: 0.0,
        },
        Hand {
            player: 1,
            since_s: 0.0,
        },
    ]
}

fn dig(n: u32) -> Happened {
    Happened::new(Kind::Gather, "block:dirt", n)
}

/// The record read back from its journal line, as a reload reads it.
fn reload(progress: &CrewProgress) -> CrewProgress {
    let line = progress.record().line(&Author::Player(1));
    let (_, record) = crate::records::Record::parse_line(&line).expect("parses");
    let mut records = Records::new();
    records.put(record);
    CrewProgress::from_records(&records).expect("reads")
}

/// The shipped chain: H1 to H4, H6 and H7 in order, every word in the
/// locale file, its crew rules as design 5 sets them.
#[test]
fn the_shipped_house_chain_resolves_with_its_words() {
    let crew = house();
    let ids: Vec<&str> = crew.chain.missions.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "house-site",
            "house-timber",
            "house-floor",
            "house-walls",
            "house-roof",
            "house-hearth"
        ]
    );
    assert!(crew.chain.pending().next().is_none(), "nothing pending");
    assert_eq!(crew.grace_s, 300.0);
    let locale = crate::locale::Locale::english();
    for m in &crew.chain.missions {
        for key in [
            format!("{CREW_KEYS}.{}.title", m.id),
            format!("{CREW_KEYS}.{}.help", m.id),
        ]
        .into_iter()
        .chain((0..m.objectives.len()).map(|i| format!("{CREW_KEYS}.{}.objective.{i}", m.id)))
        {
            assert!(locale.get(&key).is_some(), "{key} is in the locale file");
        }
    }
    let rule = |id: &str, i: usize| crew.rules[id][i];
    assert_eq!(rule("house-site", 0), CrewRule { hands: 2, each: 0 });
    assert_eq!(rule("house-timber", 1), CrewRule { hands: 0, each: 4 });
    assert_eq!(rule("house-walls", 0), CrewRule { hands: 2, each: 1 });
    assert_eq!(
        crew.chain.get("house-walls").unwrap().objectives[0].count,
        8
    );
}

/// Task 2.1, the spec's scenario: player 0 digs 3 and player 1 digs 4
/// toward the site's 7; it is met, the shares read 3 and 4, and a reload
/// keeps both, mid-way and done.
#[test]
fn two_players_digs_count_into_one_crew_objective() {
    let crew = house();
    let mut p = CrewProgress::default();
    let site = crew.chain.get("house-site").unwrap().clone();
    for _ in 0..3 {
        p.count(&crew, 0, &dig(1), &both(), 10.0);
    }
    let step = p.count(&crew, 1, &dig(3), &both(), 10.0);
    assert!(step.changed && step.done.is_empty());
    let mid = reload(&p);
    assert_eq!(mid, p, "a reload mid-way keeps every share");
    assert_eq!(mid.total(&site, 0), 6);
    let mut p = mid;
    let step = p.count(&crew, 1, &dig(1), &both(), 11.0);
    assert_eq!(step.done, ["house-site"]);
    let shares = p.shares(&site, 0);
    assert_eq!(shares, BTreeMap::from([(0, 3), (1, 4)]));
    let back = reload(&p);
    assert_eq!(back.shares(&site, 0), shares);
    assert!(back.is_done("house-site"));
    assert_eq!(back.members, BTreeSet::from([0, 1]));
    // The next mission opened, and counts from nought.
    let timber = crew.chain.get("house-timber").unwrap();
    assert_eq!(back.standing(&crew.chain, timber), Standing::Open);
    // A dig done before the chain is in nobody's way: nothing else counts.
    assert_eq!(back.total(timber, 1), 0);
}

/// Task 2.2: one player alone cannot meet a `hands: 2` objective in a
/// crew of two, and shows that another pair of hands is needed, until the
/// second player counts toward it.
#[test]
fn one_player_cannot_meet_a_two_hands_objective_and_two_can() {
    let crew = house();
    let site = crew.chain.get("house-site").unwrap().clone();
    let mut p = CrewProgress::default();
    let step = p.count(&crew, 0, &dig(9), &both(), 5.0);
    assert!(step.done.is_empty(), "one pair of hands is not two");
    assert_eq!(p.total(&site, 0), 7);
    assert_eq!(
        p.short(&crew, &site, 0, &both(), 5.0),
        Some(Short::Hands(1))
    );
    assert_eq!(
        Short::Hands(1).text().to_string(),
        crate::locale::tr("crew.needs_hands").to_string()
    );
    // Player 0 digging on adds nothing.
    p.count(&crew, 0, &dig(1), &both(), 6.0);
    assert!(!p.is_done("house-site"));
    // The second pair of hands meets it.
    let step = p.count(&crew, 1, &dig(1), &both(), 7.0);
    assert_eq!(step.done, ["house-site"]);
    assert_eq!(p.shares(&site, 0), BTreeMap::from([(0, 7), (1, 1)]));
}

/// Design 4's "a crew chain can be finished alone": a world played by one
/// player is a crew of one, and `hands` asks no more hands than it has had.
#[test]
fn a_crew_of_one_finishes_the_chain_alone() {
    let crew = house();
    let alone = [Hand {
        player: 0,
        since_s: 0.0,
    }];
    let mut p = CrewProgress::default();
    let mut done = Vec::new();
    let deeds = [
        dig(7),
        Happened::new(Kind::Fell, "tree", 6),
        Happened::new(Kind::Gather, "log", 24),
        Happened::new(Kind::Build, "floor", 7),
        Happened::new(Kind::Build, "wall", 6),
        Happened::new(Kind::Build, "door", 1),
        Happened::new(Kind::Build, "window", 1),
        Happened::new(Kind::Build, "roof", 1),
        Happened::new(Kind::Place, "block:campfire", 1),
    ];
    for deed in &deeds {
        done.extend(p.count(&crew, 0, deed, &alone, 50.0).done);
    }
    assert_eq!(done.len(), 6, "every mission of the chain: {done:?}");
    assert_eq!(p.hands_of(&HOUSE_BUILDERS), BTreeSet::from([0]));
}

/// Task 2.2: `each` waits for every player present past the grace time,
/// and not for one who joined inside it.
#[test]
fn each_waits_for_the_late_joiners_grace() {
    let crew = house();
    let timber = crew.chain.get("house-timber").unwrap().clone();
    let late = |now_s: f64| -> (Vec<Hand>, f64) {
        (
            vec![
                Hand {
                    player: 0,
                    since_s: 0.0,
                },
                Hand {
                    player: 1,
                    since_s: 1000.0,
                },
            ],
            now_s,
        )
    };
    // The site cleared by both, so the timber is open.
    let base = {
        let mut p = CrewProgress::default();
        p.count(&crew, 0, &dig(4), &both(), 900.0);
        p.count(&crew, 1, &dig(3), &both(), 900.0);
        assert!(p.is_done("house-site"));
        p
    };
    let felled = Happened::new(Kind::Fell, "tree", 6);
    let logs = Happened::new(Kind::Gather, "log", 24);
    // Player 1 joined 100 s ago: inside the grace, so player 0 alone meets
    // it.
    let (hands, now) = late(1100.0);
    let mut p = base.clone();
    p.count(&crew, 0, &felled, &hands, now);
    let step = p.count(&crew, 0, &logs, &hands, now);
    assert_eq!(step.done, ["house-timber"]);
    // Player 1 has been in for 400 s: past it, so player 0's 24 logs are
    // not everyone's.
    let (hands, now) = late(1400.0);
    let mut p = base.clone();
    p.count(&crew, 0, &felled, &hands, now);
    let step = p.count(&crew, 0, &logs, &hands, now);
    assert!(step.done.is_empty());
    assert_eq!(
        p.short(&crew, &timber, 1, &hands, now),
        Some(Short::Each(vec![1]))
    );
    // Three logs are not four.
    p.count(
        &crew,
        1,
        &Happened::new(Kind::Gather, "log", 3),
        &hands,
        now,
    );
    assert!(!p.is_done("house-timber"));
    let step = p.count(
        &crew,
        1,
        &Happened::new(Kind::Gather, "log", 1),
        &hands,
        now,
    );
    assert_eq!(step.done, ["house-timber"]);
    assert_eq!(p.shares(&timber, 1), BTreeMap::from([(0, 24), (1, 4)]));
    // At 1290 s player 1 is 10 s inside the grace: not yet held to it.
    let (hands, now) = late(1290.0);
    let mut p = base;
    p.count(&crew, 0, &felled, &hands, now);
    p.count(&crew, 0, &logs, &hands, now);
    assert!(p.is_done("house-timber"));
}

/// Two players build the house (task 3.1, in the core until pieces go
/// through the server): every mission closes with both hands, and on the
/// hearth the house's pieces gain both as co-owners.
#[test]
fn two_players_raise_the_house_and_co_own_it() {
    let crew = house();
    let mut p = CrewProgress::default();
    let mut done: Vec<String> = Vec::new();
    let first = [
        (0, dig(4)),
        (1, dig(3)),
        (0, Happened::new(Kind::Fell, "tree", 6)),
        (0, Happened::new(Kind::Gather, "log", 12)),
        (1, Happened::new(Kind::Gather, "log", 12)),
        (0, Happened::new(Kind::Build, "floor", 4)),
        (1, Happened::new(Kind::Build, "floor", 3)),
        (0, Happened::new(Kind::Build, "wall", 6)),
    ];
    let then = [
        (1, Happened::new(Kind::Build, "door", 1)),
        (1, Happened::new(Kind::Build, "window", 1)),
        (0, Happened::new(Kind::Build, "roof", 1)),
        (1, Happened::new(Kind::Place, "block:brazier", 1)),
    ];
    for (who, h) in &first {
        done.extend(p.count(&crew, *who, h, &both(), 60.0).done);
    }
    // The walls are 6 of 8, with one hand: not yet.
    assert!(!done.contains(&"house-walls".to_string()));
    for (who, h) in &then {
        done.extend(p.count(&crew, *who, h, &both(), 60.0).done);
    }
    assert_eq!(
        done,
        [
            "house-site",
            "house-timber",
            "house-floor",
            "house-walls",
            "house-roof",
            "house-hearth"
        ]
    );
    let owners = p.hands_of(&HOUSE_BUILDERS);
    assert_eq!(owners, BTreeSet::from([0, 1]));
    // The house's pieces are co-owned, and a second pass changes nothing.
    let floor = Piece::new(
        "timber",
        Shape::Floor {
            cell: 7,
            layer: 160,
        },
    );
    let roof = Piece::new(
        "timber",
        Shape::roof(vec![7], 163, crate::building::RoofShape::Gable),
    );
    let gone = Piece::new(
        "timber",
        Shape::Floor {
            cell: 8,
            layer: 160,
        },
    );
    let mut records = Records::new();
    records.put(placed(&floor));
    records.put(placed(&roof));
    records.put(removed(&gone));
    let ids = [floor.id(), roof.id(), gone.id(), 12345];
    let owned = co_own(&records, &ids, &owners);
    assert_eq!(owned.len(), 2, "the standing pieces only");
    for r in owned {
        let body: PieceBody = r.read().unwrap();
        assert_eq!(body.owners, [0, 1]);
        records.put(r);
    }
    assert!(co_own(&records, &ids, &owners).is_empty());
    // A co-owned piece is still the piece it was.
    let pieces = crate::building::from_records(&records);
    assert_eq!(pieces.get(&floor.id()), Some(&floor));
    assert_eq!(
        records
            .get(PIECE_RECORD, floor.id())
            .unwrap()
            .read::<PieceBody>()
            .unwrap()
            .piece,
        Some(floor)
    );
}

/// Saved games survive: a world from before crew missions has no record
/// and reads as a crew with nothing done; a record of a later schema is
/// left alone; a mission file with no crew section reads as it did.
#[test]
fn a_world_without_the_record_loads_as_a_fresh_crew() {
    let records = Records::new();
    assert_eq!(
        CrewProgress::from_records(&records),
        Some(CrewProgress::default())
    );
    let mut later = CrewProgress::default().record();
    later.schema = CREW_MISSIONS_SCHEMA + 1;
    let mut records = Records::new();
    records.put(later);
    assert_eq!(CrewProgress::from_records(&records), None);
    let town: MissionFile = ron::from_str(include_str!(
        "../../../../../assets/config/town_missions.ron"
    ))
    .unwrap();
    assert!(town.crew.is_none());
    // An empty crew record's body is the least it can be.
    assert_eq!(CrewProgress::default().record().body, "(done:[],shares:{})");
}

/// The crew file is validated: a grace time must be a time, and an `each`
/// no more than its count.
#[test]
fn the_crew_file_is_validated() {
    let catalog = shipped_catalog(&shipped_book());
    let mut file = shipped_file().crew.unwrap();
    file.grace_s = -1.0;
    assert!(file.resolve(&catalog, &[]).is_err());
    file.grace_s = f64::NAN;
    assert!(file.resolve(&catalog, &[]).is_err());
    file.grace_s = 0.0;
    assert!(file.resolve(&catalog, &[]).is_ok(), "nought is a time");
    file.missions[1].objectives[1].each = 25;
    assert!(file.resolve(&catalog, &[]).is_err());
}
