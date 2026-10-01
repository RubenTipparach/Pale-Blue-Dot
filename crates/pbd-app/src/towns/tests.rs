use super::*;
use crate::sites::{list_spawn, load_rules};

/// Holbrook, as a new world's list places it.
fn holbrook() -> Site {
    let list = pbd_core::sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    );
    list.sites
        .into_iter()
        .find(|s| s.home)
        .expect("a home town")
}

/// Slice 1: the village template is laid at Holbrook on the game's own
/// cells: every layout cell charted, the ground terraced under what it
/// builds and eased round it, and every building cut, standing on the
/// terrace.
#[test]
fn holbrook_is_laid_out_on_its_own_ground() {
    let site = holbrook();
    assert_eq!(site.kind, SiteKind::Village);
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let kits = load_kits();
    let repeats = load_repeats();
    let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
    let town = lay_out(&site, &template, &config).unwrap_or_else(|e| panic!("{e}"));
    let laid = build(&site, &town, None, &kits, &repeat, &config, sheet(&config))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        laid.chart.cells.len(),
        town.cells.len(),
        "every footprint cell charted from the record"
    );
    for b in &town.buildings {
        for [c, r] in &b.cells {
            assert!(laid.chart.cell(*c, *r).is_some(), "{}: ({c}, {r})", b.name);
        }
    }
    let (footprint, margin) = laid.ground.counts();
    assert!(footprint > 300, "{footprint} footprint cells");
    let anchor = laid.patch.cells[laid
        .chart
        .cell(laid.chart.anchor.0, laid.chart.anchor.1)
        .unwrap()]
    .direction;
    let at = laid
        .ground
        .at(anchor)
        .expect("the anchor is on the town's ground");
    assert_eq!(at.ring, 0);
    assert_eq!(at.height(laid.terrace_m + 3.0), laid.terrace_m);
    assert!(laid.terrace_m > config.sea_level_m, "dry");
    let _ = margin;
    let triangles: usize = laid.meshes.values().map(|m| m.positions.len() / 3).sum();
    assert!(triangles > 5_000, "{triangles}");
    for (name, m) in &laid.meshes {
        assert!(repeats.contains_key(name), "{name} has no texture");
        for p in &m.positions {
            let h = Vec3::from_array(*p).length() - config.radius_m - laid.terrace_m;
            assert!(
                (-0.5..16.0).contains(&h),
                "{name}: {h:.2} m over the terrace"
            );
        }
    }
}

