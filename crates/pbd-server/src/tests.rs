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
    // Each request: Accepted to its player, then the change to both, then
    // the dug block's drop to both (design 23).
    let out = s.drain();
    assert_eq!(out.len(), 6 * 5);
    for chunk in out.chunks(5) {
        assert!(matches!(
            chunk[0],
            (To::Player(0), ToClient::Accepted { .. })
        ));
        assert!(matches!(chunk[1], (To::Player(0), ToClient::Change { .. })));
        assert!(matches!(chunk[2], (To::Player(1), ToClient::Change { .. })));
        assert!(matches!(chunk[3], (To::Player(0), ToClient::Drop { .. })));
        assert!(matches!(chunk[4], (To::Player(1), ToClient::Drop { .. })));
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

/// Design 19: a pack move is decided by the core rule and journaled; one
/// from an empty slot is refused and journals nothing.
#[test]
fn a_pack_move_goes_through_the_server() {
    use pbd_core::accept::PackMove;
    let (mut s, mut st) = server();
    let mut slots = Slots::default();
    slots.give(Item::Block(Material::Stone), 10);
    joined(&mut s, &[1], slots.clone());
    let mv = PackMove::Shift {
        from: 0,
        to: 12,
        count: 4,
    };
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 1,
            request: Request::Pack(mv),
        },
    );
    let mut by_hand = slots.clone();
    by_hand.shift(0, 12, 4);
    assert_eq!(s.player(0).unwrap().slots, by_hand);
    assert_eq!(st.hands, vec![(Author::Player(0), None, by_hand.clone())]);
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 2,
            request: Request::Pack(PackMove::Send { index: 7 }),
        },
    );
    assert_eq!(st.hands.len(), 1, "an empty slot's move is refused");
    assert!(s.drain().iter().any(|(_, m)| matches!(
        m,
        ToClient::Refused {
            id: 2,
            reason: Refusal::EmptySlot
        }
    )));
}

/// Design 19: two players reach one drop; the first is given it and the
/// second is refused as gone, so it is given once. A partial pickup leaves
/// the rest on the drop.
#[test]
fn two_players_reaching_one_drop_are_given_it_once() {
    let (mut s, mut st) = server();
    joined(&mut s, &[1, 2], Slots::default());
    let item = Item::Block(Material::Dirt);
    st.drops.insert(
        9,
        ItemDrop {
            id: 9,
            item,
            count: 3,
            position: glam::DVec3::ZERO,
            made_s: 0.0,
        },
    );
    s.handle(
        &mut st,
        1,
        ToServer::Request {
            id: 1,
            request: Request::Pick { drop: 9 },
        },
    );
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 2,
            request: Request::Pick { drop: 9 },
        },
    );
    assert_eq!(s.player(1).unwrap().slots.get(0).map(|x| x.count), Some(3));
    assert_eq!(s.player(0).unwrap().slots.get(0), None);
    assert!(st.drops.is_empty());
    assert_eq!(st.hands.len(), 1);
    assert_eq!(st.hands[0].1, Some((9, 0)));
    assert!(s.drain().iter().any(|(to, m)| *to == To::Player(0)
        && matches!(
            m,
            ToClient::Refused {
                id: 2,
                reason: Refusal::Gone
            }
        )));
    // A pack with room for only part of a drop leaves the rest on it.
    let mut full = Slots::default();
    assert!(
        full.give(Item::Block(Material::Stone), u16::MAX) > 0,
        "every slot full"
    );
    assert_eq!(full.take(5, 2), 2);
    st.drops.insert(
        10,
        ItemDrop {
            id: 10,
            item: Item::Block(Material::Stone),
            count: 5,
            position: glam::DVec3::ZERO,
            made_s: 0.0,
        },
    );
    let (_, left) = s.pick(&mut st, 0, 3, 10, full).unwrap();
    assert_eq!(left, 3);
    assert_eq!(st.drops[&10].count, 3);
}

