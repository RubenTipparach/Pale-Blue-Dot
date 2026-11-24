use super::*;
use crate::records::Records;
use crate::roads::{Ground, nearest_key};

const RADIUS: f32 = 4_800.0;

fn here() -> (Vec3, Vec3, Vec3) {
    let d = Vec3::new(0.31, 0.62, 0.72).normalize();
    let east = Vec3::Y.cross(d).normalize();
    let north = d.cross(east);
    (d, east, north)
}

fn at(east: f32, north: f32) -> Vec3 {
    let (d, e, n) = here();
    (d + e * (east / RADIUS) + n * (north / RADIUS)).normalize()
}

fn local(c: Vec3) -> (f32, f32) {
    let (d, e, n) = here();
    let p = c / c.dot(d) - d;
    (p.dot(e) * RADIUS, p.dot(n) * RADIUS)
}

fn key(e: f32, n: f32) -> u32 {
    nearest_key(at(e, n)).unwrap()
}

fn dry(top_m: i32) -> Ground {
    Ground {
        top_m,
        water: Water::Dry,
        town: false,
        player: false,
    }
}

/// Ground rising a layer every `per_m` metres east (0 for flat).
fn slope(per_m: f32) -> impl Fn(u32) -> Option<Ground> + Sync {
    move |k| {
        let (e, _) = local(centre(k)?);
        Some(dry(if per_m > 0.0 {
            5 + (e / per_m).floor() as i32
        } else {
            5
        }))
    }
}

fn no_trees(_: u32) -> bool {
    false
}

fn cfg() -> RoadPlannerConfig {
    RoadPlannerConfig::default()
}

/// `world/roads`, the road planner's line (design 42a, 42l): the same cells
/// both ways round, each a neighbour of the one before, both ends in it.
#[test]
fn a_line_is_the_same_both_ways_round() {
    for (a, b) in [
        ((0.0, 0.0), (40.0, 0.0)),
        ((0.0, 0.0), (31.0, 23.0)),
        ((-12.0, 5.0), (8.0, -37.0)),
        ((3.0, 3.0), (3.5, 60.0)),
    ] {
        let (from, to) = (key(a.0, a.1), key(b.0, b.1));
        let there = hex_line(from, to);
        let mut back = hex_line(to, from);
        back.reverse();
        assert_eq!(there, back, "{a:?} to {b:?}");
        assert_eq!((there[0], *there.last().unwrap()), (from, to));
        for w in there.windows(2) {
            assert!(
                neighbours(w[0]).contains(&w[1]),
                "a step between neighbours"
            );
        }
        let unique: BTreeSet<u32> = there.iter().copied().collect();
        assert_eq!(unique.len(), there.len(), "no cell twice");
    }
}

/// Each reason reds only its own cell (design 42b): a tree on one cell,
/// one cell of the player's, a town's footprint, open water, and a two-layer
/// bank, whose higher cell alone is red.
#[test]
fn each_reason_reds_only_its_own_cell() {
    let road = PlanRoad::new(vec![key(0.0, 0.0), key(40.0, 0.0)], How::default());
    let line = road.centre();
    let pick = line[5];
    let case = |what: &str| -> Vec<LaidCell> {
        let ground = |k: u32| {
            let mut g = dry(5);
            if k == pick {
                match what {
                    "player" => g.player = true,
                    "town" => g.town = true,
                    "sea" => g.water = Water::Sea,
                    "deep" => {
                        g.water = Water::River {
                            depth_m: 3,
                            surface_m: 8,
                        }
                    }
                    _ => {}
                }
            }
            if what == "bank" && line.iter().position(|&c| c == k) >= Some(5) {
                g.top_m = 7;
            }
            Some(g)
        };
        let tree = |k: u32| what == "tree" && k == pick;
        lay(&road, &ground, &tree, &cfg(), 1.0)
    };
    for (what, why) in [
        ("tree", Clash::Tree),
        ("player", Clash::InTheWay),
        ("town", Clash::InTown),
        ("sea", Clash::OpenWater),
        ("deep", Clash::TooWideToBridge),
        ("bank", Clash::TooSteep),
    ] {
        let laid = case(what);
        let red: Vec<(u32, Clash)> = laid
            .iter()
            .filter_map(|c| Some((c.key, c.clash?)))
            .collect();
        assert_eq!(red, vec![(pick, why)], "{what}");
        assert_eq!(first_clash(&laid), Some((pick, why)));
    }
    // and nothing red on clear ground
    assert!(first_clash(&case("none")).is_none());
}