fn temporary(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-towns-{name}-{}-{}",
        std::process::id(),
        crate::saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Slice 3a: a world's first open stores its town, buildings first and
/// the settlement last, as the world's creation. Every later open builds
/// the stored town and writes nothing, even with the template changed; the
/// changed template is what a new world would get.
#[test]
fn a_world_stores_its_town_once_and_keeps_it_when_the_template_changes() {
    use crate::saves::{self, LOG, WorldSave};
    let root = temporary("keep");
    let slot = saves::create(&root, "Holbrook", 41).unwrap();
    let config = *crate::planet::terrain_config();
    let site = holbrook();
    let template = load_template("village");
    let made = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        assert_eq!(stored(&save, site.id), Stored::None, "no town yet");
        let (town, seq) = ensure(&mut save, &site, &template, &config).expect("laid");
        let town = town.expect("a town, not an unsettled site");
        assert!(seq > 0, "queued to the disk");
        save.drain();
        assert!(save.committed() >= seq, "and on it");
        town
    };
    let path = root.join(&slot.id).join(LOG);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rec @c building "))
            .count(),
        template.buildings.len(),
        "a line per building, the creation's"
    );
    let last = text.trim_end().lines().last().unwrap();
    assert!(
        last.starts_with(&format!("rec @c settlement {} 1 ", site.id)),
        "the settlement last"
    );
    let identity = saves::read_identity(&root.join(&slot.id)).expect("an identity");
    assert_eq!(identity.records.get("settlement"), Some(&1));
    assert_eq!(identity.records.get("building"), Some(&1));

    let mut revised = template.clone();
    revised.buildings.remove(0);
    revised.buildings[0].storeys += 1;
    let mut reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    let (kept, seq) = ensure(&mut reopened, &site, &revised, &config).expect("kept");
    assert_eq!(kept, Some(made.clone()), "the world keeps its town");
    assert_eq!(seq, 0, "and nothing is written");
    reopened.drain();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "byte for byte"
    );
    assert_ne!(
        lay_out(&site, &revised, &config).unwrap(),
        made,
        "a new world would get the revision"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A settlement record this build cannot read is never taken for no town:
/// no town is built and nothing is written over it.
#[test]
fn a_damaged_settlement_is_neither_built_nor_written_over() {
    use crate::saves::WorldSave;
    use pbd_core::records::Record;
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let mut save = WorldSave::memory_only();
    save.store(
        &Author::Creation,
        vec![Record {
            kind: record::SETTLEMENT_RECORD.into(),
            id: u64::from(site.id),
            schema: record::RECORD_SCHEMA + 1,
            body: "(later: true)".into(),
        }],
    )
    .expect("stored");
    let before = save.records.len();
    let made = ensure(&mut save, &site, &load_template("village"), &config);
    assert!(made.is_err(), "{made:?}");
    assert_eq!(save.records.len(), before, "nothing written");
}

/// The town's ground is generation (the design's slice 3a): the terrace
/// over its footprint and the margin eased to the natural ground. It is
/// pinned here as a generator version's ground is, so a change to the
/// easing is a new rule for new towns and never reshapes a made one.
#[test]
fn holbrooks_ground_is_pinned() {
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let town = lay_out(&site, &load_template("village"), &config).unwrap();
    let laid = build(
        &site,
        &town,
        None,
        &load_kits(),
        &|_: &str| 2.0,
        &config,
        sheet(&config),
    )
    .unwrap();
    let digest = Ground::new(config, vec![laid.ground.clone()]).digest();
    let (footprint, margin) = laid.ground.counts();
    println!(
        "Holbrook's ground: terrace {} m, {footprint} + {margin} cells, digest {digest:#018x}",
        town.terrace
    );
    assert_eq!(digest, HOLBROOK_GROUND);
}

/// Measured 2026-09-30 on generator 6: a terrace at 74 m over 799 cells, eased
/// over 2051.
const HOLBROOK_GROUND: u64 = 0xc331_13d0_7b63_82f3;

/// A measurement instrument, run by hand: where to stand in Holbrook for
/// the shots set beside the mockup's, as `--at` and `--yaw` for a capture.
/// Outside the Fieldstone house's door looking at it, inside looking out of
/// it (slice 2a's shot), and on the lane looking along it.
#[test]
#[ignore]
fn print_where_to_stand_for_the_mockup_shots() {
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let repeats = load_repeats();
    let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
    let town = lay_out(&site, &template, &config).unwrap();
    let laid = build(
        &site,
        &town,
        None,
        &load_kits(),
        &repeat,
        &config,
        sheet(&config),
    )
    .unwrap();
    let radius = config.radius_m;
    // `--at` and `--yaw` for standing at `stand` looking along `toward`.
    let spot = |name: &str, stand: Vec3, toward: Vec3| {
        let up = stand.normalize();
        let heading = Vec3::Y.cross(up).normalize();
        let flat = (toward - up * toward.dot(up)).normalize();
        let angle = heading.cross(flat).dot(up).atan2(heading.dot(flat));
        let (lat, lon) = pbd_core::geo::lat_lon(up).degrees();
        println!(
            "{name}: --at {lat:.5} {lon:.5} --yaw {:.1}",
            (-angle).to_degrees()
        );
    };
    let house = &template.buildings[0];
    let [c, r, d, _] = house.doors[0];
    let side = laid.chart.side(c, r, d as usize).unwrap();
    let cell = &laid.patch.cells[laid.chart.cell(c, r).unwrap()];
    let (a, e) = (cell.corners[side], cell.corners[(side + 1) % 6]);
    let door = ((a + e) * 0.5).normalize() * radius;
    let out = ((a + e) * 0.5 - cell.direction).normalize();
    spot(
        &format!("outside the {}", house.name),
        door + out * 3.5,
        -out,
    );
    spot(&format!("inside the {}", house.name), door - out * 1.6, out);
    // The lane at layout cell (20, 17), looking along the mockup's +x.
    let lane = &laid.patch.cells[laid.chart.cell(20, 17).unwrap()];
    let (lc, lr) = pbd_core::settlement::neighbour(20, 17, 0);
    let ahead = laid.patch.cells[laid.chart.cell(lc, lr).unwrap()].direction - lane.direction;
    spot("the lane at (20, 17)", lane.direction * radius, ahead);
    // Slice 2b: the Fieldstone house's newel, from the room beside its
    // doorway, below (`--up 0`) and above (`--up 3.2`); the first
    // half-timbered house's flight, from before its foot and from its
    // landing (`--up 3.2`).
    use pbd_core::settlement::pieces::Surface;
    let plan = |b: &pbd_core::settlement::pieces::BuildingSolids, p: Vec2| {
        b.frame.world(Vec3::new(p.x, 0.0, p.y))
    };
    let dir = |b: &pbd_core::settlement::pieces::BuildingSolids, d: Vec2| {
        b.frame.x * d.x + b.frame.z * d.y
    };
    let b = &laid.solids[0];
    if let Some(&Surface::Newel { centre, start, .. }) = b
        .surfaces
        .iter()
        .find(|s| matches!(s, Surface::Newel { .. }))
    {
        let entry = Vec2::new(start.cos(), start.sin());
        spot(
            "the newel, from the room beside it",
            plan(b, centre + entry * 2.6),
            dir(b, -entry),
        );
    }
    let (n, b) = laid
        .solids
        .iter()
        .enumerate()
        .find(|(_, b)| {
            b.surfaces
                .iter()
                .any(|s| matches!(s, Surface::Flight { .. }))
        })
        .expect("a flight");
    if let Some(&Surface::Flight {
        foot, dir: up, len, ..
    }) = b
        .surfaces
        .iter()
        .find(|s| matches!(s, Surface::Flight { .. }))
    {
        let name = &template.buildings[n].name;
        spot(
            &format!("the {name}'s flight, from before its foot"),
            plan(b, foot - up * 1.4),
            dir(b, up),
        );
        spot(
            &format!("the {name}'s flight, from its landing"),
            plan(b, foot + up * (len - 0.35)),
            dir(b, -up),
        );
    }
}

/// A measurement instrument, run by hand: the rain over Holbrook at 11:00,
/// on a new world's first days and after each hour of `--weather-at`, along
/// the capture's own path (`Air::open`, then `run`). Picks a dry sky for the
/// town's shots: `--rain 0` forces no storm, but the weather still rains.
#[test]
#[ignore]
fn print_the_rain_over_holbrook() {
    use pbd_core::atmosphere::AtmosphereSettings;
    use pbd_core::daylight::Clock;
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/atmosphere.ron"
    );
    let settings: AtmosphereSettings =
        ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let here = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: 29.70f32.to_radians(),
        lon: 1.36f32.to_radians(),
    });
    let hour = (3600.0 / settings.dt_s).ceil() as u32;
    for (day, at) in (0..3).flat_map(|d| [(d, 11.0), (d, 22.5)]) {
        let clock = Clock::at(day, at);
        let mut air = crate::atmosphere::Air::open(
            settings,
            crate::planet::terrain_config().seed,
            None,
            clock.seconds,
        );
        let mut line = format!("--day {day} --time {at}, --weather-at 0, 3600, ...:");
        for k in 0..6 {
            if k > 0 {
                air.run(hour, &[]);
            }
            line += &format!(" {:.2}", pbd_core::weather::rain_at(&air.now, here));
        }
        println!("{line}");
    }
}