/// A crew chain of one objective, seven blocks of anything dug by two
/// hands (`coop-missions` design 3).
fn crew_chain() -> pbd_core::missions::crew::CrewChain {
    use pbd_core::missions::crew::{CrewFile, CrewMissionEntry, CrewObjectiveEntry};
    let file = CrewFile {
        grace_s: 300.0,
        missions: vec![CrewMissionEntry {
            id: "house-site".into(),
            needs: vec![],
            icon: String::new(),
            objectives: vec![CrewObjectiveEntry {
                kind: Kind::Gather,
                what: "any".into(),
                count: 7,
                hands: 2,
                each: 0,
            }],
        }],
    };
    let catalog = pbd_core::missions::Catalog {
        live: Kind::LIVE.into_iter().collect(),
        ..Default::default()
    };
    file.resolve(&catalog, &[]).expect("resolves")
}

/// `coop-missions` task 2.1: two players' digs, decided by the server,
/// count into the world's one crew record with each share right; the store
/// journals it, and a server opened on that record again keeps both.
#[test]
fn two_players_digs_count_into_the_crew_record() {
    use pbd_core::missions::crew::{CREW_MISSIONS_ID, CREW_MISSIONS_KIND, CrewProgress};
    let (mut s, mut st) = server();
    s.set_crew(crew_chain(), CrewProgress::default());
    joined(&mut s, &[1, 2], Slots::default());
    let cells = dry_cells(&s, 7);
    for (n, &(cell, layer)) in cells.iter().enumerate() {
        let id = n as u32 + 1;
        let player = if n < 3 { 0 } else { 1 };
        s.handle(
            &mut st,
            player,
            ToServer::Request {
                id,
                request: Request::Dig { cell, layer },
            },
        );
        if n == 5 {
            // Six dug: the record as journaled, read back as a reload does.
            let saved = st
                .records
                .get(CREW_MISSIONS_KIND, CREW_MISSIONS_ID)
                .unwrap();
            let back = CrewProgress::from_records(&st.records).unwrap();
            assert_eq!(back, s.crew().unwrap().progress);
            assert_eq!(saved.read::<CrewProgress>().unwrap(), back);
        }
    }
    assert_eq!(st.entries.len(), 7, "every dig accepted");
    let crew = s.crew().unwrap();
    let site = crew.chain.chain.get("house-site").unwrap();
    assert!(crew.progress.is_done("house-site"));
    let shares = crew.progress.shares(site, 0);
    assert_eq!(shares, BTreeMap::from([(0, 3), (1, 4)]));
    // A reload: a new server on the journaled record keeps both shares.
    let saved = CrewProgress::from_records(&st.records).unwrap();
    assert_eq!(saved.shares(site, 0), shares);
    assert!(saved.is_done("house-site"));
    let (mut again, _) = server();
    again.set_crew(crew_chain(), saved.clone());
    assert_eq!(again.crew().unwrap().progress, saved);
}

/// A store that takes every edit and refuses every record.
struct NoRecords(MemoryStore);

impl Store for NoRecords {
    fn edits(&self) -> &Edits {
        self.0.edits()
    }
    fn ledger(&self) -> &Ledger {
        self.0.ledger()
    }
    fn commit(&mut self, author: &Author, edit: Edit, carried: &Slots) -> Option<Sequenced> {
        self.0.commit(author, edit, carried)
    }
    fn floating(
        &mut self,
        author: &Author,
        item: Item,
        cell: u32,
        layer: u16,
        made_s: f64,
    ) -> Option<ItemDrop> {
        self.0.floating(author, item, cell, layer, made_s)
    }
    fn commit_pack(&mut self, author: &Author, carried: &Slots) -> bool {
        self.0.commit_pack(author, carried)
    }
    fn drop(&self, id: u64) -> Option<ItemDrop> {
        self.0.drop(id)
    }
    fn commit_pick(&mut self, author: &Author, id: u64, left: u16, carried: &Slots) -> bool {
        self.0.commit_pick(author, id, left, carried)
    }
    fn commit_record(&mut self, _: &Author, _: Record) -> bool {
        false
    }
}

