use super::*;

const RADIUS: f32 = 4_800.0;

/// A place on the planet to build test ground round, away from any face
/// edge, and its east and north.
fn here() -> (Vec3, Vec3, Vec3) {
    let d = Vec3::new(0.31, 0.62, 0.72).normalize();
    let east = Vec3::Y.cross(d).normalize();
    let north = d.cross(east);
    (d, east, north)
}

/// A cell centre's east and north of `here`, metres.
fn local(c: Vec3) -> (f32, f32) {
    let (d, e, n) = here();
    let p = c / c.dot(d) - d;
    (p.dot(e) * RADIUS, p.dot(n) * RADIUS)
}

/// The direction `east` and `north` metres from `here`.
fn at(east: f32, north: f32) -> Vec3 {
    let (d, e, n) = here();
    (d + e * (east / RADIUS) + n * (north / RADIUS)).normalize()
}

fn dry(top_m: i32) -> Ground {
    Ground {
        top_m,
        water: Water::Dry,
        town: false,
        player: false,
    }
}

#[test]
fn every_cell_has_six_neighbours_or_five_and_they_are_mutual() {
    let mut keys = vec![nearest_key(at(0.0, 0.0)).unwrap()];
    // a corner of the icosahedron, and a point on a face's edge
    let (vertices, faces) = crate::topology::icosahedron();
    keys.push(nearest_key(vertices[0]).unwrap());
    let edge = (vertices[faces[3][0]] + vertices[faces[3][1]]).normalize();
    keys.push(nearest_key(edge).unwrap());
    for (n, &key) in keys.iter().enumerate() {
        let ns = neighbours(key);
        assert_eq!(ns.len(), if n == 1 { 5 } else { 6 }, "cell {n}");
        let c = centre(key).unwrap();
        for &m in &ns {
            assert!(neighbours(m).contains(&key), "a neighbour is mutual");
            let w = crate::sites::arc_m(c, centre(m).unwrap(), RADIUS);
            assert!((2.4..3.3).contains(&w), "cells {w} m apart");
        }
        assert_eq!(corners(key).len(), ns.len());
        for corner in corners(key) {
            let r = crate::sites::arc_m(c, corner, RADIUS);
            assert!((1.3..2.0).contains(&r), "a corner {r} m out");
        }
    }
}

#[test]
fn the_nearest_key_of_a_cell_centre_is_that_cell() {
    let start = nearest_key(at(0.0, 0.0)).unwrap();
    let mut keys = vec![start];
    for _ in 0..40 {
        let last = *keys.last().unwrap();
        keys.push(neighbours(last)[(last % 6) as usize % neighbours(last).len()]);
    }
    let (vertices, _) = crate::topology::icosahedron();
    for v in vertices {
        let k = nearest_key(v).unwrap();
        keys.push(k);
        keys.extend(neighbours(k));
    }
    for k in keys {
        assert_eq!(nearest_key(centre(k).unwrap()), Some(k));
    }
}

/// The cells of a straight line east of `here`, `n` of them.
fn line_east(n: usize) -> Vec<u32> {
    let mut out = vec![nearest_key(at(0.0, 0.0)).unwrap()];
    while out.len() < n {
        let last = *out.last().unwrap();
        let next = neighbours(last)
            .into_iter()
            .max_by(|a, b| {
                let (ea, na) = local(centre(*a).unwrap());
                let (eb, nb) = local(centre(*b).unwrap());
                (ea - na.abs() * 3.0).total_cmp(&(eb - nb.abs() * 3.0))
            })
            .unwrap();
        out.push(next);
    }
    out
}

/// `world/roads`, "A road slopes, never steps": a road up ten layers, a
/// layer a cell, walked along its middle in 5 cm steps, never rises more
/// than a slope's share of one: no step anywhere, and ten layers climbed.
#[test]
fn a_road_up_ten_layers_never_steps() {
    let cells = line_east(14);
    // flat for two cells, a layer a cell for ten, flat again
    let height = |k: usize| (k.saturating_sub(1)).min(10) as i32;
    let steps: Vec<Step> = cells
        .iter()
        .enumerate()
        .map(|(k, &key)| Step {
            key,
            ground: dry(height(k)),
            deck: None,
        })
        .collect();
    let road = profile(&steps).expect("a layer a cell is a road");
    assert_eq!(road.iter().filter(|c| c.up.is_some()).count(), 10);
    // Walk the centre line, cell to cell, through each shared side's middle.
    let mut points = Vec::new();
    for w in road.windows(2) {
        let (a, b) = (centre(w[0].key).unwrap(), centre(w[1].key).unwrap());
        for s in 0..=40 {
            let t = s as f32 / 40.0;
            let p = a.lerp(b, t).normalize();
            let cell = if t < 0.5 { &w[0] } else { &w[1] };
            points.push((p, cell.top_at(p)));
        }
    }
    let mut worst = 0.0f32;
    for w in points.windows(2) {
        let run = crate::sites::arc_m(w[0].0, w[1].0, RADIUS).max(1e-4);
        worst = worst.max((w[1].1 - w[0].1) / run);
    }
    let climbed = points.last().unwrap().1 - points[0].1;
    assert!((climbed - 10.0).abs() < 1e-3, "climbed {climbed}");
    // One layer over a cell's 2.45 to 3.1 m, with nothing steeper anywhere.
    assert!(
        worst < 0.45,
        "the steepest rise is {worst} m a metre: a step"
    );
}