/// Slice 2b: a door the player opened is written to the save as theirs, and
/// a reopened world finds it open, and every other door shut.
#[test]
fn a_door_opened_is_open_when_the_world_is_opened_again() {
    use crate::saves::{self, LOG, WorldSave};
    let root = temporary("door");
    let slot = saves::create(&root, "Doors", 43).unwrap();
    let config = *crate::planet::terrain_config();
    let site = holbrook();
    let template = load_template("village");
    let town = lay_out(&site, &template, &config).unwrap();
    let laid = build(
        &site,
        &town,
        None,
        &load_kits(),
        &|_: &str| 2.0,
        &config,
        sheet(&config),
    )
    .unwrap();
    let door = record::door_id(
        record::building_id(site.id, 0),
        laid.solids[0].doors[0].index,
    );
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        save.store(&Author::Player(0), vec![record::door_record(door, true)])
            .expect("stored");
        save.drain();
    }
    let text = std::fs::read_to_string(root.join(&slot.id).join(LOG)).unwrap();
    assert!(
        text.lines()
            .any(|l| l.starts_with(&format!("rec @p door {door} 1 "))),
        "the player's line: {text}"
    );
    let reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    let mut solids = laid.solids.clone();
    door_states(&mut solids, site.id, Some(&reopened.records), false);
    assert!(solids[0].doors[0].open, "open again");
    let shut = solids
        .iter()
        .flat_map(|b| &b.doors)
        .filter(|d| !d.open)
        .count();
    assert_eq!(
        shut,
        solids.iter().map(|b| b.doors.len()).sum::<usize>() - 1
    );
    door_states(&mut solids, site.id, None, true);
    assert!(
        solids.iter().flat_map(|b| &b.doors).all(|d| d.open),
        "--open-doors"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// `sun-shadows` decision 7 and task 5.1: every Holbrook building has rooms
/// drawn apart from the town's outside, a room takes the shut share of the
/// sky until a door of its building opens and the open share after, and the
/// town casts every triangle of both.
#[test]
fn holbrooks_rooms_take_their_share_of_the_sky_and_the_town_casts() {
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let town = lay_out(&site, &load_template("village"), &config).unwrap();
    let laid = build(
        &site,
        &town,
        None,
        &load_kits(),
        &|_: &str| 2.0,
        &config,
        sheet(&config),
    )
    .unwrap();
    assert_eq!(laid.rooms.len(), laid.solids.len());
    for (b, rooms) in laid.rooms.iter().enumerate() {
        assert!(!rooms.is_empty(), "building {b} has no rooms");
    }
    let triangles = |m: &Meshes| m.values().map(|b| b.positions.len()).sum::<usize>();
    let outside = triangles(&laid.meshes);
    let inside: usize = laid.rooms.iter().map(triangles).sum();
    assert!(
        inside > 1000 && outside > inside,
        "{inside} inside, {outside} outside"
    );
    let flames = laid
        .rooms
        .iter()
        .filter_map(|m| m.get(FLAME))
        .map(|b| b.positions.len())
        .sum::<usize>();
    assert!(flames > 0, "the hearths and sconces burn");
    assert_eq!(casting(&laid).len(), outside + inside - flames);
    // Decision 7a: a building's rooms carry its lights, a house's hearth
    // first, into their material.
    let mut field = crate::field_light::FieldUniform::default();
    crate::field_light::RoomLights(laid.lights[0].clone()).pack(&mut field);
    assert!(field.look.w >= 2.0, "a hearth and a sconce at least");
    assert_eq!(field.light_colour[0].w, 0.0, "a fire first");
    for i in 0..field.look.w as usize {
        let at = field.lights[i];
        assert!(at.w > 4.0 && (at.truncate().length() - 4800.0).abs() < 400.0);
        assert!(field.light_span[i].x < 0.0 && field.light_span[i].y > 0.0);
    }

    let shut = SkyShare {
        sky: ROOM_SKY_SHUT,
        bounce: ROOM_BOUNCE,
    };
    let open = SkyShare {
        sky: ROOM_SKY_OPEN,
        bounce: ROOM_BOUNCE,
    };
    let mut world = World::new();
    world.insert_resource(Structures(laid.solids.clone()));
    world.insert_resource(Towns::standing_alone(site.id, laid.solids.len()));
    let room = world
        .spawn((
            TownRoom {
                site: site.id,
                building: 0,
            },
            shut,
        ))
        .id();
    let other = world
        .spawn((
            TownRoom {
                site: site.id,
                building: 1,
            },
            shut,
        ))
        .id();
    let mut follow = IntoSystem::into_system(rooms_follow_doors);
    follow.initialize(&mut world);
    follow.run((), &mut world).unwrap();
    assert_eq!(world.get::<SkyShare>(room), Some(&shut));
    world.resource_mut::<Structures>().0[0].doors[0].open = true;
    follow.run((), &mut world).unwrap();
    assert_eq!(world.get::<SkyShare>(room), Some(&open));
    assert_eq!(
        world.get::<SkyShare>(other),
        Some(&shut),
        "another building's door is not this one's"
    );
}

/// Every village site of a new world's list, by id.
fn villages() -> Vec<Site> {
    let mut sites: Vec<Site> = pbd_core::sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    )
    .sites
    .into_iter()
    .filter(|s| s.kind == SiteKind::Village)
    .collect();
    sites.sort_by_key(|s| s.id);
    sites
}