/// The steepest grade (design 42i): at a layer every two cells, two rises
/// a cell apart are refused on the higher cell; a layer every cell is not.
#[test]
fn the_grade_reds_two_rises_too_close() {
    let road = PlanRoad::new(vec![key(0.0, 0.0), key(40.0, 0.0)], How::default());
    let ground = slope(2.9);
    assert!(first_clash(&lay(&road, &ground, &no_trees, &cfg(), 1.0)).is_none());
    let steep = RoadPlannerConfig {
        max_grade: 2,
        ..cfg()
    };
    assert_eq!(
        first_clash(&lay(&road, &ground, &no_trees, &steep, 1.0)).map(|c| c.1),
        Some(Clash::TooSteep)
    );
}

/// A road up a slope never steps (design 42a): every two neighbours are at
/// most a layer apart, sloped toward the higher, and paving its cells in
/// any order gives the same road.
#[test]
fn a_road_up_a_slope_never_steps_and_paves_the_same_in_any_order() {
    let road = PlanRoad::new(vec![key(0.0, 0.0), key(36.0, 0.0)], How::default());
    let ground = slope(2.83);
    let laid = lay(&road, &ground, &no_trees, &cfg(), 1.0);
    assert!(first_clash(&laid).is_none());
    assert!(
        laid.last().unwrap().low_m - laid[0].low_m >= 10,
        "ten layers up"
    );
    for w in laid.windows(2) {
        assert!((w[0].low_m - w[1].low_m).abs() <= 1, "no step");
        if w[1].low_m == w[0].low_m + 1 {
            assert_eq!(w[0].up, Some(w[1].key), "sloped toward the higher");
        }
    }
    let one: Vec<_> = laid
        .iter()
        .map(|c| paved_cell(7, &road, c.key, &ground, &cfg(), 1.0).unwrap())
        .collect();
    let mut keys: Vec<u32> = laid.iter().map(|c| c.key).collect();
    keys.reverse();
    let mut two: Vec<_> = keys
        .iter()
        .map(|&k| paved_cell(7, &road, k, &ground, &cfg(), 1.0).unwrap())
        .collect();
    two.reverse();
    assert_eq!(one, two);
    assert!(one.iter().all(|c| c.plan == Some(7)));
}

/// A schema 1 plan (design 39b) reads as one road of width 1 in gravel whose
/// cells are the ones it stored; a schema 2 plan reads back cell for cell.
#[test]
fn a_schema_1_plan_reads_as_a_width_1_gravel_plan_and_a_plan_reads_back() {
    let mut records = Records::new();
    let cells: Vec<u32> = hex_line(key(0.0, 0.0), key(20.0, 6.0));
    let old = paving::Plan {
        stakes: vec![cells[0], *cells.last().unwrap()],
        cells: cells.clone(),
    };
    records.put(paving::plan_v1_record(3, Some(&old)));
    let read = plans(&records);
    let road = &read[&3].roads[0];
    assert_eq!((road.width, road.surface), (1, Surface::Gravel));
    assert_eq!(road.centre(), cells, "the stored cells, not a new line");
    let plan = RoadPlan {
        roads: vec![
            PlanRoad::new(
                vec![key(0.0, 0.0), key(30.0, 0.0), key(30.0, 30.0)],
                How {
                    width: 2,
                    surface: Surface::Dirt,
                    lamps: true,
                },
            ),
            road.clone(),
        ],
    };
    records.put(plan_record(3, Some(&plan)));
    let back = plans(&records);
    assert_eq!(back[&3], plan);
    let ground = slope(0.0);
    let laid_a = lay(&plan.roads[0], &ground, &no_trees, &cfg(), 1.0);
    let laid_b = lay(&back[&3].roads[0], &ground, &no_trees, &cfg(), 1.0);
    assert_eq!(laid_a, laid_b, "cell for cell");
    // a plan taken back whole reads as none
    records.put(plan_record(3, None));
    assert!(plans(&records).is_empty());
}