#[test]
fn a_dip_one_cell_wide_is_raised_and_a_cliff_is_no_road() {
    let cells = line_east(5);
    let steps = |h: [i32; 5]| -> Vec<Step> {
        cells
            .iter()
            .zip(h)
            .map(|(&key, h)| Step {
                key,
                ground: dry(h),
                deck: None,
            })
            .collect()
    };
    let road = profile(&steps([3, 3, 2, 3, 3])).unwrap();
    assert!(road[2].raised && road[2].low_m() == 3 && road[2].up.is_none());
    let road = profile(&steps([3, 4, 3, 4, 5])).unwrap();
    assert!(road[2].raised, "a dip between a slope and a rise");
    assert!(
        profile(&steps([3, 3, 5, 5, 5])).is_none(),
        "two layers at once"
    );
}

/// Ground round `here`: flat at layer `top`, but `player` cells and a
/// `river` band of `width` metres east of `x0`, `depth` deep.
fn ground_fn(
    player: impl Fn(f32, f32) -> bool + Sync,
    river: Option<(f32, f32, i32)>,
) -> impl Fn(u32) -> Option<Ground> + Sync {
    move |key| {
        let (e, n) = local(centre(key)?);
        let mut g = dry(5);
        if let Some((x0, width, depth)) = river
            && e > x0
            && e < x0 + width
        {
            g.top_m = 5 - 1 - depth;
            g.water = Water::River {
                depth_m: depth,
                surface_m: 4,
            };
        }
        g.player = player(e, n);
        Some(g)
    }
}

/// `world/roads`, "A player's house on the route": a wall of the player's
/// cells across the straight way: the road goes round it and never crosses
/// one.
#[test]
fn a_road_goes_round_the_players_work() {
    let wall = |e: f32, n: f32| (e - 60.0).abs() < 4.0 && n.abs() < 30.0;
    let ground = ground_fn(wall, None);
    let from = nearest_key(at(0.0, 0.0)).unwrap();
    let to = nearest_key(at(120.0, 0.0)).unwrap();
    let steps = route(from, to, &ground, &RoadsConfig::default(), RADIUS).unwrap();
    assert!(steps.iter().all(|s| !s.ground.player));
    let (e, n) = local(centre(steps[steps.len() / 2].key).unwrap());
    assert!(n.abs() > 25.0, "round the wall, at {e:.0} {n:.0}");
}

#[test]
fn shallow_water_is_forded_and_deep_water_is_bridged() {
    let cfg = RoadsConfig::default();
    let from = nearest_key(at(0.0, 0.0)).unwrap();
    let to = nearest_key(at(60.0, 0.0)).unwrap();
    // a layer of water over its bed, a layer under the banks: forded
    let shallow = ground_fn(|_, _| false, Some((25.0, 8.0, 0)));
    let road = profile(&route(from, to, &shallow, &cfg, RADIUS).unwrap()).unwrap();
    assert!(road.iter().any(|c| c.kind == Kind::Ford));
    assert!(road.iter().all(|c| !matches!(c.kind, Kind::Bridge { .. })));
    // three layers deep: bridged at the banks' height
    let deep = ground_fn(|_, _| false, Some((25.0, 8.0, 3)));
    let road = profile(&route(from, to, &deep, &cfg, RADIUS).unwrap()).unwrap();
    let decks: Vec<i32> = road
        .iter()
        .filter_map(|c| match c.kind {
            Kind::Bridge { deck_m } => Some(deck_m),
            _ => None,
        })
        .collect();
    assert!(
        !decks.is_empty() && decks.iter().all(|&d| d == 5),
        "{decks:?}"
    );
    assert!(decks.len() <= cfg.max_bridge_cells as usize);
    // wider than the longest bridge, with no way round: no road
    let wide = ground_fn(|_, _| false, Some((10.0, 40.0, 3)));
    let small = RoadsConfig {
        max_expanded: 20_000,
        ..cfg
    };
    assert!(route(from, to, &wide, &small, RADIUS).is_none());
}

