use super::*;
use pbd_core::edits::Edit;

/// Two villages on the shipped seed's start, `apart_m` either side of it.
fn villages(apart_m: f32) -> Vec<Site> {
    let config = crate::planet::terrain_config();
    let start = crate::flight_view::FlightViewConfig::default().spawn_direction;
    let (_, east) = pbd_core::geo::north_east(start);
    [(-apart_m, 101), (apart_m, 202)]
        .into_iter()
        .map(|(m, id)| Site {
            id,
            kind: SiteKind::Village,
            direction: (start + east * (m / config.radius_m)).normalize(),
            name: format!("test {id}"),
            capital: false,
            home: false,
            pinned: false,
            river: false,
        })
        .collect()
}

/// `world/roads`, "An old save": a world whose save holds no roads gains
/// them when it is opened (the loading thread's `prepare`, no restart), as
/// the world's records, once: opened again, nothing is planned and the same
/// roads are read.
#[test]
fn a_save_from_before_roads_gains_them_once_in_place() {
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let towns = villages(150.0);
    let mut save = WorldSave::memory_only();
    assert!(!roads::planned(&save.records));
    let (net, _) = prepare(&mut save, &towns, &config, &cfg);
    assert!(roads::planned(&save.records), "the plan is stored");
    let stored = roads::stored(&save.records);
    assert_eq!(stored.len(), 1, "the two villages are joined");
    assert!(!net.cells.is_empty() && !net.chunks.is_empty());
    assert!(net.cells.iter().all(|l| l.surface == Surface::Dirt));
    let records = save.records.len();
    let (again, seq) = prepare(&mut save, &towns, &config, &cfg);
    assert_eq!(seq, 0, "nothing is written the second time");
    assert_eq!(save.records.len(), records);
    assert_eq!(again.cells, net.cells);
}

/// Roads version 4 (`townsfolk` design 43a): two walled towns on the
/// shipped seed's start are joined by a gravel road the world lays wide,
/// every lane of it laid with the centre line; and a world whose roads
/// version 3 planned is read as it is, one wide, never planned again.
#[test]
fn a_new_world_lays_its_gravel_road_wide_and_a_version_3_world_keeps_its_own() {
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let towns: Vec<Site> = villages(150.0)
        .into_iter()
        .map(|s| Site {
            kind: SiteKind::Walled,
            ..s
        })
        .collect();
    let mut save = WorldSave::memory_only();
    let (net, _) = prepare(&mut save, &towns, &config, &cfg);
    assert_eq!(roads::net_version(&save.records), Some(4));
    let stored = roads::stored(&save.records);
    assert_eq!(stored.len(), 1);
    let road = &stored[0];
    assert_eq!(road.surface, Surface::Gravel);
    // On the start's rolling ground a lane off the road's plane, or beside
    // a bend on a rise, is left out: wide where the ground allows it.
    assert!(
        road.lanes.len() * 3 > road.cells.len(),
        "{} lane cells beside {}",
        road.lanes.len(),
        road.cells.len()
    );
    let laid: BTreeSet<u32> = net.cells.iter().map(|l| l.cell.key).collect();
    assert!(road.every_cell().all(|c| laid.contains(&c.key)));
    assert_eq!(net.cells.len(), road.every_cell().count());
    // A world planned by version 3: its road and its net record as that
    // version wrote them.
    let ground = {
        let empty = (Edits::new(), YieldSet::new(), BTreeMap::new());
        let g = ground_of(&config, &empty.0, &empty.1, &empty.2);
        roads::plan(
            &settlements(&towns),
            &g,
            &cfg.for_version(3),
            config.radius_m,
            1,
        )
    };
    assert!(ground[0].lanes.is_empty());
    let mut old = WorldSave::memory_only();
    let mut records: Vec<_> = ground.iter().map(roads::to_record).collect();
    records.push(pbd_core::records::Record::of(
        roads::ROAD_NET_RECORD,
        0,
        roads::RECORD_SCHEMA,
        &roads::NetBody {
            version: 3,
            roads: ground.len() as u32,
        },
    ));
    old.propose(
        &author(),
        Proposal {
            edits: Vec::new(),
            records,
        },
    )
    .unwrap();
    let before = old.records.len();
    let (net3, seq) = prepare(&mut old, &towns, &config, &cfg);
    assert_eq!(seq, 0, "nothing is planned again");
    assert_eq!(old.records.len(), before);
    assert_eq!(roads::net_version(&old.records), Some(3));
    assert_eq!(roads::stored(&old.records), ground);
    assert_eq!(
        net3.cells.len(),
        ground[0].cells.len(),
        "one wide, cell for cell"
    );
}