/// Slice 4a: every village site of the shipped seed lays and cuts on its
/// own ground, dry, turned by its seed, and the grounds of two never meet.
#[test]
fn every_village_lays_and_cuts_on_its_own_ground() {
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let kits = load_kits();
    let villages = villages();
    assert!(villages.len() >= 10, "{} villages", villages.len());
    let mut turns = std::collections::BTreeSet::new();
    for site in &villages {
        let started = std::time::Instant::now();
        let town =
            lay_out(site, &template, &config).unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let laid = build(
            site,
            &town,
            Some(&template),
            &kits,
            &|_: &str| 2.0,
            &config,
            sheet(&config),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let (footprint, margin) = laid.ground.counts();
        println!(
            "{}: turn {}, a terrace at {} m over {footprint} cells eased over {margin}, laid and cut in {:.2} s",
            site.name,
            if site.home { 0 } else { record::turn(site.id) },
            town.terrace,
            started.elapsed().as_secs_f32()
        );
        assert!(footprint > 300, "{}: {footprint} cells", site.name);
        assert_eq!(laid.solids.len(), template.buildings.len(), "{}", site.name);
        assert!(laid.terrace_m > config.sea_level_m, "{} is dry", site.name);
        if !site.home {
            turns.insert(record::turn(site.id));
        }
    }
    assert!(
        turns.len() >= 3,
        "the villages face several ways: {turns:?}"
    );
    for (i, a) in villages.iter().enumerate() {
        for b in &villages[i + 1..] {
            let apart = a.direction.angle_between(b.direction) * config.radius_m;
            assert!(
                apart > 2.0 * PATCH_M,
                "{} and {}: {apart} m",
                a.name,
                b.name
            );
        }
    }
}

/// Slice 4a: a new world stores every village once, and opening it again
/// writes nothing.
#[test]
fn a_world_stores_every_village_once() {
    use crate::saves::{self, LOG, WorldSave};
    let root = temporary("villages");
    let slot = saves::create(&root, "Villages", 41).unwrap();
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let villages = villages();
    let made: Vec<Town> = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let made = villages
            .iter()
            .map(|site| {
                let (town, seq) = ensure(&mut save, site, &template, &config).expect("laid");
                assert!(seq > 0, "{} queued to the disk", site.name);
                town.expect("a town")
            })
            .collect();
        save.drain();
        made
    };
    let path = root.join(&slot.id).join(LOG);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rec @c settlement "))
            .count(),
        villages.len(),
        "a settlement line a village"
    );
    let mut reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    for (site, town) in villages.iter().zip(&made) {
        let (kept, seq) = ensure(&mut reopened, site, &template, &config).expect("kept");
        assert_eq!(kept.as_ref(), Some(town), "{} kept", site.name);
        assert_eq!(seq, 0, "{}: nothing written", site.name);
    }
    reopened.drain();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "byte for byte"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Task 4.4: a village whose ground the player changed before it was laid
/// is left unsettled, and stays so once the edit is gone.
#[test]
fn a_village_on_the_players_work_is_left_unsettled() {
    use crate::saves::WorldSave;
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let site = villages()
        .into_iter()
        .find(|s| !s.home)
        .expect("a village away from home");
    let patch = patch_round(site.direction, config.radius_m, PATCH_M);
    let cell = patch.keys[patch.nearest(site.direction).unwrap()];
    let mut save = WorldSave::memory_only();
    save.edits.set(pbd_core::edits::Edit {
        cell,
        layer: 60,
        material: pbd_core::terrain::Material::Air,
    });
    let (town, _) = ensure(&mut save, &site, &template, &config).expect("decided");
    assert!(town.is_none(), "no town on the player's work");
    assert_eq!(stored(&save, site.id), Stored::Unsettled);
    save.edits = pbd_core::edits::Edits::new();
    let (town, seq) = ensure(&mut save, &site, &template, &config).expect("decided");
    assert!(
        town.is_none() && seq == 0,
        "unsettled for good, and nothing written"
    );
}

/// Slice 4a: a town stands within 1.2 km of the viewer and goes past
/// 1.5 km, and between the two it stays as it was; each way it fades over a
/// second, a frame's share at a time.
#[test]
fn a_town_stands_in_range_goes_past_it_and_fades_each_way() {
    assert!(wanted(STAND_M - 1.0) && !wanted(STAND_M + 1.0));
    assert!(!leaving(DROP_M - 1.0, false), "standing, and in the gap");
    assert!(leaving(DROP_M + 1.0, false), "past the drop");
    assert!(leaving(DROP_M - 1.0, true), "going, and in the gap");
    assert!(!leaving(STAND_M - 1.0, true), "back in range");
    let frame = 1.0 / 60.0;
    for going in [false, true] {
        let mut shown = if going { 1.0 } else { 0.0 };
        let mut frames = 0;
        while shown != if going { 0.0 } else { 1.0 } {
            let next = faded(shown, going, frame);
            assert!((next - shown).abs() <= frame / FADE_S + 1e-6, "a step");
            shown = next;
            frames += 1;
        }
        assert!((59..=61).contains(&frames), "{frames} frames");
    }
    assert_eq!(
        faded(0.0, false, 0.4),
        0.4,
        "a hitch is a longer step, not a jump"
    );
}

