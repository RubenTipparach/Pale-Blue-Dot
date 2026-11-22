use super::*;
use pbd_core::cell_key::{Address, key};
use pbd_core::net::protocol::{PROTOCOL, PROTOCOL_MIN};
use pbd_core::terrain::{GENERATOR_VERSION, TOPOLOGY_VERSION};

fn versions() -> Versions {
    Versions {
        protocol: PROTOCOL,
        protocol_min: PROTOCOL_MIN,
        topology: TOPOLOGY_VERSION,
        generator: GENERATOR_VERSION,
        content: 7,
        world: 99,
    }
}

fn server() -> (Server, MemoryStore) {
    let world = World {
        terrain: TerrainConfig::for_version(GENERATOR_VERSION).expect("the current generator"),
        worms: WormField::DEFAULT,
        versions: versions(),
    };
    (Server::new(world, 1000.0), MemoryStore::new(Edits::new()))
}

fn hello(key: u8) -> ToServer {
    ToServer::Hello {
        versions: versions(),
        key: [key; 16],
        name: format!("player {key}"),
    }
}

/// Dry ground cells: a surface with air over it, no water in the column.
fn dry_cells(s: &Server, count: usize) -> Vec<(u32, u16)> {
    let mut out = Vec::new();
    'faces: for face in 0..20u8 {
        for n in 0..40u32 {
            let a = Address {
                face,
                level: 11,
                i: 200 + n * 23,
                j: 300 + n * 17,
            };
            let Some(cell) = key(a) else { continue };
            let column = s.column(&Edits::new(), cell).unwrap();
            let Some(top) = column.surface() else {
                continue;
            };
            if column.material(top + 1) == Material::Air
                && (0..column::LAYERS).all(|l| !column.material(l).is_water())
                && !column.material(top).is_lamp()
            {
                out.push((cell, top as u16));
                if out.len() == count {
                    break 'faces;
                }
            }
        }
    }
    assert_eq!(out.len(), count, "dry ground found");
    out
}

fn joined(s: &mut Server, keys: &[u8], slots: Slots) -> Vec<u32> {
    let players = keys
        .iter()
        .enumerate()
        .map(|(n, &k)| s.join(n as u64, &hello(k), slots.clone()).unwrap())
        .collect();
    s.drain();
    players
}

/// Task 4.1: requests are decided, journaled with ascending sequences per
/// region, answered, and sent to every player in the server's order.
#[test]
fn requests_are_journaled_in_order_and_sent_to_everyone() {
    let (mut s, mut st) = server();
    let players = joined(&mut s, &[1, 2], Slots::default());
    assert_eq!(players, vec![0, 1]);
    let cells = dry_cells(&s, 3);
    let mut id = 0;
    for &(cell, top) in &cells {
        for layer in [top, top - 1] {
            id += 1;
            s.handle(
                &mut st,
                0,
                ToServer::Request {
                    id,
                    request: Request::Dig { cell, layer },
                },
            );
        }
    }
    let journal = &st.entries;
    assert_eq!(journal.len(), 6);
    // Ascending within each region, from 1.
    let mut last: BTreeMap<u32, u64> = BTreeMap::new();
    for (author, sequenced, _) in journal {
        assert_eq!(*author, Author::Player(0));
        let prev = last.insert(sequenced.region, sequenced.seq).unwrap_or(0);
        assert_eq!(sequenced.seq, prev + 1);
    }
    for (cell, top) in &cells {
        assert_eq!(
            s.column(&st.edits, *cell)
                .unwrap()
                .material(usize::from(*top)),
            Material::Air
        );
    }
    // Each request: Accepted to its player, then the change to both.
    let out = s.drain();
    assert_eq!(out.len(), 6 * 3);
    for chunk in out.chunks(3) {
        assert!(matches!(
            chunk[0],
            (To::Player(0), ToClient::Accepted { .. })
        ));
        assert!(matches!(chunk[1], (To::Player(0), ToClient::Change { .. })));
        assert!(matches!(chunk[2], (To::Player(1), ToClient::Change { .. })));
    }
    // The ledger is the world's: its digest is a recompute of the edits.
    for (_, sequenced, _) in &st.entries {
        let r = sequenced.region;
        assert_eq!(
            st.ledger.digest(r),
            pbd_core::net::digest::recompute(r, &st.edits)
        );
    }
}

