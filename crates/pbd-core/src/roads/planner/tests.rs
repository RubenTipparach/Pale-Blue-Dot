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
    let bill1 = bill(&none, Some(&one), &paved, &|_| false, &flat, &cfg(), 1.0);
    let bill2 = bill(&none, Some(&two), &paved, &|_| false, &flat, &cfg(), 1.0);
    assert_eq!(bill1.gravel as usize, n1);
    assert_eq!(bill2.gravel as usize, n1 + lanes2);
    // on the level every lane is on its centre's plane and allowed
    assert!(laid3.iter().all(|c| c.clash.is_none()));
    for c in laid3.iter().filter(|c| c.lane != 0) {
        let centre_cell = laid3.iter().find(|l| l.key == c.centre).unwrap();
        assert_eq!(c.low_m, centre_cell.low_m, "on its centre's plane");
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
    let bad: [fn(&mut RoadPlannerConfig); 16] = [
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
        |c| c.max_bridge_cells = 0,
        |c| c.bridge_planks = 0,
        |c| c.aerial_m = 2.0,
        |c| c.aerial_ease_s = f32::NAN,
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
        &|_| false,
        &slope(0.0),
        &cfg(),
        1.0,
    );
    assert_eq!(b.lamps as usize, cells.len());
    assert_eq!(b.stakes, 2);
    assert_eq!(b.left, b.cells);
}

// ---- Design 42m: wide roads on a slope, bridges, lamps, inserting ------------

/// A straight row of the lattice from `key(0, 0)`, `cells` long: the two
/// stakes and the row's pitch, m, and the way it runs.
fn row(cells: i64) -> (u32, u32, f32, Vec3) {
    let a = key(0.0, 0.0);
    let addr = crate::cell_key::unpack(a).unwrap();
    let fa = crate::cell_key::addresses(addr)[0];
    let b = translated(a, fa.face, fa.level, (cells, 0)).expect("the row stays on its face");
    let (ca, cb) = (centre(a).unwrap(), centre(b).unwrap());
    let pitch = ca.angle_between(cb) * RADIUS / cells as f32;
    let way = (cb - ca).normalize();
    (a, b, pitch, way)
}

/// Ground rising a layer a pitch along `way` from `from`, held level for
/// `flat` pitches first and after `layers`, shifted by `phase` of a pitch.
fn hill(
    from: Vec3,
    way: Vec3,
    pitch: f32,
    flat: f32,
    layers: i32,
    phase: f32,
) -> impl Fn(u32) -> Option<Ground> + Sync {
    move |k| {
        let s = (centre(k)? - from).dot(way) * RADIUS / pitch;
        let rise = ((s - flat + phase).floor() as i32).clamp(0, layers);
        Some(dry(5 + rise))
    }
}