/// A crew record the store refuses leaves the crew's progress as it was;
/// the dig it came from was durable first, and stands.
#[test]
fn a_refused_crew_record_changes_nothing() {
    use pbd_core::missions::crew::CrewProgress;
    let (mut s, st) = server();
    let mut st = NoRecords(st);
    s.set_crew(crew_chain(), CrewProgress::default());
    joined(&mut s, &[1], Slots::default());
    let (cell, layer) = dry_cells(&s, 1)[0];
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 1,
            request: Request::Dig { cell, layer },
        },
    );
    assert_eq!(st.0.entries.len(), 1, "the dig stands");
    assert_eq!(s.crew().unwrap().progress, CrewProgress::default());
}

/// A server with no crew chain, single player's today, counts nothing and
/// journals no record.
#[test]
fn a_world_with_no_crew_chain_journals_no_crew_record() {
    let (mut s, mut st) = server();
    joined(&mut s, &[1], Slots::default());
    let (cell, layer) = dry_cells(&s, 1)[0];
    s.handle(
        &mut st,
        0,
        ToServer::Request {
            id: 1,
            request: Request::Dig { cell, layer },
        },
    );
    assert_eq!(st.entries.len(), 1);
    assert!(st.records.is_empty());
}

// ------------------------------------------------ survival (task 4.1)

use crate::survival::{Act, VITALS_EVERY_S, VITALS_KIND, vitals_of};
use pbd_core::survival::health::Cause;
use pbd_core::survival::temperature::Exposure;
use pbd_core::survival::{Conditions, Event, SurvivalConfig, Vitals, WorldMode};

#[test]
fn a_long_dive_drains_health_and_surfacing_refills_breath() {
    let cfg = SurvivalConfig::shipped();
    let (mut s, mut store) = server();
    let p = joined(&mut s, &[1], Slots::new())[0];
    s.set_vitals(p, vitals_of(&cfg, WorldMode::Survival, &store.records, p));
    let dive = Conditions {
        head_in_water: true,
        in_deep_water: true,
        exposure: Exposure {
            air_c: 20.0,
            in_water: true,
            ..Exposure::default()
        },
        ..Conditions::default()
    };
    // Breath runs out after its 30 s, then health drains at 5 a second.
    for _ in 0..400 {
        s.survive(&mut store, &cfg, p, &dive, 0.1).unwrap();
    }
    let v = s.vitals(p).unwrap().clone();
    assert_eq!(v.breath.breath_s, 0.0);
    assert!((v.health - 50.0).abs() < 1.0, "{}", v.health);
    // Journaled as it went: the record holds the drained bar.
    let saved: Vitals = store
        .records
        .get(VITALS_KIND, u64::from(p))
        .and_then(|r| r.read())
        .unwrap();
    // (at most VITALS_EVERY_S behind: 5 s of drowning is 25 points)
    assert!(
        saved.health < v.health + 26.0 && saved.health < 80.0,
        "{}",
        saved.health
    );
    let surfaced = Conditions::default();
    for _ in 0..40 {
        s.survive(&mut store, &cfg, p, &surfaced, 0.1).unwrap();
    }
    let v = s.vitals(p).unwrap();
    assert!((v.breath.breath_s - cfg.breath.under_s).abs() < 1e-3);
}

#[test]
fn an_explorer_world_has_no_vitals_and_an_old_save_reads_explorer() {
    let cfg = SurvivalConfig::shipped();
    let (mut s, mut store) = server();
    let p = joined(&mut s, &[1], Slots::new())[0];
    // A save from before survival has no mode: Explorer, no bars.
    let mode = WorldMode::of_saved(None);
    s.set_vitals(p, vitals_of(&cfg, mode, &store.records, p));
    assert!(
        s.survive(&mut store, &cfg, p, &Conditions::default(), 1.0)
            .is_none()
    );
    assert!(
        s.survival_act(&mut store, &cfg, p, Act::Hit(Cause::Fall, 50.0))
            .is_none()
    );
    assert!(store.records.get(VITALS_KIND, u64::from(p)).is_none());
}

