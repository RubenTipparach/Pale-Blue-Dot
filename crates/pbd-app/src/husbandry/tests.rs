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
        if sp.plan == pbd_core::husbandry::Plan::Horse {
            // The horse is one lofted body of flat triangles, not boxes
            // (`land-transport` design 11).
            assert_eq!(still.indices.len(), still.positions.len(), "{}", sp.id);
        } else {
            assert_eq!(still.positions.len() % 24, 0, "{}", sp.id);
            assert_eq!(still.indices.len(), still.positions.len() / 4 * 6);
        }
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

/// Design 13, E10: a world saved before breeding (an animal record with no
/// `brood` or `mate`) opens with that animal as it was; the farm's own
/// events write nothing for it however long it is left; and the identity
/// gains no breeding kind until one is stored.
#[test]
fn a_world_saved_before_breeding_opens_unchanged_and_writes_nothing() {
    use pbd_core::husbandry::breeding::{self, Fitness};
    use pbd_core::records::Record;
    let root = temporary("before-breeding");
    let slot = create(&root, "Old farm", 5).unwrap();
    // The line the build before design 13 wrote for a tamed ewe.
    let old = "(body:\"home\",sp:1,sex:F,born:3.0,gen:[1,2,3,4,5,6,7,8,9,10,11,12,13,14],pen:Some(5),home:5,owner:0,state:Alive,via:Fed,wild:77,as_of:4.0,health:100.0,hungry:0.0,fed_until:6.0,made:{})";
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        note_kinds(&mut save);
        let line = Record {
            kind: record::ANIMAL.into(),
            id: 1,
            schema: 1,
            body: old.into(),
        };
        assert!(save.store(&Author::Player(0), vec![line]).is_some());
        save.drain();
    }
    let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
    let kept = record::animals(&reopened.records);
    assert_eq!(kept.len(), 1);
    let body = &kept[0].1;
    assert_eq!((body.brood, body.mate), (None, None));
    assert_eq!(
        (body.sp, body.home, body.pen, body.wild),
        (1, 5, Some(5), 77)
    );
    assert_eq!(
        reopened
            .records
            .get(record::ANIMAL, 1)
            .map(|r| r.body.as_str()),
        Some(old),
        "the stored line itself is untouched"
    );
    let settings = HusbandrySettings::default();
    let roster = settings.roster(pbd_core::fauna::HOME_BODY);
    let content = |_: u64, _: &AnimalBody| Fitness {
        mood: 100.0,
        health: 100.0,
    };
    let events = breeding::farm_events(
        &reopened.records,
        roster,
        &settings.breeding,
        400.0,
        &content,
        &|_| true,
    );
    assert!(
        events.is_empty(),
        "a lone ewe and no broods: nothing to write"
    );
    assert!(
        breeding::KINDS
            .iter()
            .all(|(kind, _)| !reopened.identity.records.contains_key(*kind)),
        "no breeding kind is noted before one is stored"
    );
}

/// Design 13, E3 and E10: fertile eggs in the pack and the carried clutch
/// that holds their parentage survive a quit and a reload, and the kinds are
/// noted.
#[test]
fn fertile_eggs_and_their_parentage_survive_a_reload() {
    use pbd_core::husbandry::breeding::{self, BroodBody, Place};
    let root = temporary("fertile");
    let slot = create(&root, "Hatchery", 5).unwrap();
    let brood = BroodBody {
        sp: 0,
        dam: 2,
        sire: 1,
        eggs: 3,
        laid: 12.5,
        ..BroodBody::default()
    };
    let mut carried = Slots::new();
    carried.give(Item::Good(Good::FERTILE_EGG), 3);
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        assert!(save.store_with_hands(brood.record(1), breeding::BROOD_SCHEMA, &carried));
        save.drain();
    }
    let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
    let broods = breeding::broods(&reopened.records);
    assert_eq!(broods.get(&1), Some(&brood));
    assert_eq!(broods[&1].place, Place::Carried);
    assert_eq!(reopened.carried.as_ref(), Some(&carried));
    assert_eq!(
        reopened.identity.records.get(breeding::BROOD),
        Some(&breeding::BROOD_SCHEMA)
    );
}

