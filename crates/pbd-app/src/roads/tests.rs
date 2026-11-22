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
        .filter_map(|d| roads::nearest_key(d))
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