/// `world/roads`, "A player's house on the route": a cell the player
/// changed before the plan is never on a road; one changed after it is
/// left out when the road is cut, and the player's cell is unchanged.
#[test]
fn a_road_is_never_laid_on_a_players_edit() {
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let towns = villages(150.0);
    // The straight way's middle, edited first: the road goes round it.
    let mid = roads::nearest_key(towns[0].direction.lerp(towns[1].direction, 0.5)).unwrap();
    let mut save = WorldSave::memory_only();
    let edit = Edit {
        cell: mid,
        layer: 200,
        material: Material::Stone,
    };
    save.edits.set(edit);
    save.yields.touch_cell(mid);
    let (net, _) = prepare(&mut save, &towns, &config, &cfg);
    assert!(!net.cells.is_empty());
    assert!(
        net.cells.iter().all(|l| l.cell.key != mid),
        "round the edit"
    );
    // An edit made after the plan, on the road: that cell is not cut.
    let later = net.cells[net.cells.len() / 2].cell.key;
    let chunk = net
        .chunks
        .iter()
        .position(|c| c.cells.iter().any(|&i| net.cells[i].cell.key == later))
        .unwrap();
    let cut = |save: &WorldSave| {
        let yields = save.yields.clone();
        cut_chunk(
            &net,
            chunk,
            &|k| yields.has_cell(k),
            &cfg,
            config.radius_m,
            &|_| 2.0,
        )
        .1
    };
    let before = cut(&save);
    // The world's own edit there (a cutting, `townsfolk` design 32) does not
    // take the road away.
    save.edits.set(Edit {
        cell: later,
        layer: 199,
        material: Material::Air,
    });
    assert_eq!(cut(&save).len(), before.len(), "the world's edit");
    save.edits.set(Edit {
        cell: later,
        layer: 200,
        material: Material::Dirt,
    });
    save.yields.touch_cell(later);
    let after = cut(&save);
    assert_eq!(after.len() + 1, before.len());
    assert_eq!(
        save.edits.for_cell(later),
        &[(199, Material::Air), (200, Material::Dirt)]
    );
}

/// `world/roads`, "Dirt and gravel": the road under the feet is walked at
/// its surface's speed, gravel 15% over the ground, and off it at the
/// ground's.
#[test]
fn gravel_is_walked_faster_than_dirt() {
    let cfg = load_config();
    let key = roads::nearest_key(Vec3::new(0.3, 0.6, 0.7)).unwrap();
    let radius = crate::planet::terrain_config().radius_m;
    let cell = RoadCell {
        key,
        base_m: 10,
        kind: roads::Kind::Ground,
        raised: false,
        cut_fill: 0,
        bed: Default::default(),
        up: None,
        from: None,
    };
    let at = |s: Surface| {
        let p = roads::piece::cut(&cell, s, &[], radius, &|_| 2.0).unwrap();
        RoadPieces {
            solids: vec![p.solids],
            speed: vec![cfg.speed(s)],
        }
    };
    let feet = roads::centre(key).unwrap() * (radius + 10.0 + roads::piece::LIFT_M);
    let gravel = at(Surface::Gravel).speed_at(feet, 0.35);
    let dirt = at(Surface::Dirt).speed_at(feet, 0.35);
    assert!(
        (gravel / dirt - 1.15).abs() < 1e-5,
        "{gravel} against {dirt}"
    );
    let off = roads::centre(roads::neighbours(key)[0]).unwrap() * (radius + 10.0);
    assert_eq!(at(Surface::Gravel).speed_at(off, 0.35), 1.0);
    let above = roads::centre(key).unwrap() * (radius + 12.0);
    assert_eq!(at(Surface::Gravel).speed_at(above, 0.35), 1.0, "in the air");
}