/// Slice 4a, in a running app: a village stands, cut on the pool and faded
/// in, when the walker comes within range, and gives the walker its
/// buildings; it fades out and is taken down, buildings and all, when the
/// walker goes on past.
#[test]
fn a_village_stands_as_the_walker_comes_and_is_taken_down_as_it_leaves() {
    use bevy::time::TimeUpdateStrategy;
    let config = *crate::planet::terrain_config();
    let site = villages()
        .into_iter()
        .find(|s| !s.home)
        .expect("a village away from home");
    let town = lay_out(&site, &load_template("village"), &config).unwrap();
    let held = Held {
        site: site.clone(),
        town,
    };
    let point = held.point(config.radius_m) + site.direction * 1.6;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Image>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(1.0 / 60.0),
        ))
        .insert_resource(TownAssets {
            kits: Arc::new(load_kits()),
            village: load_template("village"),
            walled: load_template("town"),
            harbour: load_template("coast"),
            repeats: Arc::new(load_repeats()),
        })
        .insert_resource(Towns {
            held: vec![held],
            ..default()
        })
        .insert_resource(Structures::default())
        .add_systems(Update, stand_in_range);
    let walker = app
        .world_mut()
        .spawn((Walker, avian3d::prelude::Position(point)))
        .id();
    let standing = |app: &App| app.world().resource::<Towns>().standing.clone();
    let mut frames = 0;
    while standing(&app).is_empty() {
        assert!(frames < 3000, "the village never stood");
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(5));
        frames += 1;
    }
    let root = standing(&app)[0].entity;
    assert!(
        app.world().get::<Faded>(root).is_some_and(|f| f.0 > 0.9),
        "it comes in faded"
    );
    assert_eq!(
        app.world().resource::<Structures>().0.len(),
        load_template("village").buildings.len(),
        "the walker has its buildings"
    );
    for _ in 0..70 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Faded>(root),
        Some(&Faded(0.0)),
        "whole in a second"
    );
    // Past the drop: it fades out and goes.
    let (_, east) = pbd_core::geo::north_east(site.direction);
    app.world_mut()
        .get_mut::<avian3d::prelude::Position>(walker)
        .unwrap()
        .0 = point + east * (DROP_M + 100.0);
    app.update();
    assert!(!standing(&app).is_empty(), "not gone in one frame");
    for _ in 0..70 {
        app.update();
    }
    assert!(standing(&app).is_empty(), "gone after its fade");
    assert!(app.world().get_entity(root).is_err(), "its meshes with it");
    assert!(
        app.world().resource::<Structures>().0.is_empty(),
        "and its buildings"
    );
}

/// A measurement instrument, run by hand (slice 4a): what a height costs
/// with every village's ground installed, against none. A height in no town
/// pays one dot product a town. It installs the ground for the whole
/// process, so it is run alone:
/// `cargo test --release -p pbd-app --lib print_the_cost_of_a_height_with_the_villages -- --ignored --nocapture`
#[test]
#[ignore]
fn print_the_cost_of_a_height_with_the_villages() {
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let grounds: Vec<TownGround> = villages()
        .iter()
        .map(|site| {
            let town = lay_out(site, &template, &config).unwrap();
            let patch = patch_round(site.direction, config.radius_m, PATCH_M);
            record::ground_of(&town, &patch, config.radius_m, natural(&config))
                .unwrap()
                .1
        })
        .collect();
    let count = grounds.len();
    // Directions spread over the sphere, as the far terrain asks them.
    let directions: Vec<Vec3> = (0..200_000u32)
        .map(|i| {
            let t = (i as f32 + 0.5) / 200_000.0;
            let z = 1.0 - 2.0 * t;
            let r = (1.0 - z * z).sqrt();
            let a = i as f32 * 2.399_963;
            Vec3::new(r * a.cos(), z, r * a.sin())
        })
        .collect();
    let time = |label: &str| {
        let started = std::time::Instant::now();
        let mut sum = 0.0f32;
        for d in &directions {
            sum += pbd_core::column::surface_m(&config, *d);
        }
        let ns = started.elapsed().as_nanos() as f64 / directions.len() as f64;
        println!("{label}: {ns:.0} ns a height ({sum:.0})");
        ns
    };
    ground::install(None);
    let none = time("no towns");
    let home = villages().iter().position(|s| s.home).unwrap();
    ground::install(Some(Ground::new(config, vec![grounds[home].clone()])));
    let one = time("the home village alone");
    ground::install(Some(Ground::new(config, grounds)));
    let all = time(&format!("{count} villages"));
    ground::install(None);
    println!(
        "{:+.1}% a height with the home village, {:+.1}% with all",
        (one / none - 1.0) * 100.0,
        (all / none - 1.0) * 100.0
    );
}

/// Slice 4b: every walled site of the shipped seed lays and cuts the walled
/// town on its own ground, on its levels, dry.
#[test]
fn every_walled_town_lays_and_cuts_on_its_levels() {
    let config = *crate::planet::terrain_config();
    let template = load_template("town");
    let kits = load_kits();
    let mut sites: Vec<Site> = pbd_core::sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    )
    .sites
    .into_iter()
    .filter(|s| s.kind == SiteKind::Walled)
    .collect();
    sites.sort_by_key(|s| s.id);
    assert!(!sites.is_empty(), "walled sites");
    for site in &sites {
        let started = std::time::Instant::now();
        let town =
            lay_out(site, &template, &config).unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let laid = build(
            site,
            &town,
            Some(&template),
            &kits,
            &|_: &str| 2.0,
            &config,
            sheet(&config),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let (footprint, margin) = laid.ground.counts();
        let levels: std::collections::BTreeSet<i8> = town.levels.iter().copied().collect();
        let (lat, lon) = pbd_core::geo::lat_lon(site.direction).degrees();
        println!(
            "{}{}: --at {lat:.5} {lon:.5}, turn {}, a terrace at {} m on levels {levels:?}, {footprint} cells eased over {margin}, laid and cut in {:.2} s",
            site.name,
            if site.capital { " (the capital)" } else { "" },
            record::turn(site.id),
            town.terrace,
            started.elapsed().as_secs_f32()
        );
        assert_eq!(
            laid.solids.len(),
            template.buildings.len() + template.masonry.len(),
            "{}: every building and every cell of the wall",
            site.name
        );
        assert!(levels.len() >= 3, "{}: levels {levels:?}", site.name);
        let lowest = town.terrace as f32 + f32::from(*levels.first().unwrap());
        assert!(lowest > config.sea_level_m, "{} is dry", site.name);
    }
}