/// `world/roads`, wide roads on one plane (design 10b, 42m): a two- and a
/// three-wide road straight up ten layers, the hill's rows in step with the
/// road's or half a layer off it, are laid with no red, the lanes on a
/// half-height bed where they need one and never cut; and a walker crossing
/// every lane diagonally up the climb and back finds no step.
#[test]
fn a_wide_road_up_ten_layers_is_crossed_lane_to_lane_without_a_step() {
    let (a, b, pitch, way) = row(16);
    let ca = centre(a).unwrap();
    let left = ca.cross(centre(b).unwrap()).normalize();
    let mut halves = 0;
    for width in [2u8, 3] {
        for phase in [0.25f32, 0.75] {
            let ground = hill(ca, way, pitch, 2.0, 10, phase);
            let road = PlanRoad::new(
                vec![a, b],
                How {
                    width,
                    ..How::default()
                },
            );
            let laid = lay(&road, &ground, &no_trees, &cfg(), 1.0);
            assert_eq!(first_clash(&laid), None, "width {width}, phase {phase}");
            assert!(laid.iter().any(|c| c.lane != 0));
            halves += laid.iter().filter(|c| c.bed.lift_mm > 0).count();
            // never under its own ground: no cut
            for c in &laid {
                let d = c.drawn();
                assert!(d.low_top_m() >= c.base_m as f32, "a cut at {}", c.key);
            }
            let cells: BTreeMap<u32, RoadCell> = laid.iter().map(|c| (c.key, c.drawn())).collect();
            let height = |p: Vec3| -> Option<f32> {
                let k = nearest_key(p)?;
                Some(cells.get(&k)?.top_at(p))
            };
            // Diagonally across the lanes and back, up the whole climb.
            let across = if width == 3 {
                (-2.6f32, 2.6f32)
            } else {
                (0.0, 2.6)
            };
            let length = 14.0 * pitch;
            let mut points = Vec::new();
            for i in 0..=3000 {
                let s = pitch + i as f32 / 3000.0 * (length - pitch);
                let wave = ((s / 6.0).fract() * 2.0 - 1.0).abs();
                let off = across.0 + (across.1 - across.0) * wave;
                let p = (ca + way * (s / RADIUS) + left * (off / RADIUS)).normalize();
                let h = height(p).unwrap_or_else(|| panic!("off the road at {s:.1} m, {off:.1} m"));
                points.push((p, h));
            }
            // The lattice on the sphere is not exactly regular, so a lane
            // sits a little off the half-cell grid and meets its neighbour
            // with a seam of a few centimetres: held under what the walker
            // takes without easing its eye (`walking::STEP_EASE_MIN_M`,
            // 0.12 m). Over half a metre nothing is steeper than a slope.
            let seam = points
                .windows(2)
                .map(|w| (w[1].1 - w[0].1).abs())
                .fold(0.0f32, f32::max);
            let mut worst = 0.0f32;
            for w in points.windows(40) {
                let run = crate::sites::arc_m(w[0].0, w[39].0, RADIUS).max(1e-4);
                worst = worst.max((w[39].1 - w[0].1).abs() / run);
            }
            let climbed = points.last().unwrap().1 - points[0].1;
            assert!(climbed > 9.0, "climbed {climbed}");
            assert!(
                seam < 0.12,
                "width {width}, phase {phase}: a seam of {seam} m"
            );
            assert!(
                worst < 0.5,
                "width {width}, phase {phase}: the steepest is {worst} m a metre, a step"
            );
        }
    }
    assert!(halves > 0, "a half-height bed somewhere");
}

/// The half-height bed is stood on where it is drawn (design 42m): the
/// walker's floor of a lane cut on its bed, rising over its head half.
#[test]
fn a_half_bed_is_stood_on_where_it_is_drawn() {
    let (a, b, _, _) = row(2);
    for bed in [
        Bed {
            lift_mm: 500,
            from: 0,
            to: 1000,
            rise_mm: 1000,
        },
        Bed {
            lift_mm: 500,
            from: 500,
            to: 1000,
            rise_mm: 500,
        },
        Bed {
            lift_mm: 250,
            from: 0,
            to: 400,
            rise_mm: 400,
        },
    ] {
        let cell = RoadCell {
            key: a,
            base_m: 12,
            kind: Kind::Ground,
            raised: false,
            cut_fill: 0,
            bed,
            up: Some(
                neighbours(a)
                    .into_iter()
                    .find(|&n| hex_line(a, b).contains(&n))
                    .unwrap(),
            ),
            from: None,
        };
        let piece =
            super::super::piece::cut(&cell, Surface::Gravel, &[], RADIUS, &|_| 2.0).unwrap();
        let c = centre(a).unwrap();
        for corner in super::super::corners(a) {
            let d = c.lerp(corner, 0.9).normalize();
            let local = piece.solids.frame.local(d * (RADIUS + 30.0));
            let floor = piece.solids.stand_in(local, 50.0).0.unwrap();
            let stood = floor + cell.low_top_m();
            let drawn = cell.top_at(d) + super::super::piece::LIFT_M;
            assert!(
                (stood - drawn).abs() < 0.03,
                "{bed:?}: stood {stood}, drawn {drawn}"
            );
        }
    }
}