/// A measurement (`townsfolk` design 10a): the shipped seed's network,
/// planned over every templated settlement but the cave towns, with no
/// towns' ground installed: how many links route, how many cells, how
/// long. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn the_shipped_seed_network() {
    let config = *crate::planet::terrain_config();
    let base = load_config();
    let version: u32 = std::env::var("PBD_ROADS_VERSION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(roads::NET_VERSION);
    let expanded: Option<usize> = std::env::var("PBD_ROADS_EXPANDED")
        .ok()
        .and_then(|v| v.parse().ok());
    let mut cfg = base.for_version(version);
    if let Some(e) = expanded {
        cfg.max_expanded = e;
    }
    println!("roads version {version}, {} cells a link", cfg.max_expanded);
    let sites = pbd_core::sites::generate(
        &crate::sites::load_rules(),
        &config,
        crate::sites::list_spawn(),
        4,
    )
    .sites;
    let set = settlements(&sites);
    let links = roads::links(&set, &cfg, config.radius_m);
    let started = std::time::Instant::now();
    let empty = (Edits::new(), YieldSet::new(), BTreeMap::new());
    let ground = ground_of(&config, &empty.0, &empty.1, &empty.2);
    let planned = roads::plan(&set, &ground, &cfg, config.radius_m, threads());
    let cells: usize = planned.iter().map(|r| r.cells.len()).sum();
    let slopes: usize = planned
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| c.up.is_some())
        .count();
    let bridges = planned
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| matches!(c.kind, roads::Kind::Bridge { .. }))
        .count();
    let worked = planned
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| c.cut_fill != 0)
        .count();
    println!("{worked} cells cut or filled");
    println!(
        "{} settlements, {} links, {} routed, {cells} cells ({slopes} sloped, {bridges} bridged), in {:.1} s",
        set.len(),
        links.len(),
        planned.len(),
        started.elapsed().as_secs_f32()
    );
    for r in &planned {
        let (a, b) = (
            sites.iter().find(|s| s.id == r.from_site).unwrap(),
            sites.iter().find(|s| s.id == r.to_site).unwrap(),
        );
        let (lat, lon) =
            pbd_core::geo::lat_lon(roads::centre(r.cells[r.cells.len() / 2].key).unwrap())
                .degrees();
        println!(
            "  {} to {} ({:?}): {} cells, middle at --at {lat:.5} {lon:.5}",
            a.name,
            b.name,
            r.surface,
            r.cells.len()
        );
    }
    // Why the links left out were left out: what the straight way between
    // their towns crosses (the sea, deep water wider than a bridge, a climb
    // of more than a layer), sampled a cell at a time.
    let routed: BTreeSet<(u32, u32)> = planned.iter().map(|r| (r.from_site, r.to_site)).collect();
    for &(a, b) in links.iter().filter(|l| !routed.contains(l)) {
        let (da, db) = (
            set.iter().find(|s| s.site == a).unwrap().direction,
            set.iter().find(|s| s.site == b).unwrap().direction,
        );
        let m = pbd_core::sites::arc_m(da, db, config.radius_m);
        let n = (m / 2.8).ceil() as usize;
        let (mut sea, mut cliffs, mut water_run, mut widest) = (0, 0, 0, 0);
        let mut last: Option<i32> = None;
        for k in 0..=n {
            let d = da.lerp(db, k as f32 / n as f32).normalize();
            let Some(g) = roads::nearest_key(d).and_then(&ground) else {
                continue;
            };
            match g.water {
                roads::Water::Sea => sea += 1,
                roads::Water::River { .. } => {
                    water_run += 1;
                    widest = widest.max(water_run);
                }
                roads::Water::Dry => water_run = 0,
            }
            if let Some(h) = last
                && (g.top_m - h).abs() > 1
            {
                cliffs += 1;
            }
            last = Some(g.top_m);
        }
        println!(
            "  unrouted {a} to {b}: {m:.0} m, straight way crosses {sea} sea cells, river {widest} cells wide at most, {cliffs} steps of over a layer"
        );
    }
    assert!(!planned.is_empty());
}

/// `world/roads`, "Dirt and gravel" (`townsfolk` task 8.3): the place
/// button with gravel lays a gravel road cell of the player's on dry
/// ground, taking one gravel and writing it with the pack; a cell with a
/// road already, or no gravel, is refused; the shovel lays a dirt track;
/// the player's own cell is taken up with its gravel back.
#[test]
fn the_player_lays_gravel_and_dirt_and_takes_them_up() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    let config = *crate::planet::terrain_config();
    let start = crate::flight_view::FlightViewConfig::default().spawn_direction;
    let (_, east) = pbd_core::geo::north_east(start);
    // Dry ground near the start: the first of a few cells east that is.
    let dry = (0..40)
        .map(|k| (start + east * (k as f32 * 3.0 / config.radius_m)).normalize())
        .filter_map(roads::nearest_key)
        .find(|&k| {
            let c = roads::centre(k).unwrap();
            let top = pbd_core::column::surface_m(&config, c);
            top > config.sea_level_m + 1.0
                && !pbd_core::map::river_water(&config, c)
                && !ground::cleared(&config, c)
        })
        .expect("dry ground near the start");
    let on_top = |key: u32| {
        let c = roads::centre(key).unwrap();
        c * (config.radius_m + pbd_core::column::surface_m(&config, c).floor())
    };
    let mut save = WorldSave::memory_only();
    let net = RoadNet::default();
    let mut slots = Slots::new();
    let facing = east;
    let point = on_top(dry);
    assert_eq!(
        lay(
            &mut save,
            &mut slots,
            &net,
            &config,
            point,
            facing,
            Some(Surface::Gravel)
        ),
        Err(LayRefused::NoGravel)
    );
    slots.give(Item::Good(Good::GRAVEL), 3);
    assert_eq!(
        lay(
            &mut save,
            &mut slots,
            &net,
            &config,
            point,
            facing,
            Some(Surface::Gravel)
        ),
        Ok(Lay::Laid(dry, Surface::Gravel))
    );
    assert_eq!(slots.count(Item::Good(Good::GRAVEL)), 2);
    let built = roads::built(&save.records);
    assert_eq!(built.get(&dry).map(|b| b.surface), Some(Surface::Gravel));
    assert_eq!(
        lay(
            &mut save,
            &mut slots,
            &net,
            &config,
            point,
            facing,
            Some(Surface::Dirt)
        ),
        Err(LayRefused::Road),
        "a road is there already"
    );
    // Off a cell's top (half way up a side): refused.
    let side = point * (1.0 + 0.5 / point.length());
    assert_eq!(
        lay(
            &mut save,
            &mut slots,
            &net,
            &config,
            side,
            facing,
            Some(Surface::Dirt)
        ),
        Err(LayRefused::NotATop)
    );
    // The net the player's cells are drawn from has it, at its speed.
    let fresh = RoadNet::of(&roads::stored(&save.records), &built, save.serial());
    assert!(
        fresh
            .cells
            .iter()
            .any(|l| l.cell.key == dry && l.surface == Surface::Gravel)
    );
    // Taken up: the gravel back, the cell gone.
    assert_eq!(
        lay(&mut save, &mut slots, &fresh, &config, point, facing, None),
        Ok(Lay::TakenUp(dry))
    );
    assert_eq!(slots.count(Item::Good(Good::GRAVEL)), 3);
    assert!(!roads::built(&save.records).contains_key(&dry));
    // A dirt track takes nothing.
    assert_eq!(
        lay(
            &mut save,
            &mut slots,
            &net,
            &config,
            point,
            facing,
            Some(Surface::Dirt)
        ),
        Ok(Lay::Laid(dry, Surface::Dirt))
    );
    assert_eq!(slots.count(Item::Good(Good::GRAVEL)), 3);
}