/// Two players dig the same layer: the server's order decides, the second
/// is refused as unchanged and re-sent the region, and only one dig is in
/// the journal.
#[test]
fn two_players_on_one_cell_are_ordered_by_the_server() {
    let (mut s, mut st) = server();
    joined(&mut s, &[1, 2], Slots::default());
    let (cell, top) = dry_cells(&s, 1)[0];
    s.handle(
        &mut st,
        1,
        ToServer::Request {
            id: 7,
            request: Request::Dig { cell, layer: top },
        },
    );
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 3,
            request: Request::Dig { cell, layer: top },
        },
    );
    assert_eq!(st.entries.len(), 1);
    assert_eq!(st.entries[0].0, Author::Player(1));
    let out = s.drain();
    let refused: Vec<_> = out
        .iter()
        .filter(|(to, m)| {
            *to == To::Player(0)
                && matches!(m, ToClient::Refused { .. } | ToClient::Snapshot { .. })
        })
        .collect();
    assert!(matches!(
        refused[0].1,
        ToClient::Refused {
            id: 3,
            reason: Refusal::Unchanged
        }
    ));
    let ToClient::Snapshot { seq, bytes, .. } = &refused[1].1 else {
        panic!("the region follows the refusal")
    };
    assert_eq!(*seq, 1);
    let (_, held) = pbd_core::net::digest::snapshot_edits(bytes).unwrap();
    assert_eq!(
        held,
        vec![Edit {
            cell,
            layer: top,
            material: Material::Air
        }]
    );
}

/// A place spends the slot it names; an empty slot is refused; a journal
/// that refuses changes nothing.
#[test]
fn a_place_spends_and_a_refusal_changes_nothing() {
    let (mut s, mut st) = server();
    let mut slots = Slots::default();
    slots.give(Item::Block(Material::Stone), 2);
    joined(&mut s, &[1], slots);
    let (cell, top) = dry_cells(&s, 1)[0];
    let above = top + 1;
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 1,
            request: Request::Place {
                cell,
                layer: above,
                slot: 0,
            },
        },
    );
    assert_eq!(
        s.column(&st.edits, cell)
            .unwrap()
            .material(usize::from(above)),
        Material::Stone
    );
    assert_eq!(s.player(0).unwrap().slots.get(0).unwrap().count, 1);
    // Slot 5 holds nothing.
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 2,
            request: Request::Place {
                cell,
                layer: above + 1,
                slot: 5,
            },
        },
    );
    assert!(s.drain().iter().any(|(_, m)| matches!(
        m,
        ToClient::Refused {
            id: 2,
            reason: Refusal::EmptySlot
        }
    )));
    // The writer fails: the dig is refused and the world stays.
    st.failed = true;
    let before = &st.edits.len();
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 3,
            request: Request::Dig { cell, layer: above },
        },
    );
    assert_eq!(&st.edits.len(), before);
    assert_eq!(
        s.column(&st.edits, cell)
            .unwrap()
            .material(usize::from(above)),
        Material::Stone
    );
    assert!(s.drain().iter().any(|(_, m)| matches!(
        m,
        ToClient::Refused {
            id: 3,
            reason: Refusal::NotSaved
        }
    )));
    assert_eq!(s.player(0).unwrap().slots.get(0).unwrap().count, 1);
}

/// Design 9: a build of another world or generator is refused; the same
/// install key gets the same number back; a ninth player is refused.
#[test]
fn a_join_is_checked_and_numbered() {
    let (mut s, _) = server();
    let wrong = ToServer::Hello {
        versions: Versions {
            generator: 1,
            ..versions()
        },
        key: [9; 16],
        name: "x".into(),
    };
    assert_eq!(s.join(50, &wrong, Slots::default()), None);
    assert_eq!(
        s.drain(),
        vec![(
            To::Connection(50),
            ToClient::Rejected {
                reason: JoinRefusal::WorldVersion
            }
        )]
    );
    for k in 0..MAX_PLAYERS as u8 {
        assert_eq!(
            s.join(u64::from(k), &hello(k), Slots::default()),
            Some(u32::from(k))
        );
    }
    assert_eq!(s.join(60, &hello(200), Slots::default()), None);
    s.leave(3);
    assert_eq!(
        s.join(61, &hello(3), Slots::default()),
        Some(3),
        "the same key, the same number"
    );
}

/// Design 11 and 15.8: a server with nobody on it does not advance its
/// clock; with a player on, it runs and is sent every five seconds.
#[test]
fn the_clock_runs_only_while_someone_plays() {
    let (mut s, _) = server();
    for _ in 0..200 {
        s.tick(TICK_S);
    }
    assert_eq!(s.clock_s(), 1000.0);
    joined(&mut s, &[1], Slots::default());
    for _ in 0..(20 * 11) {
        s.tick(TICK_S);
    }
    assert!((s.clock_s() - 1011.0).abs() < 1e-6);
    let clocks = s
        .drain()
        .into_iter()
        .filter(|(_, m)| matches!(m, ToClient::Clock { .. }))
        .count();
    assert_eq!(clocks, 2);
}