/// Design 14: a pen piece wears its texture at the manifest's scale. A
/// 2.8 m rail of `poles` (2 m a repeat) spans 1.4 repeats along its length;
/// an animal's flat box writes no UVs, so its mesh keeps its layout.
#[test]
fn a_textured_box_lays_its_texture_at_the_repeat() {
    let mut rail = rig::Builder::default();
    rail.textured_cuboid(
        Vec3::ZERO,
        [Vec3::X * 1.4, Vec3::Y * 0.05, Vec3::Z * 0.03],
        2.0,
    );
    assert!(rail.textured);
    assert_eq!(rail.uvs.len(), rail.positions.len());
    let widest = rail
        .uvs
        .iter()
        .map(|uv| uv[0].max(uv[1]))
        .fold(0.0f32, f32::max);
    assert!((widest - 1.4).abs() < 1e-5, "{widest}");
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    rail.write(&mut mesh);
    assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    let mut animal = rig::Builder::default();
    animal.cuboid(Vec3::ZERO, [Vec3::X, Vec3::Y, Vec3::Z], [0.5, 0.4, 0.3]);
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    animal.write(&mut mesh);
    assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_none());
    // The pen's textures are the settlement manifest's own.
    let repeats = crate::towns::load_repeats();
    for name in PEN_TEXTURES {
        assert!(repeats.contains_key(name), "{name} is a settlement texture");
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
        .find(|r| r.iter().any(|s| s.name() == "Yard hen"))
        .expect("a roster with the hen")
        .clone();
    let hen = roster.iter().position(|s| s.name() == "Yard hen").unwrap() as u16;
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

/// `world-frames` 10.6: an animal striding across the ground, `steps` steps
/// at `speed`, on a sphere of `radius`, the origin following it (`follow`)
/// or held at the planet's centre (the yard before). The worst per-step
/// planet-frame move against its intended `facing * speed * dt`, m, and the
/// rebases crossed.
fn stride_error(radius: f64, speed: f32, steps: usize, follow: bool) -> (f64, u32) {
    let (errors, rebases) = stride_errors(radius, speed, steps, follow);
    (errors.iter().copied().fold(0.0, f64::max), rebases)
}

/// Every step's error of [`stride_error`], in order.
fn stride_errors(radius: f64, speed: f32, steps: usize, follow: bool) -> (Vec<f64>, u32) {
    use bevy::math::DVec3;
    use pbd_core::world_frame::{ORIGIN_CELL_M, WorldPos};
    let home = crate::sim_origin::HOME;
    let dt = 1.0 / 60.0;
    let ground = move |_: Vec3| radius;
    let start = DVec3::new(0.62, 0.41, -0.67).normalize() * radius;
    let mut o = if follow {
        SimOrigin::new(home, (start / ORIGIN_CELL_M).round() * ORIGIN_CELL_M)
    } else {
        SimOrigin::new(home, DVec3::ZERO)
    };
    let mut pos = o.to_local(WorldPos::new(home, start));
    let mut facing = o.up_at(pos).any_orthonormal_vector();
    let (mut errors, mut rebases) = (Vec::with_capacity(steps), 0);
    for _ in 0..steps {
        let before = o.to_world(pos).pos;
        let intended = (facing * speed * dt).as_dvec3();
        pos = stride(&o, pos, facing, speed, dt, ground);
        let moved = o.to_world(pos).pos - before;
        errors.push((moved - intended).length());
        // As `step` keeps the facing along the ground.
        let up = o.up_at(pos);
        facing = (facing - up * facing.dot(up)).normalize();
        if follow && let Some(r) = o.rebase_for(pos, ORIGIN_CELL_M) {
            // What the yard's hook does at a rebase.
            pos -= r.shift;
            o = o.rebased(r);
            rebases += 1;
        }
    }
    (errors, rebases)
}

/// The median and the worst of a run's errors.
fn p50_max(mut e: Vec<f64>) -> (f64, f64) {
    e.sort_by(f64::total_cmp);
    (e[e.len() / 2], e[e.len() - 1])
}

/// The numbers for `world-frames` 10.7 (`--ignored --nocapture`).
#[test]
#[ignore]
fn stride_error_report() {
    for (radius, speed, follow) in [
        (4_800.0, 1.0, false),
        (4_800.0, 1.0, true),
        (4_800.0, 8.0, false),
        (4_800.0, 8.0, true),
        (4_800_000.0, 1.0, true),
        (4_800_000.0, 8.0, true),
    ] {
        let (e, rebases) = stride_errors(radius, speed, 60 * 300, follow);
        let (p50, max) = p50_max(e);
        eprintln!(
            "STRIDE r {radius} m, {speed} m/s, {}: p50 {:.3} um, max {:.3} um, {rebases} rebases",
            if follow { "local" } else { "planet-centred" },
            p50 * 1e6,
            max * 1e6
        );
    }
}

/// An animal walking and running across rebases moves continuously: every
/// step's planet-frame move is its intended step to 1e-5 m, at the home
/// planet's radius and a thousand times it. Planet-centred, as the yard
/// was, a step is off by more (the before).
#[test]
fn an_animal_strides_continuously_across_a_rebase() {
    for radius in [4_800.0, 4_800_000.0] {
        for (speed, steps) in [(1.0, 60 * 300), (8.0, 60 * 60)] {
            let (error, rebases) = stride_error(radius, speed, steps, true);
            eprintln!("stride {speed} m/s at {radius} m, local: {error:e} m, {rebases} rebases");
            assert!(rebases > 0, "the walk crossed no rebase");
            assert!(error < 1e-5, "local {error} at {speed} m/s, r {radius}");
        }
    }
    let (planet, _) = stride_error(4_800.0, 1.0, 600, false);
    eprintln!("stride 1 m/s at 4800 m, planet-centred: {planet:e} m");
    assert!(planet > 1e-5, "planet-centred {planet}");
}

/// A rebase moves every animal's feet and goal by the shift, so each is
/// where it was in the planet's frame.
#[test]
fn a_rebase_leaves_the_yard_where_it_was() {
    use bevy::math::DVec3;
    use pbd_core::world_frame::Rebase;
    let home = crate::sim_origin::HOME;
    let o = SimOrigin::new(home, DVec3::new(4352.0, 1920.0, -256.0));
    let pos = Vec3::new(12.25, -3.5, 40.125);
    let mut beast = Beast::new(
        Who::Wild { seed: 7 },
        0,
        Sex::F,
        Genome::default(),
        pos,
        o.up_at(pos),
        Range::Around(o.up_at(pos), 6.0),
        7,
    );
    beast.goal = Some(pos + Vec3::X * 2.0);
    let (feet, goal) = (o.to_world(pos).pos, o.to_world(pos + Vec3::X * 2.0).pos);
    let mut yard = Farmyard::default();
    yard.beasts.push(beast);
    let delta = DVec3::new(128.0, 0.0, -128.0);
    let r = Rebase {
        shift: delta.as_vec3(),
        delta,
    };
    yard.rebase(r.shift);
    let n = o.rebased(r);
    let b = &yard.beasts[0];
    assert!((n.to_world(b.pos).pos - feet).length() < 1e-6);
    assert!((n.to_world(b.goal.unwrap()).pos - goal).length() < 1e-6);
}
