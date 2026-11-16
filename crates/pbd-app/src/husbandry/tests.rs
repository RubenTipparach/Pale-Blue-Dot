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

/// MEASUREMENT (`step-videos` 5.2, the sturdier bot; run by hand with
/// `--ignored`): level pasture by the sea with a wild Yard hen herd within
/// 30 m, near the fields spawn, as `--at` degrees for the showcase's start.
#[test]
#[ignore]
fn find_a_coastal_start_with_hens() {
    use crate::planet::surface_height;
    let settings: HusbandrySettings =
        ron::from_str(include_str!("../../../../assets/config/husbandry.ron")).unwrap();
    let roster = settings
        .land
        .values()
        .find(|r| r.iter().any(|s| s.name == "Yard hen"))
        .expect("a roster with the hen")
        .clone();
    let hen = roster.iter().position(|s| s.name == "Yard hen").unwrap() as u16;
    let seed = crate::planet::terrain_config().seed;
    let spawn = Vec3::new(0.8776, 0.4794, 0.0).normalize();
    let mut found = Vec::new();
    for i in -500..=500 {
        for j in -500..=500 {
            let east = Vec3::Y.cross(spawn).normalize();
            let north = spawn.cross(east);
            let d = (spawn * PLANET_RADIUS + east * (i as f32 * 12.0) + north * (j as f32 * 12.0))
                .normalize();
            let h = surface_height(d);
            if !(1.0..6.0).contains(&h) {
                continue;
            }
            let e2 = Vec3::Y.cross(d).normalize();
            let n2 = d.cross(e2);
            let at = |r: f32, a: f32| {
                (d * PLANET_RADIUS + (e2 * a.cos() + n2 * a.sin()) * r).normalize()
            };
            let sea = (0..12).any(|k| {
                let a = k as f32 / 12.0 * std::f32::consts::TAU;
                surface_height(at(25.0, a)) < -0.6
            });
            if !sea {
                continue;
            }
            let level = (0..8).all(|k| {
                let a = k as f32 / 8.0 * std::f32::consts::TAU;
                [10.0, 20.0].iter().all(|r| {
                    let hh = surface_height(at(*r, a));
                    hh < 0.0 || (hh - h).abs() < 3.0
                })
            });
            if !level {
                continue;
            }
            let hens = herds::squares_near(d, 60.0, PLANET_RADIUS, &settings.herds)
                .into_iter()
                .filter_map(|sq| herds::herd_in(seed, sq, PLANET_RADIUS, &settings.herds, &roster))
                .any(|herd| {
                    herd.species == hen
                        && herd.centre.angle_between(d) * PLANET_RADIUS < 45.0
                        && surface_height(herd.centre) > 0.5
                });
            if hens {
                found.push((d.distance(spawn) * PLANET_RADIUS, d));
            }
        }
    }
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (m, d) in found.iter().take(6) {
        let (lat, lon) = pbd_core::geo::lat_lon(*d).degrees();
        println!("COASTAL_HENS {m:.0} m: --at {lat:.4} {lon:.4}");
    }
    assert!(!found.is_empty());
}