/// Two clicks make a road, staked every `max_segment` cells; a road from a
/// stake of another shares it; roads are added, never replaced (design 42k).
#[test]
fn roads_are_added_from_a_stake_never_replaced() {
    let ground = slope(0.0);
    let c = RoadPlannerConfig {
        max_segment: 8,
        ..cfg()
    };
    let none = BTreeMap::new();
    let (a, b) = (key(0.0, 0.0), key(50.0, 0.0));
    let first = add_road(&none, 1, a, b, How::default(), &ground, &no_trees, &c, 1.0).unwrap();
    let road = &first.plans[&1].roads[0];
    assert_eq!(road.stakes[0], a);
    assert_eq!(*road.stakes.last().unwrap(), b);
    let line = hex_line(a, b);
    assert_eq!(
        road.stakes.len(),
        2 + (line.len() - 2) / 8,
        "one each 8 cells"
    );
    assert_eq!(first.stakes, -(road.stakes.len() as i32), "a stake each");
    // a second road from its middle stake
    let mid = road.stakes[1];
    let second = add_road(
        &first.plans,
        2,
        mid,
        key(20.0, 30.0),
        How::default(),
        &ground,
        &no_trees,
        &c,
        1.0,
    )
    .unwrap();
    assert_eq!(second.plans[&1].roads.len(), 2, "added to the same plan");
    assert_eq!(second.plans[&1].roads[0], *road, "the first untouched");
    assert_eq!(
        second.plans[&1].roads[1].stakes[0], mid,
        "sharing the stake"
    );
    assert_eq!(
        second.stakes,
        -(second.plans[&1].roads[1].stakes.len() as i32 - 1),
        "the shared stake is not taken again"
    );
    // a clashing end takes nothing
    let tree_at_end = |k: u32| k == key(20.0, -30.0);
    assert_eq!(
        add_road(
            &first.plans,
            2,
            mid,
            key(20.0, -30.0),
            How::default(),
            &ground,
            &tree_at_end,
            &c,
            1.0
        ),
        Err(Clash::Tree)
    );
}

/// The eraser (design 42k): the whole road goes, its stakes come back but
/// the one another road holds, and its paved cells are for removal unless
/// another road runs over them.
#[test]
fn the_eraser_takes_a_whole_road_and_its_stakes_back() {
    let ground = slope(0.0);
    let none = BTreeMap::new();
    let (a, b, c) = (key(0.0, 0.0), key(30.0, 0.0), key(30.0, 30.0));
    let one = add_road(
        &none,
        1,
        a,
        b,
        How::default(),
        &ground,
        &no_trees,
        &cfg(),
        1.0,
    )
    .unwrap();
    let two = add_road(
        &one.plans,
        2,
        b,
        c,
        How::default(),
        &ground,
        &no_trees,
        &cfg(),
        1.0,
    )
    .unwrap();
    // pave every cell of both
    let mut paved = BTreeMap::new();
    for (id, r, cell) in every_cell(&two.plans, &ground, &cfg(), 1.0).into_values() {
        paved.insert(cell.key, cell.built(id, two.plans[&id].roads[r].surface));
    }
    let on_second = road_at(&two.plans, hex_line(b, c)[3], &ground, &cfg(), 1.0).unwrap();
    let erased = erase(&two.plans, on_second.0, on_second.1);
    assert_eq!(erased.stakes, 1, "c back; b is still the first road's");
    let cells = every_cell(&erased.plans, &ground, &cfg(), 1.0);
    let gone = for_removal(&cells, &paved);
    let second: BTreeSet<u32> = hex_line(b, c).into_iter().skip(1).collect();
    assert_eq!(gone.iter().copied().collect::<BTreeSet<_>>(), second);
    assert!(!gone.contains(&b), "the first road still runs over b");
    let last = erase(&erased.plans, 1, 0);
    assert!(last.plans.is_empty());
    assert_eq!(last.stakes, 2);
}