/// Two stakes a few cells apart on dry ground near the shipped start, and
/// where the top of each cell is.
fn staked_ground() -> (Vec3, Vec3) {
    let config = *crate::planet::terrain_config();
    let start = crate::flight_view::FlightViewConfig::default().spawn_direction;
    let (north, east) = pbd_core::geo::north_east(start);
    let dry = |d: Vec3| {
        let k = roads::nearest_key(d)?;
        let c = roads::centre(k)?;
        let top = pbd_core::column::surface_m(&config, c);
        (top > config.sea_level_m + 1.0
            && !pbd_core::map::river_water(&config, c)
            && !ground::cleared(&config, c))
        .then(|| c * (config.radius_m + top.floor()))
    };
    // The first pair 12 m apart, east to west, that both stand dry and
    // that the stake's search joins.
    for k in 0..200 {
        let a = (start
            + east * ((k % 20) as f32 * 9.0 / config.radius_m)
            + north * ((k / 20) as f32 * 9.0 / config.radius_m))
            .normalize();
        let b = (a + east * (12.0 / config.radius_m)).normalize();
        if let (Some(pa), Some(pb)) = (dry(a), dry(b)) {
            let (edits, yields) = (
                pbd_core::edits::Edits::default(),
                pbd_core::records::YieldSet::new(),
            );
            let ground = paving::stake_ground(&config, &edits, &yields);
            let (ka, kb) = (
                roads::nearest_key(a).unwrap(),
                roads::nearest_key(b).unwrap(),
            );
            let road = pbd_core::roads::planner::PlanRoad::new(
                vec![ka, kb],
                pbd_core::roads::planner::How::default(),
            );
            let laid = pbd_core::roads::planner::lay(
                &road,
                &ground,
                &|_| false,
                &paving::load_planner(),
                1.0,
            );
            if pbd_core::roads::planner::first_clash(&laid).is_none() {
                return (pa, pb);
            }
        }
    }
    panic!("no dry pair near the start")
}

