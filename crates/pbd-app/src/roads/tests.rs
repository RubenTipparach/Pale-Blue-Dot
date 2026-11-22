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
    let before = cut_chunk(&net, chunk, &save.edits, &cfg, config.radius_m, &|_| 2.0).1;
    save.edits.set(Edit {
        cell: later,
        layer: 200,
        material: Material::Dirt,
    });
    let after = cut_chunk(&net, chunk, &save.edits, &cfg, config.radius_m, &|_| 2.0).1;
    assert_eq!(after.len() + 1, before.len());
    assert_eq!(save.edits.for_cell(later), &[(200, Material::Dirt)]);
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
    let cfg = load_config();
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
    assert!(!planned.is_empty());
}
