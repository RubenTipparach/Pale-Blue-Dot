use super::*;
use crate::terrain::Material;

fn site(id: u64, body: &str) -> Record {
    Record {
        kind: "site".into(),
        id,
        schema: 1,
        body: body.into(),
    }
}

/// Every author's token reads back as that author, and a token that names
/// nobody is refused (task 3.2).
#[test]
fn every_author_round_trips_through_its_token() {
    for author in [
        Author::Player(0),
        Author::Player(7),
        Author::Creation,
        Author::World {
            process: "growth".into(),
            kind: "settlement".into(),
            id: 111_243,
        },
    ] {
        assert_eq!(Author::parse(&author.token()), Some(author.clone()));
    }
    assert_eq!(Author::Player(0).token(), "@p");
    for bad in [
        "p",
        "@",
        "@x",
        "@px",
        "@w",
        "@wgrowth",
        "@wgrowth:site",
        "@wGrowth:site/1",
        "@wgrowth:site/x",
    ] {
        assert_eq!(Author::parse(bad), None, "{bad:?}");
    }
}

/// A line with no token is the player's, as every line before authors was;
/// a token leads the line; a bad token is a damaged line, not the player's.
#[test]
fn a_line_without_a_token_is_the_players() {
    assert_eq!(split_author("7 3 1"), Some((Author::Player(0), "7 3 1")));
    assert_eq!(split_author("@c 7 3 1"), Some((Author::Creation, "7 3 1")));
    assert_eq!(
        split_author("@p2 pack -"),
        Some((Author::Player(2), "pack -"))
    );
    assert_eq!(split_author("@zz 7 3 1"), None);
    assert_eq!(split_author("@c"), None, "a token with nothing after it");
}

/// A record line carries its author, kind, id, schema and body, and the body
/// comes back byte for byte, whatever it is (task 3.1).
#[test]
fn a_record_line_round_trips_byte_for_byte() {
    let body = r#"(name:"Ashingstead",direction:(0.1,-0.2,0.97),capital:true)"#;
    let record = site(111_243, body);
    let line = record.line(&Author::Creation);
    assert_eq!(line, format!("rec @c site 111243 1 {body}\n"));
    assert_eq!(Record::parse_line(&line), Some((Author::Creation, record)));
    // A kind this build has never heard of, in a schema it has never seen:
    // held as its text, and written back as it came.
    let strange = r#"rec @c landmark 9 4 Landmark(name: "Old Tower", keeps: {"bell": [1, 2]})  "#;
    let (_, r) = Record::parse_line(strange).expect("kept, not understood");
    assert_eq!(r.kind, "landmark");
    assert_eq!(r.schema, 4);
    assert_eq!(
        r.body,
        r#"Landmark(name: "Old Tower", keeps: {"bell": [1, 2]})  "#
    );
    assert_eq!(r.line(&Author::Creation).trim_end_matches('\n'), strange);
    for bad in [
        "rec",
        "rec @c site",
        "rec @c site 1 1",
        "rec @c site 1 1 ",
        "rec site 1 1 (a:1)",
        "rec @c Site 1 1 (a:1)",
        "rec @c site x 1 (a:1)",
        "rec @c site 1 1 (name:\"Ash",
        "rec @c site 1 1 (a:1",
        "7 3 1",
    ] {
        assert_eq!(Record::parse_line(bad), None, "{bad:?}");
    }
}

/// The store keeps the last value for each key, and lists a kind by id.
#[test]
fn the_store_keeps_the_last_value_for_each_key() {
    let mut records = Records::new();
    assert!(records.put(site(5, "(a:1)")).is_none());
    assert!(records.put(site(2, "(a:1)")).is_none());
    let old = records.put(site(5, "(a:2)")).expect("replaced");
    assert_eq!(old.body, "(a:1)");
    records.put(Record {
        kind: "sites".into(),
        id: 0,
        schema: 1,
        body: "(b:1)".into(),
    });
    let ids: Vec<u64> = records.of_kind("site").map(|r| r.id).collect();
    assert_eq!(ids, vec![2, 5]);
    assert_eq!(
        records.get("site", 5).map(|r| r.body.as_str()),
        Some("(a:2)")
    );
    assert_eq!(records.len(), 3);
}

/// The fields a change touches are the top-level names whose values differ.
#[test]
fn a_change_touches_the_fields_it_changes() {
    let set = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<BTreeSet<_>>();
    assert_eq!(
        changed_fields(None, "(name:\"A\",state:0)"),
        set(&["name", "state"])
    );
    assert_eq!(
        changed_fields(Some("(name:\"A\",state:0)"), "(name:\"A\",state:1)"),
        set(&["state"])
    );
    assert_eq!(changed_fields(Some("(a:1)"), "(a:1)"), set(&[]));
    assert_eq!(changed_fields(Some("(a:1)"), "not a struct"), set(&["*"]));
}

/// "The world yields to the player": a proposal touching a cell the player
/// edited, or a record field the player changed, is refused whole; one that
/// touches neither is taken (task 3.3).
#[test]
fn a_world_proposal_touching_the_players_work_is_refused_whole() {
    let mut records = Records::new();
    let house = Record {
        kind: "building".into(),
        id: 40,
        schema: 1,
        body: "(state:0,door:0,walls:0)".into(),
    };
    records.put(house.clone());
    let mut touched = YieldSet::new();
    touched.touch_cell(1234);
    // The player opened the door.
    let opened = Record {
        body: "(state:0,door:1,walls:0)".into(),
        ..house.clone()
    };
    touched.touch_record(Some(&house), &opened);
    records.put(opened.clone());

    let dig = |cell| Edit {
        cell,
        layer: 3,
        material: Material::Air,
    };
    let abandon = Record {
        body: "(state:1,door:1,walls:0)".into(),
        ..opened.clone()
    };
    let shut = Record {
        body: "(state:0,door:0,walls:0)".into(),
        ..opened.clone()
    };
    // Abandoning the house changes its state, which the player never
    // touched, and digs a cell nobody touched: taken.
    let fine = Proposal {
        edits: vec![dig(99)],
        records: vec![abandon.clone()],
    };
    assert_eq!(touched.check(&fine, &records), Ok(()));
    // Closing the door the player opened: refused, naming the door.
    let door = Proposal {
        edits: vec![dig(99)],
        records: vec![abandon.clone(), shut],
    };
    assert_eq!(
        touched.check(&door, &records),
        Err(Refusal::Field {
            kind: "building".into(),
            id: 40,
            field: "door".into()
        })
    );
    // Filling a hole the player dug: refused, naming the cell.
    let fill = Proposal {
        edits: vec![dig(99), dig(1234)],
        records: vec![abandon],
    };
    assert_eq!(touched.check(&fill, &records), Err(Refusal::Cell(1234)));
    // A record the player never touched is the world's to change.
    let other = Proposal {
        edits: vec![],
        records: vec![Record { id: 41, ..house }],
    };
    assert_eq!(touched.check(&other, &records), Ok(()));
}