fn harbours() -> Vec<Site> {
    let mut sites: Vec<Site> = pbd_core::sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    )
    .sites
    .into_iter()
    .filter(|s| s.kind == SiteKind::Harbour)
    .collect();
    sites.sort_by_key(|s| s.id);
    sites
}

/// Slice 4d: every harbour site of the shipped seed lies with its sea over
/// the planet's, stands on the sea with its quay a metre over the water,
/// and cuts whole: its buildings, its piers' stretches, its light, its
/// dressing and its cog.
#[test]
fn every_harbour_lays_on_its_sea_and_cuts() {
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let kits = load_kits();
    let sites = harbours();
    assert!(!sites.is_empty(), "harbour sites");
    for site in &sites {
        let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
        let at = patch.nearest(site.direction).unwrap();
        let (_, east) = pbd_core::geo::north_east(patch.cells[at].direction);
        let d0 = (patch.side_toward(at, east) + record::turn(site.id)) % 6;
        let started = std::time::Instant::now();
        let (_, _, share) = sea::placement(
            &template,
            &patch,
            at,
            d0,
            record::template_anchor(&template),
            natural(&config),
            config.sea_level_m,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let placed = started.elapsed().as_secs_f32();
        let town =
            lay_out(site, &template, &config).unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let laid = build(
            site,
            &town,
            Some(&template),
            &kits,
            &|_: &str| 2.0,
            &config,
            sheet(&config),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", site.name));
        let (lat, lon) = pbd_core::geo::lat_lon(site.direction).degrees();
        println!(
            "{}: --at {lat:.5} {lon:.5}, {:.0}% of its cells agree about the sea, placed in {placed:.2} s, laid and cut in {:.2} s, {} pieces",
            site.name,
            share * 100.0,
            started.elapsed().as_secs_f32(),
            laid.solids.len()
        );
        assert!(share >= 0.75, "{}: {:.0}% agree", site.name, share * 100.0);
        assert_eq!(
            town.terrace as f32,
            config.sea_level_m.floor(),
            "{}",
            site.name
        );
        let quay = town
            .cells
            .iter()
            .position(|c| (c.0, c.1) == (20, 15))
            .unwrap_or_else(|| panic!("{}: no quay", site.name));
        assert_eq!(town.terrace_of(quay), 1.0, "{}: the quay", site.name);
        assert!(
            laid.solids.len() > template.buildings.len() + template.piers.len(),
            "{}: buildings, piers and the light",
            site.name
        );
        // Task 4.2c: its dressing, every thing on its chart, and the
        // shipyard's hull, planks and slip.
        assert_eq!(
            laid.dressing,
            (template.dressing.len() + 3, 0),
            "{}: its dressing",
            site.name
        );
        // `sail-the-cog` design 6, step 1: its cog at its mooring.
        assert!(laid.cog.is_some(), "{}: its cog", site.name);
    }
}

/// Slice 4d: a world stores each harbour once, in schema 3, and a second
/// open writes nothing.
#[test]
fn a_world_stores_every_harbour_once() {
    use crate::saves::{self, LOG, WorldSave};
    let root = temporary("harbours");
    let slot = saves::create(&root, "Harbours", 41).unwrap();
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let sites = harbours();
    let made: Vec<Town> = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let made = sites
            .iter()
            .map(|site| {
                let (town, seq) = ensure(&mut save, site, &template, &config).expect("laid");
                assert!(seq > 0, "{} queued to the disk", site.name);
                town.expect("a town")
            })
            .collect();
        save.drain();
        made
    };
    let path = root.join(&slot.id).join(LOG);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rec @c settlement "))
            .count(),
        sites.len(),
        "a settlement line a harbour"
    );
    let mut reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    for (site, town) in sites.iter().zip(&made) {
        let (kept, seq) = ensure(&mut reopened, site, &template, &config).expect("kept");
        assert_eq!(kept.as_ref(), Some(town), "{} kept", site.name);
        assert_eq!(seq, 0, "{}: nothing written", site.name);
    }
    reopened.drain();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "byte for byte"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An instrument for the slice 4d shots: where a harbour's pieces stand on
/// the shipped seed, as `--at` takes them. A harbour lies up to its shift
/// from its site's marker, so its shots are taken from where it stands.
#[test]
#[ignore = "an instrument: prints where to stand for the harbour shots"]
fn print_where_the_harbours_stand() {
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let kits = load_kits();
    for site in harbours() {
        let town = lay_out(&site, &template, &config).unwrap();
        let laid = build(
            &site,
            &town,
            Some(&template),
            &kits,
            &|_: &str| 2.0,
            &config,
            sheet(&config),
        )
        .unwrap();
        let at = |d: Vec3| {
            let (lat, lon) = pbd_core::geo::lat_lon(d).degrees();
            format!("--at {lat:.5} {lon:.5}")
        };
        let cell = |c: i32, r: i32| laid.patch.cells[laid.chart.cell(c, r).unwrap()].direction;
        let cell_m = template.grid.cell_m;
        let pier = &template.piers[0];
        let head = sea::point(&laid.chart, &laid.patch, pier.to[0], pier.to[2], cell_m).unwrap();
        let quay =
            sea::point(&laid.chart, &laid.patch, pier.from[0], pier.from[2], cell_m).unwrap();
        // `--yaw` from one point toward another, as the walker's start reads
        // it: its heading is `Y x up` turned by `-yaw` about up.
        let yaw = |from: Vec3, to: Vec3| {
            let up = from.normalize();
            let base = Vec3::Y.cross(up).normalize();
            let along = to - from;
            let t = (along - up * along.dot(up)).normalize();
            -base.cross(t).dot(up).atan2(base.dot(t)).to_degrees()
        };
        let point = |x: f32, z: f32| sea::point(&laid.chart, &laid.patch, x, z, cell_m).unwrap();
        let mid = |c: i32, r: i32| sea::centre(c, r, cell_m);
        println!(
            "{}: footprint {}; quay {} --yaw {:.1}; pier head {}; fish hut {}; boathouse {}; marker {}",
            site.name,
            at(laid.ground.anchor()),
            at(quay),
            yaw(quay, head),
            at(head),
            at(cell(4, 9)),
            at(cell(9, 12)),
            at(site.direction)
        );
        // Task 4.2c: where to stand for the fish market (from the harbour
        // street, over a stall to the racks on the beach) and the shipyard
        // (the mockup's own view of it).
        let ((sx, sz), (tx, tz)) = (mid(21, 16), mid(16, 14));
        let (street, racks) = (point(sx, sz), point(tx, tz));
        println!("   market {} --yaw {:.1}", at(street), yaw(street, racks));
        if let Some(c) = &template.cog {
            println!("   cog {}", at(point(c.x, c.z)));
        }
        if let Some(y) = &template.shipyard {
            let from = point(y.x - 7.0, mid(0, 14).1 + 0.9);
            let hull = point(y.x, y.z);
            println!("   shipyard {} --yaw {:.1}", at(from), yaw(from, hull));
        }
        // Holes: a cell left unlaid with every neighbour laid, or one laid
        // lower than all of its own.
        let level: std::collections::BTreeMap<(i32, i32), f32> = town
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| ((c.0, c.1), town.terrace_of(i)))
            .collect();
        for g in &template.ground {
            let (c, r) = (g.c, g.r);
            let around: Vec<Option<f32>> = (0..6)
                .map(|d| {
                    let n = pbd_core::settlement::neighbour(c, r, d);
                    level.get(&n).copied()
                })
                .collect();
            if around.iter().all(|a| a.is_some()) {
                match level.get(&(c, r)) {
                    None => println!("   hole at ({c}, {r}), template {} {}", g.h, g.top),
                    Some(&l) if around.iter().all(|a| a.unwrap() > l) => {
                        println!("   pit at ({c}, {r}) level {l}, template {} {}", g.h, g.top)
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Task 4.2b: each harbour of the shipped seed moors its boats once, each
/// afloat at its berth, anchored and tagged with its berth; the berths over
/// land or too shallow are skipped and counted; asked again, it makes none.
#[test]
fn every_harbours_boats_are_made_once_at_their_berths() {
    use crate::vehicles::harbour::boats_for;
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let fleet = crate::vehicles::Fleet::new(crate::config::VehiclesConfig::default().0);
    // The drawn sea, which the game's boats float on.
    let sea_radius = config.radius_m + sheet_m(&config, &crate::config::WaterSettings::default());
    let mut total = 0;
    for site in harbours() {
        let town = lay_out(&site, &template, &config).unwrap();
        let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
        let mut next = 1;
        let (made, skipped) = boats_for(
            site.id,
            &town,
            &template,
            &patch,
            sea_radius,
            &Default::default(),
            &fleet.specs,
            &fleet.hulls,
            &mut next,
            crate::vehicles::place::floor,
        );
        println!(
            "{}: {} boats moored ({} Terns), {skipped} berths skipped",
            site.name,
            made.len(),
            made.iter()
                .filter(|c| c.kind == pbd_core::vehicle::Kind::Tern)
                .count()
        );
        assert_eq!(made.len() + skipped, template.boats.len(), "{}", site.name);
        let berths: std::collections::BTreeSet<(u32, u32)> =
            made.iter().filter_map(|c| c.berth).collect();
        assert_eq!(berths.len(), made.len(), "{}: a berth a boat", site.name);
        for c in &made {
            assert!(
                c.mooring.is_some_and(|m| m.anchored),
                "{}: anchored",
                site.name
            );
            let afloat = c.reference_position().length() - f64::from(sea_radius);
            assert!(
                afloat.abs() < 0.5,
                "{}: at the sea, {afloat:.2} m",
                site.name
            );
        }
        assert_eq!(next, made.len() as u64 + 1);
        let all: std::collections::BTreeSet<(u32, u32)> = (0..template.boats.len() as u32)
            .map(|n| (site.id, n))
            .collect();
        let (again, _) = boats_for(
            site.id,
            &town,
            &template,
            &patch,
            sea_radius,
            &all,
            &fleet.specs,
            &fleet.hulls,
            &mut next,
            crate::vehicles::place::floor,
        );
        assert!(again.is_empty(), "{}: made once", site.name);
        total += made.len();
    }
    assert!(total > 0, "some boats moored");
}

/// `sail-the-cog` step 3, part 3: each harbour of the shipped seed makes its
/// cog once, at its berth, on a bollard and on the drawn sea, riding its
/// swing; asked again, it makes none.
#[test]
fn every_harbours_cog_is_made_once_at_its_berth() {
    use crate::vehicles::harbour::{COG, cog_berth, cog_for, on_swing};
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let fleet = crate::vehicles::Fleet::new(crate::config::VehiclesConfig::default().0);
    let sea_radius = config.radius_m + sheet_m(&config, &crate::config::WaterSettings::default());
    let mut made = 0;
    for site in harbours() {
        let town = lay_out(&site, &template, &config).unwrap();
        let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
        let mut next = 1;
        let make = |made: &std::collections::BTreeSet<(u32, u32)>, next: &mut u64| {
            cog_for(
                site.id,
                &town,
                &template,
                &patch,
                config.radius_m,
                sea_radius,
                made,
                &fleet.specs,
                &fleet.hulls,
                next,
            )
        };
        let cog = make(&Default::default(), &mut next)
            .unwrap_or_else(|| panic!("{}: its cog", site.name));
        assert_eq!(cog.kind, pbd_core::vehicle::Kind::Cog);
        assert_eq!(cog.berth, Some((site.id, COG)), "{}", site.name);
        assert!(on_swing(&cog), "{}: on its bollard", site.name);
        let (origin, _) = cog_berth(&town, &template, &patch, config.radius_m, sea_radius).unwrap();
        assert!(
            cog.reference_position().distance(origin.as_dvec3()) < 1e-3,
            "{}: at its berth",
            site.name
        );
        let afloat = cog.reference_position().length() - f64::from(sea_radius);
        assert!(
            afloat.abs() < 0.01,
            "{}: on the drawn sea, {afloat:.3} m",
            site.name
        );
        assert!(
            make(&[(site.id, COG)].into_iter().collect(), &mut next).is_none(),
            "{}: made once",
            site.name
        );
        made += 1;
    }
    assert!(made > 0, "some cogs made");
}

/// An instrument for the finding in Coringport's beach (2026-10-01): how
/// many of each town's footprint cells the planet's worms open within the
/// top two layers of the town's ground, where a cave mouth or a hole shows.
#[test]
#[ignore = "an instrument: counts cave mouths in towns' ground"]
fn print_where_caves_open_into_towns() {
    use pbd_core::column::layer_at;
    let config = *crate::planet::terrain_config();
    let field = pbd_core::worms::WormField::DEFAULT;
    let assets_village = load_template("village");
    let assets_walled = load_template("town");
    let assets_coast = load_template("coast");
    let sites: Vec<Site> = pbd_core::sites::generate(&load_rules(), &config, list_spawn(), 4)
        .sites
        .into_iter()
        .filter(|s| {
            matches!(
                s.kind,
                SiteKind::Village | SiteKind::Walled | SiteKind::Harbour
            )
        })
        .collect();
    let mut opened_towns = 0;
    for site in &sites {
        let template = match site.kind {
            SiteKind::Village => &assets_village,
            SiteKind::Walled => &assets_walled,
            _ => &assets_coast,
        };
        let town = lay_out(site, template, &config).unwrap();
        let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
        let (chart, _) =
            record::ground_of(&town, &patch, config.radius_m, natural(&config)).unwrap();
        let worms = pbd_core::worms::gather(&field, &config, site.direction, 250.0);
        let mut opened = Vec::new();
        for (i, c) in town.cells.iter().enumerate() {
            let d = patch.cells[chart.cell(c.0, c.1).unwrap()].direction;
            let terrace = town.terrace_of(i);
            let top: Vec<usize> = [terrace - 0.5, terrace - 1.5]
                .iter()
                .filter_map(|&a| layer_at(a))
                .collect();
            let mut hit = false;
            worms.carve(&config, d, field.floor_layers, |index| {
                hit |= top.contains(&index)
            });
            if hit {
                opened.push((c.0, c.1));
            }
        }
        if let Some(&(c, r)) = opened.first() {
            let d = patch.cells[chart.cell(c, r).unwrap()].direction;
            let (lat, lon) = pbd_core::geo::lat_lon(d).degrees();
            println!("   the first at --at {lat:.5} {lon:.5}");
        }
        if !opened.is_empty() {
            opened_towns += 1;
            println!(
                "{} ({:?}): {} of {} footprint cells open to a cave in their top two layers, {:?}",
                site.name,
                site.kind,
                opened.len(),
                town.cells.len(),
                &opened[..opened.len().min(6)]
            );
        }
    }
    println!("{opened_towns} of {} towns", sites.len());
}

/// Instrument (`cities-in-the-world` task 4.2b, found 2026-10-01): the
/// seabed under each harbour's berths, the planet's own and as the
/// harbour's ground, with its margin eased to its terrace, has it.
#[test]
#[ignore = "instrument: prints the seabed under each harbour's berths"]
fn print_the_seabed_under_the_harbours_berths() {
    let config = *crate::planet::terrain_config();
    let template = load_template("coast");
    let sea_m = config.sea_level_m;
    for site in harbours() {
        let town = lay_out(&site, &template, &config).unwrap();
        let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
        let (chart, ground) =
            record::ground_of(&town, &patch, config.radius_m, natural(&config)).unwrap();
        let nat = natural(&config);
        let (mut raised, mut shallow_then, mut shallow_now) = (0, 0, 0);
        for boat in &template.boats {
            let Some((d, _)) = sea::boat_pose(&chart, &patch, boat, template.grid.cell_m) else {
                continue;
            };
            let n = nat(d);
            let eased = ground.at(d).map_or(n, |g| g.height(n));
            if eased > n + 0.01 {
                raised += 1;
            }
            shallow_then += usize::from(sea_m - n.floor() < 1.0);
            shallow_now += usize::from(sea_m - eased.floor() < 1.0);
        }
        println!(
            "{}: {} berths, {raised} with the seabed raised by the margin; under a metre of water: {shallow_then} on the planet's own seabed, {shallow_now} on the harbour's",
            site.name,
            template.boats.len()
        );
    }
}

/// The drawn sea's surface over the radius, as the game draws it.
fn sheet(config: &TerrainConfig) -> f32 {
    sheet_m(config, &crate::config::WaterSettings::default())
}