/// River ground across a row: `width` cells of water `depth` deep from
/// pitch `x0`, banks at 5.
fn river(
    from: Vec3,
    way: Vec3,
    pitch: f32,
    x0: f32,
    width: f32,
) -> impl Fn(u32) -> Option<Ground> + Sync {
    move |k| {
        let s = (centre(k)? - from).dot(way) * RADIUS / pitch;
        let mut g = dry(5);
        if s > x0 - 0.5 && s < x0 + width - 0.5 {
            g.top_m = 1;
            g.water = Water::River {
                depth_m: 3,
                surface_m: 4,
            };
        }
        Some(g)
    }
}

/// `world/roads`, the player's bridges (design 42m): a river within
/// `max_bridge_cells` is bridged at the bank's height, the deck paved with
/// planks and a log; past it the water past the limit is red; a three-wide
/// bridge has rails only on its outer edges.
#[test]
fn a_river_is_bridged_by_the_worlds_rule_and_a_wide_bridge_is_railed_outside() {
    let (a, b, pitch, way) = row(16);
    let ca = centre(a).unwrap();
    let road = PlanRoad::new(vec![a, b], How::default());
    let laid = lay(
        &road,
        &river(ca, way, pitch, 5.0, 4.0),
        &no_trees,
        &cfg(),
        1.0,
    );
    assert_eq!(first_clash(&laid), None);
    let decks: Vec<&LaidCell> = laid.iter().filter(|c| c.deck.is_some()).collect();
    assert_eq!(decks.len(), 4);
    assert!(decks.iter().all(|c| c.deck == Some(5)));
    let built = decks[0].built(1, Surface::Gravel);
    assert_eq!(built.deck_m, Some(5));
    assert!(matches!(
        super::super::built_cell(decks[0].key, &built).kind,
        Kind::Bridge { deck_m: 5 }
    ));
    let b1 = bill(
        &BTreeMap::new(),
        Some(&road),
        &BTreeMap::new(),
        &|_| false,
        &river(ca, way, pitch, 5.0, 4.0),
        &cfg(),
        1.0,
    );
    assert_eq!((b1.planks, b1.logs), (4 * cfg().bridge_planks, 4));
    assert_eq!(b1.gravel as usize, laid.len() - 4);
    // too wide: the cells past the sixth are red
    let laid = lay(
        &road,
        &river(ca, way, pitch, 3.0, 8.0),
        &no_trees,
        &cfg(),
        1.0,
    );
    let red: Vec<Clash> = laid.iter().filter_map(|c| c.clash).collect();
    assert_eq!(red, vec![Clash::TooWideToBridge; 2]);
    // the world's search asks the same rule
    assert_eq!(
        bridged(
            Water::River {
                depth_m: 3,
                surface_m: 4
            },
            0,
            0,
            5,
            1.0,
            6
        ),
        Some(Some((5, 1)))
    );
    assert_eq!(
        bridged(
            Water::River {
                depth_m: 3,
                surface_m: 4
            },
            6,
            5,
            5,
            1.0,
            6
        ),
        Some(None)
    );
    assert_eq!(
        bridged(
            Water::River {
                depth_m: 3,
                surface_m: 6
            },
            0,
            0,
            5,
            1.0,
            6
        ),
        Some(None)
    );
    assert_eq!(bridged(Water::Dry, 0, 0, 5, 1.0, 6), None);
    // three wide: every lane bridged, rails only outside
    let wide = PlanRoad::new(
        vec![a, b],
        How {
            width: 3,
            ..How::default()
        },
    );
    let laid = lay(
        &wide,
        &river(ca, way, pitch, 5.0, 4.0),
        &no_trees,
        &cfg(),
        1.0,
    );
    assert_eq!(first_clash(&laid), None);
    let keys: BTreeSet<u32> = laid.iter().map(|c| c.key).collect();
    let mut outer_rails = 0;
    for c in laid.iter().filter(|c| c.deck.is_some()) {
        let around: Vec<u32> = neighbours(c.key)
            .into_iter()
            .filter(|n| keys.contains(n))
            .collect();
        let piece =
            super::super::piece::cut(&c.drawn(), Surface::Gravel, &around, RADIUS, &|_| 2.0)
                .unwrap();
        if c.lane == 0 {
            assert!(piece.solids.solids.is_empty(), "no rail between lanes");
        } else {
            outer_rails += piece.solids.solids.len();
        }
    }
    assert!(
        laid.iter()
            .filter(|c| c.deck.is_some() && c.lane != 0)
            .count()
            >= 6
    );
    assert!(outer_rails > 0, "rails on the outer edges");
}