#[test]
fn vitals_are_journaled_on_a_hit_and_every_few_seconds_and_read_back() {
    let cfg = SurvivalConfig::shipped();
    let (mut s, mut store) = server();
    let p = joined(&mut s, &[1], Slots::new())[0];
    s.set_vitals(p, vitals_of(&cfg, WorldMode::Survival, &store.records, p));
    // Ticking alone writes nothing until VITALS_EVERY_S has passed.
    s.survive(&mut store, &cfg, p, &Conditions::default(), 1.0);
    assert!(store.records.get(VITALS_KIND, u64::from(p)).is_none());
    s.survive(&mut store, &cfg, p, &Conditions::default(), VITALS_EVERY_S);
    assert!(store.records.get(VITALS_KIND, u64::from(p)).is_some());
    // A hit is written at once.
    let acted = s
        .survival_act(&mut store, &cfg, p, Act::Hit(Cause::Fall, 100.0))
        .unwrap();
    assert!(acted.events.contains(&Event::Died(Cause::Fall)));
    // A reopened world (a new server over the same records) reads it back.
    let (mut again, _) = server();
    let q = joined(&mut again, &[1], Slots::new())[0];
    again.set_vitals(q, vitals_of(&cfg, WorldMode::Survival, &store.records, q));
    assert!(again.vitals(q).unwrap().is_dead());
    // Waking up is journaled too: full health, half food.
    s.survival_act(&mut store, &cfg, p, Act::WakeUp).unwrap();
    let saved: Vitals = store
        .records
        .get(VITALS_KIND, u64::from(p))
        .and_then(|r| r.read())
        .unwrap();
    assert_eq!(saved.health, cfg.health.max_pts);
    assert_eq!(saved.food_fp as f32, cfg.food.wake_fp);
}

/// The messages queued for `player`, in order.
fn to_player(s: &mut Server, player: u32) -> Vec<ToClient> {
    s.drain()
        .into_iter()
        .filter(|(to, _)| *to == To::Player(player))
        .map(|(_, m)| m)
        .collect()
}

/// Design 22: a remote player is sent their slots after the welcome, and a
/// place is answered, then followed by their slots with the block spent.
#[test]
fn a_remote_players_place_is_followed_by_their_slots() {
    use pbd_core::inventory::Stack;
    let (mut s, mut st) = server();
    let mut carried = Slots::default();
    carried.set(0, Some(Stack::new(Item::Block(Material::Stone), 3)));
    let host = s.join(0, &hello(1), carried.clone()).unwrap();
    let guest = s.join(1, &hello(2), carried).unwrap();
    assert_eq!((host, guest), (0, 1));
    let sent = s.drain();
    // The host's own player is never sent slots; the guest is, after its
    // welcome.
    let slots_to = |p: u32| {
        sent.iter()
            .filter(|(to, m)| *to == To::Player(p) && matches!(m, ToClient::Slots { .. }))
            .count()
    };
    assert_eq!((slots_to(0), slots_to(1)), (0, 1));
    let (cell, top) = dry_cells(&s, 1)[0];
    s.handle(
        &mut st,
        guest,
        ToServer::Request {
            id: 4,
            request: Request::Place {
                cell,
                layer: top + 1,
                slot: 0,
            },
        },
    );
    let got = to_player(&mut s, guest);
    let accepted = got
        .iter()
        .position(|m| matches!(m, ToClient::Accepted { id: 4, .. }))
        .unwrap_or_else(|| panic!("accepted: {got:?}"));
    let Some(ToClient::Slots { slots }) = got.last() else {
        panic!("the slots come last: {got:?}");
    };
    assert!(got.len() - 1 > accepted);
    assert_eq!(slots.get(0).map(|stack| stack.count), Some(2));
    assert_eq!(slots, &s.player(guest).unwrap().slots);
}