#[test]
fn the_sea_is_never_crossed() {
    let from = nearest_key(at(0.0, 0.0)).unwrap();
    let to = nearest_key(at(40.0, 0.0)).unwrap();
    let ground = |key: u32| {
        let (e, _) = local(centre(key)?);
        let mut g = dry(5);
        if e > 15.0 && e < 25.0 {
            g.water = Water::Sea;
        }
        Some(g)
    };
    let cfg = RoadsConfig {
        max_expanded: 20_000,
        ..RoadsConfig::default()
    };
    assert!(route(from, to, &ground, &cfg, RADIUS).is_none());
}

/// `world/roads`, "Roads join the towns": each settlement is joined to its
/// two nearest within 2 km, and the large towns each to its nearest large
/// town further off.
#[test]
fn every_settlement_reaches_its_two_nearest_within_2_km() {
    let cfg = RoadsConfig::default();
    let s = |site: u32, e: f32, n: f32, large: bool| Settlement {
        site,
        direction: at(e, n),
        large,
    };
    let towns = vec![
        s(1, 0.0, 0.0, true),
        s(2, 900.0, 0.0, false),
        s(3, 0.0, 1_200.0, false),
        s(4, 1_500.0, 1_500.0, false),
        s(5, 5_000.0, 0.0, true),
        s(6, 5_600.0, 300.0, false),
    ];
    let pairs = links(&towns, &cfg, RADIUS);
    for t in &towns {
        let mut near: Vec<(f32, u32)> = towns
            .iter()
            .filter(|o| o.site != t.site)
            .map(|o| {
                (
                    crate::sites::arc_m(t.direction, o.direction, RADIUS),
                    o.site,
                )
            })
            .filter(|(m, _)| *m <= cfg.link_m)
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, o) in near.into_iter().take(2) {
            assert!(pairs.contains(&(t.site.min(o), t.site.max(o))));
        }
    }
    assert!(pairs.contains(&(1, 5)), "the large towns are joined");
    assert!(!pairs.contains(&(3, 5)), "a village is not joined 5 km off");
    assert_eq!(surface_of(&towns[0], &towns[4]), Surface::Gravel);
    assert_eq!(surface_of(&towns[0], &towns[1]), Surface::Dirt);
}

#[test]
fn a_plan_is_the_same_twice_and_its_records_read_back() {
    let towns = vec![
        Settlement {
            site: 7,
            direction: at(0.0, 0.0),
            large: false,
        },
        Settlement {
            site: 9,
            direction: at(70.0, 20.0),
            large: false,
        },
    ];
    let cfg = RoadsConfig::default();
    let g = ground_fn(|_, _| false, Some((30.0, 6.0, 2)));
    let a = plan(&towns, &g, &cfg, RADIUS, 1);
    let b = plan(&towns, &g, &cfg, RADIUS, 3);
    assert_eq!(a, b, "the same on one thread as on three");
    assert_eq!(a.len(), 1);
    let mut records = Records::new();
    assert!(!planned(&records));
    for road in &a {
        records.put(to_record(road));
    }
    records.put(net_record(a.len()));
    assert!(planned(&records));
    assert_eq!(stored(&records), a);
    // and a line of the journal carries it whole
    let line = to_record(&a[0]).line(&crate::records::Author::Creation);
    let (_, back) = Record::parse_line(&line).unwrap();
    assert_eq!(from_record(&back).as_ref(), Some(&a[0]));
}

#[test]
fn a_built_cell_rises_toward_the_first_neighbour_a_layer_up() {
    assert_eq!(built_up(4, &[(10, 4), (11, 5), (12, 5)]), Some(11));
    assert_eq!(built_up(4, &[(10, 4), (11, 6)]), None);
    let mut records = Records::new();
    let cell = BuiltCell {
        surface: Surface::Gravel,
        base_m: 4,
        up: Some(11),
        from: None,
    };
    let key = nearest_key(at(0.0, 0.0)).unwrap();
    records.put(built_record(key, &cell));
    assert_eq!(built(&records).get(&key), Some(&cell));
    assert_eq!(built_cell(key, &cell).up, Some(11));
}

#[test]
fn the_shipped_config_validates() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/roads.ron"
    ))
    .unwrap();
    let cfg: RoadsConfig = ron::from_str(&text).unwrap();
    cfg.validate().unwrap();
    assert_eq!(cfg, RoadsConfig::default(), "the tests' copy is the file's");
    assert!((cfg.speed(Surface::Gravel) / cfg.speed(Surface::Dirt) - 1.15).abs() < 1e-6);
}

#[test]
fn a_road_cell_is_cleared_and_its_neighbour_is_not() {
    let key = nearest_key(at(0.0, 0.0)).unwrap();
    let cleared = Cleared::new([key], RADIUS);
    assert!(cleared.on_road(centre(key).unwrap()));
    assert!(cleared.on_road(at(0.5, 0.5)));
    for n in neighbours(key) {
        assert!(!cleared.on_road(centre(n).unwrap()));
    }
}