/// `world/roads`, "A road is marked before it is paved", "Marks are saved"
/// and "Dirt and gravel" (`townsfolk` design 39 to 42l): a road by its two
/// ends marks the cells between, each marked cell is paved with one gravel,
/// the last with none is refused with nothing taken, and the plan, the
/// paved cells and the pack read back from the save as they were written.
#[test]
fn a_plan_is_marked_and_paved_and_survives_a_reload() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    use pbd_core::roads::planner as core;
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let pcfg = paving::load_planner();
    let (a, b) = staked_ground();
    let (ka, kb) = (paving::key_of(a).unwrap(), paving::key_of(b).unwrap());
    let root = std::env::temp_dir().join(format!("pbd-paving-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let slot = crate::saves::create(&root, "Paving", 41).unwrap();
    let stake = Item::Good(Good::SURVEY_STAKES);
    let gravel = Item::Good(Good::GRAVEL);
    let how = core::How::default();
    let (plans, built, carried, marked) = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let mut slots = Slots::new();
        // No stake in the pack: refused, nothing written.
        assert_eq!(
            paving::end_road(
                &mut save,
                &mut slots,
                ka,
                kb,
                how,
                &|_| false,
                &config,
                &cfg,
                &pcfg
            ),
            Err(paving::EditRefused::NoStakes)
        );
        assert!(core::plans(&save.records).is_empty());
        slots.give(stake, 4);
        let (staked, cells) = paving::end_road(
            &mut save,
            &mut slots,
            ka,
            kb,
            how,
            &|_| false,
            &config,
            &cfg,
            &pcfg,
        )
        .unwrap();
        assert_eq!(staked, 2, "a stake at each end");
        assert!(cells >= 4, "{cells} cells marked");
        assert_eq!(slots.count(stake), 2, "one stake a waypoint");
        let plans = core::plans(&save.records);
        let cells = plans.values().next().unwrap().roads[0].centre();
        // One gravel a cell: three gravel pave three cells, then none.
        slots.give(gravel, 3);
        for (n, &key) in cells.iter().take(3).enumerate() {
            assert!(paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg).is_ok());
            assert_eq!(slots.count(gravel), 2 - n as u32);
        }
        assert_eq!(
            paving::pave(&mut save, &mut slots, cells[3], &config, &cfg, &pcfg),
            Err(paving::PaveRefused::NoGravel)
        );
        assert_eq!(slots.count(gravel), 0);
        // A cell paved twice, or one not marked, takes nothing.
        slots.give(gravel, 1);
        assert_eq!(
            paving::pave(&mut save, &mut slots, cells[0], &config, &cfg, &pcfg),
            Err(paving::PaveRefused::NotMarked)
        );
        assert_eq!(slots.count(gravel), 1);
        // That gravel was never written; the pack saved is the last paving's.
        assert!(slots.take_item(gravel, 1));
        let built = roads::built(&save.records);
        assert_eq!(built.len(), 3);
        let id = *plans.keys().next().unwrap();
        assert!(
            built
                .values()
                .all(|c| c.surface == Surface::Gravel && c.plan == Some(id))
        );
        let (marked, _) = paving::marks(&save);
        assert_eq!(marked.len(), cells.len() - 3);
        save.drain();
        (plans, built, slots, marked)
    };
    let reopened = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
    assert_eq!(
        core::plans(&reopened.records),
        plans,
        "the plan, cell for cell"
    );
    assert_eq!(roads::built(&reopened.records), built, "the paved cells");
    assert_eq!(reopened.carried.as_ref(), Some(&carried), "the pack");
    // Tufts go at once from a paved or marked cell (design 44): the
    // terrain's word for the cell carries the road bit once installed.
    let road_bit = 1u32 << crate::planet::terrain::ROAD_BIT;
    // Held, so another test's install cannot replace these marks mid-check.
    let (_, _held) = paving::install_held(&reopened, config.radius_m);
    for &key in built.keys().chain(marked.keys()) {
        let c = roads::centre(key).unwrap();
        let h = pbd_core::column::surface_m(&config, c);
        assert!(pbd_core::roads::paving::bare(c), "cell {key} is bare");
        assert_ne!(
            crate::planet::surface_code(c, h) & road_bit,
            0,
            "cell {key}"
        );
        assert_eq!(
            pbd_core::roads::paving::mark_of(key).is_some(),
            marked.contains_key(&key)
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// A dirt road paves for no material (design 42c), and the eraser gives a
/// whole road's stakes back, its paved cells marked for removal (design
/// 42k): taking one up gives its gravel back.
#[test]
fn a_dirt_road_is_free_and_the_eraser_gives_its_stakes_back() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    use pbd_core::roads::planner as core;
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let pcfg = paving::load_planner();
    let (a, b) = staked_ground();
    let (ka, kb) = (paving::key_of(a).unwrap(), paving::key_of(b).unwrap());
    let root = std::env::temp_dir().join(format!("pbd-eraser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let slot = crate::saves::create(&root, "Eraser", 41).unwrap();
    let mut save = WorldSave::open(root.clone(), slot);
    let mut slots = Slots::new();
    let stake = Item::Good(Good::SURVEY_STAKES);
    slots.give(stake, 4);
    let dirt = core::How {
        surface: Surface::Dirt,
        ..core::How::default()
    };
    paving::end_road(
        &mut save,
        &mut slots,
        ka,
        kb,
        dirt,
        &|_| false,
        &config,
        &cfg,
        &pcfg,
    )
    .unwrap();
    let cells = core::plans(&save.records).values().next().unwrap().roads[0].centre();
    for &key in &cells {
        let cell = paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg).unwrap();
        assert_eq!(cell.surface, Surface::Dirt, "dirt, for nothing");
    }
    assert_eq!(slots.count(Item::Good(Good::GRAVEL)), 0);
    assert_eq!(slots.count(stake), 2);
    assert!(paving::erase_road(&mut save, &mut slots, cells[1], &config, &cfg, &pcfg).unwrap());
    assert_eq!(slots.count(stake), 4, "both stakes back");
    assert!(core::plans(&save.records).is_empty());
    let (_, removal) = paving::marks(&save);
    assert_eq!(removal.len(), cells.len(), "every paved cell for removal");
    let _ = std::fs::remove_dir_all(&root);
}

/// `world/roads`, lamp posts built on paving (design 42m): paving the cell
/// beside a post of a road with lamps places the town's street lantern on
/// the verge as the player's block, a log and a street lantern taken from
/// the pack; short of either, the cell is paved and the lamp waits for the
/// use key on it.
#[test]
fn paving_beside_a_post_builds_a_street_lantern_from_the_pack() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    use pbd_core::roads::planner as core;
    use pbd_core::terrain::Material;
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let pcfg = paving::load_planner();
    let (a, b) = staked_ground();
    let (ka, kb) = (paving::key_of(a).unwrap(), paving::key_of(b).unwrap());
    let root = std::env::temp_dir().join(format!("pbd-lamps-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let slot = crate::saves::create(&root, "Lamps", 41).unwrap();
    let mut save = WorldSave::open(root.clone(), slot);
    let mut slots = Slots::new();
    slots.give(Item::Good(Good::SURVEY_STAKES), 4);
    let lit = core::How {
        surface: Surface::Dirt,
        lamps: true,
        ..core::How::default()
    };
    paving::end_road(
        &mut save,
        &mut slots,
        ka,
        kb,
        lit,
        &|_| false,
        &config,
        &cfg,
        &pcfg,
    )
    .unwrap();
    let plans = core::plans(&save.records);
    let road = plans.values().next().unwrap().roads[0].clone();
    let posts = {
        let ground = paving::stake_ground(&config, &save.edits, &save.yields);
        let laid = core::lay(&road, &ground, &|_| false, &pcfg, cfg.ford_depth_m);
        core::lamp_posts(&road, &laid, &ground, &pcfg)
    };
    assert!(!posts.is_empty(), "a road with lamps has posts");
    let (beside, post) = posts[0];
    // Paved with nothing for the lamp: paved, and the lamp waits.
    assert!(paving::pave(&mut save, &mut slots, beside, &config, &cfg, &pcfg).is_ok());
    assert_eq!(
        paving::build_lamps(&mut save, &mut slots, beside, &config, &cfg),
        Err(paving::PaveRefused::NoLamp)
    );
    assert!(!paving::lamp_standing(&save, post));
    assert_eq!(paving::lamps_owed(&save, beside, &config, &cfg).len(), 1);
    // With a log and a street lantern, the use key builds it.
    let lantern = Item::Block(Material::LanternPost);
    slots.give(Item::Good(Good::LOG), 1);
    slots.give(lantern, 1);
    let made = paving::build_lamps(&mut save, &mut slots, beside, &config, &cfg).unwrap();
    assert_eq!(made.len(), 1);
    assert_eq!(
        (made[0].cell, made[0].material),
        (post, Material::LanternPost)
    );
    assert!(paving::lamp_standing(&save, post));
    assert_eq!(slots.count(Item::Good(Good::LOG)), 0);
    assert_eq!(slots.count(lantern), 0);
    assert!(paving::lamps_owed(&save, beside, &config, &cfg).is_empty());
    // It is the town's lamp: lit at dusk by the cell light field.
    assert!(Material::LanternPost.dusk_lit());
    // The bill no longer counts it.
    let built = roads::built(&save.records);
    let ground = paving::stake_ground(&config, &save.edits, &save.yields);
    let after = core::bill(
        &core::plans(&save.records),
        None,
        &built,
        &|k| paving::lamp_standing(&save, k),
        &ground,
        &pcfg,
        cfg.ford_depth_m,
    );
    assert_eq!(after.lamps as usize, posts.len() - 1);
    let _ = std::fs::remove_dir_all(&root);
}

/// The owner's lifted view (design 42n, survey P2): with survey stakes in
/// hand the eye eases up to `aerial_m` over `aerial_ease_s`, never in one
/// frame, and comes back down when they leave the hand; the look is tipped
/// down so the ground ahead stays in view.
#[test]
fn the_planner_view_eases_up_with_stakes_in_hand_and_back_down() {
    use crate::roads::planner::{PlannerLift, ease_lift};
    use pbd_core::goods::Good;
    use pbd_core::inventory::Item;
    let pcfg = paving::load_planner();
    assert_eq!(pcfg.aerial_m, 15.0, "the owner's 10 to 20 m, 15 by default");
    assert_eq!(pcfg.ghost_opacity, 0.55, "the owner's stronger ghost");
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .insert_resource(paving::PlannerSettings(std::sync::Arc::new(pcfg.clone())))
        .init_resource::<PlannerLift>()
        .insert_resource(crate::walking::WalkingReadout {
            active: true,
            ..Default::default()
        })
        .insert_resource(crate::hotbar::Hotbar::default())
        .add_systems(Update, ease_lift);
    let step = std::time::Duration::from_secs_f32(1.0 / 60.0);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(step));
    app.update();
    app.update();
    assert!(!app.world().resource::<PlannerLift>().lifted());
    // Stakes in hand: up over the ease, a little each frame.
    app.world_mut()
        .resource_mut::<crate::hotbar::Hotbar>()
        .0
        .give(Item::Good(Good::SURVEY_STAKES), 4);
    assert_eq!(
        app.world()
            .resource::<crate::hotbar::Hotbar>()
            .0
            .held()
            .map(|s| s.item),
        Some(Item::Good(Good::SURVEY_STAKES)),
        "the stakes are in hand"
    );
    let mut last = 0.0f32;
    let mut frames = 0;
    while app.world().resource::<PlannerLift>().amount < 1.0 && frames < 200 {
        app.update();
        let now = app.world().resource::<PlannerLift>().eased();
        assert!(now - last < 0.12, "no jump: {last} to {now}");
        last = now;
        frames += 1;
    }
    let secs = frames as f32 / 60.0;
    assert!(
        (secs - pcfg.aerial_ease_s).abs() < 0.1,
        "eased in over {secs} s"
    );
    let lift = *app.world().resource::<PlannerLift>();
    let (offset, tilt) = lift.offset(Vec3::Y, Vec3::NEG_Z);
    assert!((offset.y - pcfg.aerial_m).abs() < 1e-3, "{offset}");
    assert!(offset.z > 0.0, "back behind the player");
    assert!(tilt > 0.3 && tilt < 1.2, "tipped down {tilt}");
    // Stakes put away: back down to first person, eased too.
    app.world_mut().resource_mut::<crate::hotbar::Hotbar>().0 = pbd_core::inventory::Slots::new();
    let mut frames = 0;
    while app.world().resource::<PlannerLift>().lifted() && frames < 200 {
        app.update();
        frames += 1;
    }
    assert!(frames > 10, "eased out over {frames} frames");
    let (offset, tilt) = app
        .world()
        .resource::<PlannerLift>()
        .offset(Vec3::Y, Vec3::NEG_Z);
    assert_eq!((offset, tilt), (Vec3::ZERO, 0.0), "the walker's own eye");
}

/// `townsfolk` design 42d (task 14.5): the planner's bill falls by one
/// gravel for each cell paved, and the HUD line is drawn red while the pack
/// is short of a good it names and not once it holds enough; with the
/// shovel in hand the compass's tick is on the nearest cell not yet paved.
#[test]
fn the_bill_falls_a_gravel_a_cell_paved_and_a_short_good_is_red() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    use pbd_core::roads::planner as core;
    let config = *crate::planet::terrain_config();
    let cfg = load_config();
    let pcfg = paving::load_planner();
    assert_eq!(pcfg.cost_scale, 1.0, "one gravel a cell");
    let (a, b) = staked_ground();
    let (ka, kb) = (paving::key_of(a).unwrap(), paving::key_of(b).unwrap());
    let mut save = WorldSave::memory_only();
    let mut slots = Slots::new();
    let (stake, gravel) = (Item::Good(Good::SURVEY_STAKES), Item::Good(Good::GRAVEL));
    slots.give(stake, 4);
    paving::end_road(
        &mut save,
        &mut slots,
        ka,
        kb,
        core::How::default(),
        &|_| false,
        &config,
        &cfg,
        &pcfg,
    )
    .unwrap();
    let bill = |save: &WorldSave| {
        let ground = paving::stake_ground(&config, &save.edits, &save.yields);
        core::bill(
            &core::plans(&save.records),
            None,
            &roads::built(&save.records),
            &|_| false,
            &ground,
            &pcfg,
            cfg.ford_depth_m,
        )
    };
    let cells = core::plans(&save.records).values().next().unwrap().roads[0].centre();
    let start = bill(&save);
    assert_eq!(start.gravel as usize, cells.len());
    assert_eq!(start.left as usize, cells.len());
    // The HUD against a pack of `gravel` gravel.
    let line = |b: core::Bill, gravel: u32| {
        let mut aim = planner::PlannerAim::default();
        aim.showing = true;
        aim.bill = b;
        aim.have = [4, gravel, 0, 0, 0];
        planner::hud_line(&aim, &planner::Planner::default()).1
    };
    assert_eq!(
        line(start, 0),
        crate::ui_kit::WAYPOINT,
        "short of gravel: red"
    );
    assert_eq!(line(start, start.gravel), crate::ui_kit::LABEL, "enough");
    // The tick: the nearest cell not paved, from the road's first end.
    let tick = |save: &WorldSave| {
        let marked: Vec<u32> = paving::marks(save).0.into_keys().collect();
        paving::nearest_unpaved(&marked, a, pcfg.ghost_reach_m)
    };
    assert_eq!(tick(&save), Some(cells[0]));
    slots.give(gravel, 3);
    for (n, &key) in cells.iter().take(3).enumerate() {
        paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg).unwrap();
        let now = bill(&save);
        assert_eq!(now.gravel, start.gravel - 1 - n as u32, "one gravel a cell");
        assert_eq!(now.left, start.left - 1 - n as u32);
        assert_eq!(tick(&save), Some(cells[n + 1]), "on to the next cell");
        assert_eq!(line(now, slots.count(gravel)), crate::ui_kit::WAYPOINT);
    }
    assert_eq!(slots.count(gravel), 0);
    // None in reach from far off.
    let far = a.normalize() * (a.length()) + a.any_orthonormal_vector() * 500.0;
    assert_eq!(
        paving::nearest_unpaved(
            &paving::marks(&save).0.into_keys().collect::<Vec<_>>(),
            far,
            pcfg.ghost_reach_m
        ),
        None
    );
}

