use super::*;
use crate::saves::{create, list};
use pbd_core::husbandry::Edge;
use pbd_core::inventory::Stack;

fn temporary(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-husbandry-{name}-{}-{}",
        std::process::id(),
        crate::saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Task 5.2: a tamed animal, written as the player's the moment it is
/// tamed, survives a quit and a reload with its genome; a fence piece and
/// the pack it was spent from come back with it.
#[test]
fn a_tamed_animal_survives_quit_and_reload_with_its_genome() {
    let root = temporary("tamed");
    let slot = create(&root, "Farm", 5).unwrap();
    let genome = Genome::roll(0xfeed, 3, 2);
    let body = AnimalBody {
        sp: 1,
        sex: Sex::M,
        genes: genome.0.to_vec(),
        wild: 0xfeed,
        home: 4242,
        pen: Some(4242),
        ..AnimalBody::default()
    };
    let edge = Edge::new(4242, 4243);
    let mut carried = Slots::new();
    carried.give(Item::Good(Good::FENCE), 23);
    carried.give(Item::Good(Good::EGG), 2);
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        note_kinds(&mut save);
        assert!(
            save.store(&Author::Player(0), vec![body.record(1)])
                .is_some()
        );
        let piece = FenceBody {
            gate: true,
            state: Standing::Placed,
        };
        assert!(
            save.store(&Author::Player(0), vec![piece.record(edge)])
                .is_some()
        );
        assert!(save.record_pack(&carried));
        save.drain();
    }
    let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
    let kept = record::animals(&reopened.records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].0, 1);
    assert_eq!(kept[0].1.genome(), genome, "the same genome");
    assert_eq!(kept[0].1, body);
    assert_eq!(record::pieces(&reopened.records).get(&edge), Some(&true));
    assert_eq!(reopened.carried.as_ref(), Some(&carried), "eggs and fences");
    assert_eq!(
        reopened.identity.records.get(record::ANIMAL),
        Some(&record::ANIMAL_SCHEMA),
        "identity.ron names the kind and its schema"
    );
    // The player wrote it: no world process may change its pen.
    let moved = AnimalBody {
        pen: None,
        ..body.clone()
    };
    let mut reopened = reopened;
    assert!(
        reopened
            .propose(
                &Author::World {
                    process: "husbandry".into(),
                    kind: record::ANIMAL.into(),
                    id: 1,
                },
                pbd_core::records::Proposal {
                    edits: vec![],
                    records: vec![moved.record(1)],
                },
            )
            .is_err()
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Task 5.3: a world saved before husbandry opens and gains no animals and
/// no pieces; its identity names no new kinds until one is stored.
#[test]
fn a_world_saved_before_opens_and_gains_no_animals() {
    let root = temporary("before");
    let slot = create(&root, "Old", 9).unwrap();
    let mut carried = Slots::new();
    carried.give(Item::Block(Material::Dirt), 3);
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        assert!(save.record_pack(&carried));
        save.drain();
    }
    let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
    assert!(record::animals(&reopened.records).is_empty());
    assert!(record::pieces(&reopened.records).is_empty());
    assert!(!reopened.identity.records.contains_key(record::ANIMAL));
    let _ = std::fs::remove_dir_all(&root);
}

/// The slice's goods are saved by their codes, read back, and each has its
/// icon on disk.
#[test]
fn every_good_round_trips_its_slot_code_and_has_its_icon() {
    let goods = [Good::FENCE, Good::GATE, Good::EGG, Good::WOOL, Good::MILK];
    let mut slots = Slots::new();
    for (k, good) in goods.into_iter().enumerate() {
        slots.set(k, Some(Stack::new(Item::Good(good), 3)));
    }
    let line = crate::saves::format::pack_line_of(&slots);
    assert!(
        line.contains(" gfence,3 ggate,3 gegg,3 gwool,3 gmilk,3"),
        "{line}"
    );
    let back = crate::saves::format::parse_line(&line);
    assert!(
        matches!(back, Some(crate::saves::format::Record::Pack { slots: ref s }) if *s == slots),
        "{back:?}"
    );
    let assets = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    for good in goods {
        assert!(assets.join(good.icon()).exists(), "{good:?}");
    }
}

/// A rig is whole boxes, and a walking leg moves where a standing one does
/// not: the feet follow the phase.
#[test]
fn a_rig_builds_boxes_and_its_legs_follow_the_phase() {
    let settings = HusbandrySettings::default();
    for sp in settings.roster("home") {
        let build = |phase: f32, swing: f32| {
            let mut b = rig::Builder::default();
            rig::build(
                &mut b,
                sp,
                &Genome::default(),
                Sex::F,
                &rig::Pose {
                    phase,
                    swing,
                    ..rig::Pose::default()
                },
                [Vec3::X, Vec3::Y, Vec3::Z],
            );
            b
        };
        let still = build(0.0, 0.0);
        assert_eq!(still.positions.len() % 24, 0, "{}", sp.id);
        assert_eq!(still.indices.len(), still.positions.len() / 4 * 6);
        let top = still
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::MIN, f32::max);
        assert!(
            (top - sp.height_m).abs() < sp.height_m * 0.5,
            "{}: {top} m tall",
            sp.id
        );
        let low = still
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::MAX, f32::min);
        assert!(low.abs() < 0.05, "{}: feet at {low}", sp.id);
        let a = build(1.0, 0.5);
        let b = build(2.5, 0.5);
        assert_ne!(a.positions, b.positions, "{}: the legs move", sp.id);
    }
}