/// Editing (design 42f): moving a stake marks the paved cells it drops for
/// removal; a move that would clash keeps the stake; taking back a middle
/// stake joins its neighbours, or is refused and keeps it.
#[test]
fn moving_and_taking_back_stakes() {
    let ground = slope(0.0);
    let none = BTreeMap::new();
    let (a, b) = (key(0.0, 0.0), key(40.0, 0.0));
    let c = RoadPlannerConfig {
        max_segment: 6,
        ..cfg()
    };
    let one = add_road(&none, 1, a, b, How::default(), &ground, &no_trees, &c, 1.0).unwrap();
    let mut paved = BTreeMap::new();
    for (id, _, cell) in every_cell(&one.plans, &ground, &c, 1.0).into_values() {
        paved.insert(cell.key, cell.built(id, Surface::Gravel));
    }
    let mid = one.plans[&1].roads[0].stakes[2];
    let (me, mn) = local(centre(mid).unwrap());
    let (aside, other) = (key(me, mn + 6.0), key(me, mn - 6.0));
    let moved = move_stake(&one.plans, mid, aside, &ground, &no_trees, &c, 1.0).unwrap();
    assert_eq!(moved.stakes, 0);
    let dropped = for_removal(&every_cell(&moved.plans, &ground, &c, 1.0), &paved);
    assert!(!dropped.is_empty() && dropped.iter().all(|k| paved.contains_key(k)));
    // a tree where it would go: refused, nothing moves
    let tree = |k: u32| k == other;
    assert_eq!(
        move_stake(&one.plans, mid, other, &ground, &tree, &c, 1.0),
        Err(Clash::Tree)
    );
    // taking the middle back joins its neighbours
    // (joined by one leg of twelve cells: past the six of `c`, so with the
    // shipped 24)
    assert_eq!(
        take_back(&one.plans, mid, &ground, &no_trees, &c, 1.0),
        Err(Clash::TooFar)
    );
    let back = take_back(&one.plans, mid, &ground, &no_trees, &cfg(), 1.0).unwrap();
    assert_eq!(back.stakes, 1);
    assert!(!back.plans[&1].roads[0].stakes.contains(&mid));
    // but not where the join would cross the player's work
    let stakes = &one.plans[&1].roads[0].stakes;
    let (s1, s3) = (stakes[1], stakes[3]);
    let joined = hex_line(s1, s3);
    let blocked = joined[joined.len() / 2];
    let walled = |k: u32| {
        let mut g = dry(5);
        g.player = k == blocked && !one.plans[&1].roads[0].centre().contains(&k);
        Some(g)
    };
    if !one.plans[&1].roads[0].centre().contains(&blocked) {
        assert_eq!(
            take_back(&one.plans, mid, &walled, &no_trees, &cfg(), 1.0),
            Err(Clash::InTheWay)
        );
    }
    // an end stake of a two-stake road takes the road with it
    let short = add_road(
        &none,
        1,
        a,
        key(10.0, 0.0),
        How::default(),
        &ground,
        &no_trees,
        &c,
        1.0,
    )
    .unwrap();
    let gone = take_back(&short.plans, a, &ground, &no_trees, &c, 1.0).unwrap();
    assert!(gone.plans.is_empty());
    assert_eq!(gone.stakes, 2);
}

/// Wide roads (design 10b, 42l): a second lane beside the centre, on its
/// plane; the bill's gravel rises by the lane's cells; a lane off the plane
/// is "Too steep across" on that cell alone.
#[test]
fn a_wide_road_lays_its_lanes_on_one_plane() {
    let (a, b) = (key(0.0, 0.0), key(28.0, 0.0));
    let flat = slope(0.0);
    let one = PlanRoad::new(vec![a, b], How::default());
    let two = PlanRoad::new(
        vec![a, b],
        How {
            width: 2,
            ..How::default()
        },
    );
    let three = PlanRoad::new(
        vec![a, b],
        How {
            width: 3,
            ..How::default()
        },
    );
    let n1 = lay(&one, &flat, &no_trees, &cfg(), 1.0).len();
    let laid2 = lay(&two, &flat, &no_trees, &cfg(), 1.0);
    let laid3 = lay(&three, &flat, &no_trees, &cfg(), 1.0);
    let lanes2 = laid2.iter().filter(|c| c.lane != 0).count();
    assert!(
        lanes2 + 2 >= n1 && lanes2 <= n1 + 2,
        "a lane as long as the road: {lanes2} beside {n1}"
    );
    assert!(laid2.iter().all(|c| c.lane >= 0), "to the left");
    assert!(laid3.iter().any(|c| c.lane < 0) && laid3.iter().any(|c| c.lane > 0));
    let none = BTreeMap::new();
    let paved = BTreeMap::new();
    let bill1 = bill(&none, Some(&one), &paved, &flat, &cfg(), 1.0);
    let bill2 = bill(&none, Some(&two), &paved, &flat, &cfg(), 1.0);
    assert_eq!(bill1.gravel as usize, n1);
    assert_eq!(bill2.gravel as usize, n1 + lanes2);
    // on the level every lane is on its centre's plane and allowed
    assert!(laid3.iter().all(|c| c.clash.is_none()));
    for c in laid3.iter().filter(|c| c.lane != 0) {
        let centre_cell = laid3.iter().find(|l| l.key == c.centre).unwrap();
        assert_eq!(c.low_m, centre_cell.low_m, "on its centre's plane");
    }
    // up a slope the centre climbs, and a lane beside a sloped cell is red
    // (no half-layer bed yet), never drawn with a seam
    let up = slope(2.83);
    let laid = lay(&three, &up, &no_trees, &cfg(), 1.0);
    assert!(
        laid.iter()
            .filter(|c| c.lane == 0)
            .all(|c| c.clash.is_none())
    );
    for c in laid.iter().filter(|c| c.lane != 0) {
        let centre_cell = laid.iter().find(|l| l.key == c.centre).unwrap();
        if centre_cell.up.is_some() {
            assert_eq!(c.clash, Some(Clash::TooSteepAcross));
        }
    }
    // a ridge beside the road: those lane cells alone are red
    let line = two.centre();
    let ridge = |k: u32| Some(dry(if line.contains(&k) { 5 } else { 8 }));
    let laid = lay(&two, &ridge, &no_trees, &cfg(), 1.0);
    assert!(
        laid.iter()
            .filter(|c| c.lane == 0)
            .all(|c| c.clash.is_none())
    );
    assert!(
        laid.iter()
            .filter(|c| c.lane != 0)
            .all(|c| c.clash == Some(Clash::TooSteepAcross))
    );
}