/// `townsfolk` design 42o: the planner's ghost of a bridged cell is the
/// bridge piece paving builds (its deck, the posts down to the bed and the
/// rails), not a flat top; a lamp post stands in it on its cell's ground up
/// to the lantern's height; a post already holding its lamp is not drawn.
#[test]
fn the_ghost_carries_a_bridge_s_deck_and_rail_and_leg_cards_and_its_lamp_posts() {
    use pbd_core::roads::planner::LaidCell;
    use std::collections::BTreeSet;
    let radius = crate::planet::terrain_config().radius_m;
    let key = roads::nearest_key(Vec3::new(0.3, 0.6, 0.7)).unwrap();
    let ahead = roads::neighbours(key)[0];
    let cell = |deck: Option<i32>| LaidCell {
        key,
        centre: key,
        lane: 0,
        before: None,
        after: Some(ahead),
        low_m: 10,
        base_m: 10,
        raised: false,
        ford: false,
        deck,
        bed: Default::default(),
        up: None,
        from: None,
        clash: None,
    };
    let centre = roads::centre(key).unwrap() * radius;
    let heights = |mesh: &Mesh| -> Vec<f32> {
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions");
        };
        p.iter()
            .map(|p| (Vec3::from_array(*p) + centre).length() - radius)
            .collect()
    };
    let road: BTreeSet<u32> = [key, ahead].into();
    let flat = heights(&planner::ghost_mesh(
        &[&cell(None)],
        &road,
        &[],
        radius,
        centre,
    ));
    let bridged = heights(&planner::ghost_mesh(
        &[&cell(Some(14))],
        &road,
        &[],
        radius,
        centre,
    ));
    assert!(
        bridged.len() > flat.len() * 2,
        "the deck and its sides: {} vertices against the top's {}",
        bridged.len(),
        flat.len()
    );
    // The rail and leg cards (design 42p), each in its own texture, the
    // same cards paving cuts: legs under the deck, rails over it.
    let [rails, legs] = planner::ghost_cards(&[&cell(Some(14))], &road, radius, centre);
    let (rails, legs) = (heights(&rails), heights(&legs));
    assert!(!rails.is_empty() && !legs.is_empty());
    let low = legs.iter().copied().fold(f32::MAX, f32::min);
    let high = rails.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        low < 14.0 - roads::piece::DECK_M,
        "legs under the deck: {low}"
    );
    assert!(
        high > 14.0 + 0.5 * roads::piece::RAIL_M,
        "rails over it: {high}"
    );
    let [none, _] = planner::ghost_cards(&[&cell(None)], &road, radius, centre);
    assert!(heights(&none).is_empty(), "no cards off a bridge");
    // A lamp post on its ground, the lantern up to about 1.9 m.
    let ground = |k: u32| {
        Some(roads::Ground {
            top_m: if k == ahead { 12 } else { 10 },
            water: roads::Water::Dry,
            town: false,
            player: false,
        })
    };
    let lamps = planner::ghost_lamps([ahead, key], &|k| k == key, &ground);
    assert_eq!(lamps, vec![(ahead, 12)], "the standing lamp is not drawn");
    let lamp = heights(&planner::ghost_mesh(&[], &road, &lamps, radius, centre));
    assert!(!lamp.is_empty());
    let (low, high) = lamp
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
    assert!((low - 12.0).abs() < 0.05, "on its ground: {low}");
    assert!((high - 13.9).abs() < 0.05, "the lantern's top: {high}");
}