/// Lamp posts stand off the road on its verge, on the left of a one-wide
/// road and both sides of a three-wide road, staggered; a standing post
/// leaves the bill (design 42e, 42m).
#[test]
fn lamp_posts_stand_on_the_verge_and_both_sides_of_a_three_wide_road() {
    let (a, b, _, _) = row(24);
    let flat = slope(0.0);
    for width in [1u8, 3] {
        let road = PlanRoad::new(
            vec![a, b],
            How {
                width,
                lamps: true,
                ..How::default()
            },
        );
        let laid = lay(&road, &flat, &no_trees, &cfg(), 1.0);
        let keys: BTreeSet<u32> = laid.iter().map(|c| c.key).collect();
        let posts = lamp_posts(&road, &laid, &flat, &cfg());
        assert!(posts.len() >= 3, "width {width}: {posts:?}");
        for (beside, post) in &posts {
            assert!(keys.contains(beside) && !keys.contains(post));
            assert!(neighbours(*beside).contains(post));
        }
        let left = laid
            .iter()
            .filter(|c| posts.iter().any(|p| p.0 == c.key) && c.lane >= 0)
            .count();
        let right = laid
            .iter()
            .filter(|c| posts.iter().any(|p| p.0 == c.key) && c.lane < 0)
            .count();
        if width == 3 {
            assert!(left > 0 && right > 0, "both sides");
        } else {
            assert_eq!(right, 0);
        }
        let (none, paved) = (BTreeMap::new(), BTreeMap::new());
        let all = bill(&none, Some(&road), &paved, &|_| false, &flat, &cfg(), 1.0);
        assert_eq!(all.lamps as usize, posts.len());
        assert_eq!((all.lanterns, all.logs), (all.lamps, all.lamps));
        let first = posts[0].1;
        let less = bill(
            &none,
            Some(&road),
            &paved,
            &|k| k == first,
            &flat,
            &cfg(),
            1.0,
        );
        assert_eq!(less.lamps + 1, all.lamps);
    }
}

/// Inserting a stake on a leg (design 42f, 42m): it takes a stake, goes
/// between the leg's two stakes, and the road runs through it; a stake or a
/// cell no road runs over is not a leg.
#[test]
fn a_stake_inserted_on_a_leg_takes_one_and_the_road_runs_through_it() {
    let (a, b, _, _) = row(10);
    let flat = slope(0.0);
    let mut plans = BTreeMap::new();
    plans.insert(
        1,
        RoadPlan {
            roads: vec![PlanRoad::new(vec![a, b], How::default())],
        },
    );
    let mid = hex_line(a, b)[5];
    let e = insert_stake(&plans, mid, &flat, &no_trees, &cfg(), 1.0)
        .expect("on a leg")
        .expect("allowed");
    assert_eq!(e.stakes, -1);
    assert_eq!(e.plans[&1].roads[0].stakes, vec![a, mid, b]);
    assert!(e.plans[&1].roads[0].centre().contains(&mid));
    assert!(insert_stake(&plans, a, &flat, &no_trees, &cfg(), 1.0).is_none());
    let off = key(0.0, 40.0);
    assert!(insert_stake(&plans, off, &flat, &no_trees, &cfg(), 1.0).is_none());
}