/// Design 22: a refused place is answered with the slots as they were, so
/// a prediction can be put back.
#[test]
fn a_refused_place_is_answered_with_the_slots_as_they_were() {
    let (mut s, mut st) = server();
    let players = joined(&mut s, &[1, 2], Slots::default());
    let (cell, top) = dry_cells(&s, 1)[0];
    s.handle(
        &mut st,
        players[1],
        ToServer::Request {
            id: 2,
            request: Request::Place {
                cell,
                layer: top + 1,
                slot: 0,
            },
        },
    );
    let got = to_player(&mut s, players[1]);
    assert!(
        matches!(got.first(), Some(ToClient::Refused { id: 2, .. })),
        "{got:?}"
    );
    assert_eq!(
        got.last(),
        Some(&ToClient::Slots {
            slots: Slots::default()
        })
    );
    assert!(st.entries.is_empty());
}

/// Design 22: a player who leaves and joins again is given the slots they
/// left with, not the new player's kit.
#[test]
fn a_player_who_rejoins_gets_the_slots_they_left_with() {
    use pbd_core::inventory::Stack;
    let (mut s, _) = server();
    let mut carried = Slots::default();
    carried.set(2, Some(Stack::new(Item::Block(Material::Stone), 9)));
    let host = s.join(0, &hello(1), Slots::default()).unwrap();
    let guest = s.join(1, &hello(2), carried.clone()).unwrap();
    assert!(s.leave(guest).is_some());
    s.drain();
    let again = s.join(2, &hello(2), Slots::default()).unwrap();
    assert_eq!(again, guest);
    assert_eq!(s.player(guest).unwrap().slots, carried);
    assert_eq!(
        to_player(&mut s, guest).last(),
        Some(&ToClient::Slots { slots: carried })
    );
    let _ = host;
}

/// Design 23: a remote player's dig makes a drop in the store, written with
/// the dig, and every player is told of it; the guest's pickup of it tells
/// every player what is left and sends the guest their slots holding it.
#[test]
fn a_guests_dig_drop_is_told_to_everyone_and_picked_once() {
    let (mut s, mut st) = server();
    let players = joined(&mut s, &[1, 2], Slots::default());
    let (cell, top) = dry_cells(&s, 1)[0];
    let was = s
        .column(&Edits::new(), cell)
        .unwrap()
        .material(top as usize);
    s.handle(
        &mut st,
        players[1],
        ToServer::Request {
            id: 1,
            request: Request::Dig { cell, layer: top },
        },
    );
    let sent = s.drain();
    let made: Vec<(To, ItemDrop)> = sent
        .iter()
        .filter_map(|(to, m)| match m {
            ToClient::Drop { drop } => Some((*to, *drop)),
            _ => None,
        })
        .collect();
    assert_eq!(made.len(), 2, "both players are told: {sent:?}");
    let drop = made[0].1;
    assert_eq!(made[1].1, drop);
    assert_eq!(drop.item, Item::Block(was));
    assert_eq!(st.drop(drop.id), Some(drop), "written with the dig");
    s.handle(
        &mut st,
        players[1],
        ToServer::Request {
            id: 2,
            request: Request::Pick { drop: drop.id },
        },
    );
    let sent = s.drain();
    let left: Vec<&ToClient> = sent
        .iter()
        .filter(|(_, m)| matches!(m, ToClient::DropLeft { .. }))
        .map(|(_, m)| m)
        .collect();
    assert_eq!(left.len(), 2, "both players are told");
    assert_eq!(
        left[0],
        &ToClient::DropLeft {
            id: drop.id,
            left: 0
        }
    );
    let Some((_, ToClient::Slots { slots })) = sent.last() else {
        panic!("the guest's slots come last: {sent:?}");
    };
    assert!(
        slots
            .iter()
            .flatten()
            .any(|stack| stack.item == Item::Block(was) && stack.count == 1)
    );
    // The host reaching the same drop is refused: it is gone.
    s.handle(
        &mut st,
        players[0],
        ToServer::Request {
            id: 3,
            request: Request::Pick { drop: drop.id },
        },
    );
    assert!(s.drain().iter().any(|(_, m)| matches!(
        m,
        ToClient::Refused {
            id: 3,
            reason: Refusal::Gone
        }
    )));
}