/// A wide road bends only on the level (design 10b).
#[test]
fn a_wide_road_bends_only_on_the_level() {
    let (a, b, c) = (key(0.0, 0.0), key(28.0, 0.0), key(28.0, 28.0));
    let wide = PlanRoad::new(
        vec![a, b, c],
        How {
            width: 2,
            ..How::default()
        },
    );
    assert!(first_clash(&lay(&wide, &slope(0.0), &no_trees, &cfg(), 1.0)).is_none());
    let up = slope(2.83);
    let laid = lay(&wide, &up, &no_trees, &cfg(), 1.0);
    let bend = laid.iter().find(|l| l.key == b).unwrap();
    assert_eq!(bend.clash, Some(Clash::WideBend));
}

/// Every tunable of `road_planner.ron` in its range, the shipped file valid,
/// and each out of range refused at load (design 42i).
#[test]
fn every_planner_tunable_out_of_range_is_refused() {
    let shipped: RoadPlannerConfig = ron::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/road_planner.ron"
    )))
    .expect("road_planner.ron reads");
    shipped.validate().expect("the shipped planner validates");
    assert_eq!(shipped, cfg(), "the defaults are the shipped values");
    let bad: [fn(&mut RoadPlannerConfig); 12] = [
        |c| c.ghost_opacity = 1.5,
        |c| c.ghost_tint = [0.0, 2.0, 0.0],
        |c| c.refuse_tint = [f32::NAN, 0.0, 0.0],
        |c| c.max_grade = 0,
        |c| c.max_segment = 1,
        |c| c.stake_reach_m = 0.0,
        |c| c.reach_m = 100.0,
        |c| c.ghost_reach_m = -1.0,
        |c| c.cost_scale = 0.0,
        |c| c.lamp_spacing = 0,
        |c| c.refund_share = -0.1,
        |c| c.max_grade = 9,
    ];
    for (n, f) in bad.iter().enumerate() {
        let mut c = cfg();
        f(&mut c);
        assert!(c.validate().is_err(), "case {n}");
    }
}

/// Lamps stand every `lamp_spacing` cells along a road with lamps, and are
/// in its bill (design 42e).
#[test]
fn lamps_stand_along_a_road_with_lamps() {
    let road = PlanRoad::new(
        vec![key(0.0, 0.0), key(40.0, 0.0)],
        How {
            lamps: true,
            ..How::default()
        },
    );
    let cells = lamp_cells(&road, &cfg());
    let n = road.centre().len();
    assert!(
        cells.len() >= n / 6 && cells.len() <= n / 6 + 1,
        "{} of {n}",
        cells.len()
    );
    let b = bill(
        &BTreeMap::new(),
        Some(&road),
        &BTreeMap::new(),
        &slope(0.0),
        &cfg(),
        1.0,
    );
    assert_eq!(b.lamps as usize, cells.len());
    assert_eq!(b.stakes, 2);
    assert_eq!(b.left, b.cells);
}