/// Design 42p (the owner's survey P7): the bridge's rail and leg cards are
/// drawn in the timber's and the plank's own colours, with clear holes at
/// alpha 0 and no other alpha, and the manifest cuts both.
#[test]
fn the_bridge_cards_are_drawn_in_the_timber_s_palette() {
    let read = |name: &str| {
        let path = crate::paths::assets_dir().join(format!("textures/settlement/{name}.png"));
        image::open(&path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .to_rgba8()
    };
    let wood: std::collections::BTreeSet<[u8; 4]> = ["timber", "plank"]
        .iter()
        .flat_map(|n| read(n).pixels().map(|p| p.0).collect::<Vec<_>>())
        .collect();
    let cut = crate::towns::cut_textures();
    for name in planner::CARDS {
        assert!(cut.contains(name), "{name} is cut in the manifest");
        let card = read(name);
        assert_eq!(card.dimensions(), (64, 32), "{name}");
        let clear = card.pixels().filter(|p| p.0[3] == 0).count();
        let total = (card.width() * card.height()) as usize;
        assert!(
            clear > total / 4 && clear < total * 3 / 4,
            "{name}: {clear} clear"
        );
        for p in card.pixels().filter(|p| p.0[3] != 0) {
            assert_eq!(p.0[3], 255, "{name}: cut, not blended");
            assert!(wood.contains(&p.0), "{name}: {:?} is not the wood's", p.0);
            assert!(
                p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32 > 120,
                "no outline"
            );
        }
    }
}

/// Design 42p's measure (a measuring instrument, run by hand): at the bridge
/// capture's spot (`road-bridge.jpg`), the far river sheet's surface and the
/// ground under it at each fine level, and the generated column's water, so
/// the drawn river's height can be told from the deck's.
#[test]
#[ignore]
fn measure_the_river_at_the_bridge_capture() {
    let cfg = crate::planet::terrain_config();
    let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: 14.31677f32.to_radians(),
        lon: 59.09392f32.to_radians(),
    });
    let road = [1061057443u32, 1061057445, 1061057448];
    for &k in &road {
        let (lat, lon) = crate::place::at_flag(roads::centre(k).unwrap());
        println!("cell {k}: --at {lat:.5} {lon:.5}");
    }
    for (name, d) in [("spot", d)].into_iter().chain(
        road.iter()
            .map(|&k| ("road end", roads::centre(k).unwrap())),
    ) {
        let ground = crate::planet::surface_height(d);
        let column = pbd_core::column::generate_solid(cfg, d);
        let top = column.surface().unwrap();
        let water = (top + 1..pbd_core::column::LAYERS)
            .take_while(|&l| column.material(l).is_water())
            .count();
        println!(
            "{name}: ground {ground:.2}, column top {:.0}, water layers {water}",
            pbd_core::column::layer_altitude(top) + 1.0
        );
        for level in 0..6u32 {
            let cell_m = 2.833 * (1u32 << level) as f32;
            println!(
                "  cell {cell_m:.1} m: far surface {:?}, flood top {:?}",
                pbd_core::rivers::far_surface(cfg, d, cell_m),
                pbd_core::rivers::flood_top(cfg, d)
            );
        }
    }
}
