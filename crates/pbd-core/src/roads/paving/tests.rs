use super::*;
use crate::roads::{Ground, Water, nearest_key};
use glam::Vec3;

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

/// Ground rising `per_m` layers a metre to the east, with a river where
/// `wet` says, and nothing else in the way.
fn slope(
    per_m: f32,
    wet: impl Fn(f32, f32) -> bool + Sync,
) -> impl Fn(u32) -> Option<Ground> + Sync {
    move |key| {
        let (e, n) = local(centre(key)?);
        Some(Ground {
            top_m: (e * per_m).floor() as i32,
            water: if wet(e, n) {
                Water::River {
                    depth_m: 3,
                    surface_m: 10,
                }
            } else {
                Water::Dry
            },
            town: false,
            player: false,
        })
    }
}

fn key(e: f32, n: f32) -> u32 {
    nearest_key(at(e, n)).unwrap()
}

/// `world/roads`, "A road is marked before it is paved": two stakes on a
/// hillside mark the cells between, never more than a layer apart.
#[test]
fn a_segment_never_steps_more_than_a_layer() {
    let cfg = RoadsConfig::default();
    let g = slope(0.3, |_, _| false);
    let (a, b) = (key(0.0, 0.0), key(30.0, 6.0));
    let cells = segment(a, b, &g, &cfg, RADIUS).unwrap();
    assert_eq!(cells.first(), Some(&a));
    assert_eq!(cells.last(), Some(&b));
    assert!(cells.len() >= 11, "{} cells", cells.len());
    for w in cells.windows(2) {
        assert!(touching(w[0], w[1]), "neighbours");
        let (h0, h1) = (g(w[0]).unwrap().top_m, g(w[1]).unwrap().top_m);
        assert!((h0 - h1).abs() <= 1, "{h0} to {h1}");
    }
}

/// "Too steep", "Too far" and water: refused, and nothing is marked.
#[test]
fn a_stake_with_no_way_is_refused() {
    let cfg = RoadsConfig::default();
    let flat = slope(0.0, |_, _| false);
    let steep = slope(1.0, |_, _| false);
    let river = slope(0.0, |e, _| (14.0..22.0).contains(&e));
    let plans = BTreeMap::new();
    let first = drive(&plans, 1, None, key(0.0, 0.0), &steep, &cfg, RADIUS).unwrap();
    let mut plans = BTreeMap::new();
    plans.insert(first.id, first.plan.clone());
    assert_eq!(
        drive(&plans, 2, Some(1), key(20.0, 0.0), &steep, &cfg, RADIUS),
        Err(Refused::TooSteep)
    );
    assert_eq!(
        drive(&plans, 2, Some(1), key(120.0, 0.0), &flat, &cfg, RADIUS),
        Err(Refused::TooFar)
    );
    assert_eq!(
        drive(&plans, 2, Some(1), key(36.0, 0.0), &river, &cfg, RADIUS),
        Err(Refused::Water)
    );
    assert_eq!(
        drive(&plans, 2, Some(1), key(18.0, 0.0), &river, &cfg, RADIUS),
        Err(Refused::Water),
        "a stake in the river"
    );
    assert_eq!(plans[&1], first.plan, "nothing changed");
}

/// A middle stake taken back: its neighbours are joined again where there
/// is a way, and the plan splits where there is none.
#[test]
fn taking_back_a_middle_stake_rejoins_or_splits() {
    let cfg = RoadsConfig::default();
    let flat = slope(0.0, |_, _| false);
    let mut plans = BTreeMap::new();
    let mut open = None;
    for e in [0.0, 25.0, 50.0] {
        let d = drive(
            &plans,
            open.unwrap_or(1),
            open,
            key(e, 0.0),
            &flat,
            &cfg,
            RADIUS,
        )
        .unwrap();
        open = Some(d.id);
        plans.insert(d.id, d.plan);
    }
    let plan = plans[&1].clone();
    assert_eq!(plan.stakes.len(), 3);
    // Split: a river has come between since (the search is asked again).
    let cut = slope(0.0, |e, _| (5.0..12.0).contains(&e));
    let (first, second) = take_back(&plan, plan.stakes[1], &cut, &cfg, RADIUS);
    let (first, second) = (first.unwrap(), second.unwrap());
    assert_eq!(first.stakes, vec![plan.stakes[0]]);
    assert_eq!(second.stakes, vec![plan.stakes[2]]);
    assert_eq!(first.cells, vec![plan.stakes[0]]);
    assert_eq!(second.cells, vec![plan.stakes[2]]);
    // Rejoined where there is a way.
    let (joined, none) = take_back(&plan, plan.stakes[1], &flat, &cfg, RADIUS);
    let joined = joined.unwrap();
    assert!(none.is_none());
    assert_eq!(joined.stakes, vec![plan.stakes[0], plan.stakes[2]]);
    assert_eq!(joined.cells.first(), Some(&plan.stakes[0]));
    assert_eq!(joined.cells.last(), Some(&plan.stakes[2]));
    for w in joined.cells.windows(2) {
        assert!(touching(w[0], w[1]));
    }
    // An end stake: its segment goes.
    let (shorter, _) = take_back(&plan, plan.stakes[2], &flat, &cfg, RADIUS);
    assert_eq!(shorter.unwrap().cells.last(), Some(&plan.stakes[1]));
    // The last stake: the plan goes.
    let lone = Plan {
        stakes: vec![plan.stakes[0]],
        cells: vec![plan.stakes[0]],
    };
    assert_eq!(
        take_back(&lone, plan.stakes[0], &flat, &cfg, RADIUS),
        (None, None)
    );
}

/// `world/roads`, "Marks are saved": a plan reads back cell for cell.
#[test]
fn a_plan_reads_back_cell_for_cell() {
    let cfg = RoadsConfig::default();
    let g = slope(0.2, |_, _| false);
    let d0 = drive(&BTreeMap::new(), 1, None, key(0.0, 0.0), &g, &cfg, RADIUS).unwrap();
    let mut plans = BTreeMap::from([(d0.id, d0.plan)]);
    let d1 = drive(&plans, 2, Some(1), key(20.0, 10.0), &g, &cfg, RADIUS).unwrap();
    plans.insert(d1.id, d1.plan.clone());
    let mut records = Records::new();
    records.put(plan_record(1, Some(&d1.plan)));
    let line = plan_record(1, Some(&d1.plan)).line(&crate::records::Author::Creation);
    let (_, back) = Record::parse_line(&line).unwrap();
    assert_eq!(
        back.read::<Option<Plan>>().flatten().as_ref(),
        Some(&d1.plan)
    );
    assert_eq!(super::plans(&records)[&1], d1.plan);
    assert_eq!(next_plan_id(&records), 2);
    records.put(plan_record(1, None));
    assert!(super::plans(&records).is_empty(), "taken back whole");
    assert_eq!(next_plan_id(&records), 2, "never reused");
}

/// Design 40: a paved plan up ten layers never steps, and paving the cells
/// in any order gives the same cells.
#[test]
fn paving_in_any_order_gives_the_same_road() {
    let cfg = RoadsConfig::default();
    let g = slope(0.33, |_, _| false);
    let mut plans = BTreeMap::new();
    let mut open = None;
    for e in [0.0, 15.0, 30.0] {
        let d = drive(&plans, 1, open, key(e, 0.0), &g, &cfg, RADIUS).unwrap();
        open = Some(d.id);
        plans.insert(d.id, d.plan);
    }
    let plan = &plans[&1];
    let forward: Vec<BuiltCell> = plan
        .cells
        .iter()
        .map(|&k| paved_cell(1, plan, k, &g).unwrap())
        .collect();
    let mut backward: Vec<BuiltCell> = plan
        .cells
        .iter()
        .rev()
        .map(|&k| paved_cell(1, plan, k, &g).unwrap())
        .collect();
    backward.reverse();
    assert_eq!(forward, backward);
    let rise = g(*plan.cells.last().unwrap()).unwrap().top_m - g(plan.cells[0]).unwrap().top_m;
    assert!(rise >= 8, "climbs {rise} layers");
    // Never a step: each cell's top meets the next's at their shared side.
    for (w, k) in forward.windows(2).zip(plan.cells.windows(2)) {
        let (a, b) = (drawn(k[0], &w[0]), drawn(k[1], &w[1]));
        let side = crate::roads::edge_middle(k[0], k[1]).unwrap();
        let (ta, tb) = (a.top_at(side), b.top_at(side));
        assert!((ta - tb).abs() < 0.05, "{ta} against {tb}");
    }
    // Marked is what is not paved.
    let mut paved = BTreeMap::new();
    paved.insert(plan.cells[2], forward[2]);
    let m = marked(&plans, &paved);
    assert_eq!(m.len(), plan.cells.len() - 1);
    assert!(!m.contains_key(&plan.cells[2]));
    assert_eq!(
        m[&plan.cells[1]],
        (Some(plan.cells[0]), Some(plan.cells[2]))
    );
}

/// The way between two waypoints (the stake's rule, and the road planner's
/// live ghost later): the cells a road takes, and each cell on the straight
/// line that refuses a road with its reason.
#[test]
fn a_way_names_each_clashing_cell_and_why() {
    let cfg = RoadsConfig::default();
    let river = slope(0.0, |e, _| (14.0..20.0).contains(&e));
    let (a, b) = (key(0.0, 0.0), key(36.0, 0.0));
    let w = way(a, b, &river, &cfg, RADIUS);
    assert_eq!(w.cells, Err(Refused::Water));
    assert!(!w.clashes.is_empty());
    assert!(w.clashes.iter().all(|(_, why)| *why == Refused::Water));
    for (k, _) in &w.clashes {
        let (e, _) = local(centre(*k).unwrap());
        assert!((12.0..22.0).contains(&e), "a clash at {e} m");
    }
    // A clear way has a route and no clash.
    let flat = slope(0.0, |_, _| false);
    let w = way(a, key(20.0, 0.0), &flat, &cfg, RADIUS);
    assert!(w.cells.is_ok() && w.clashes.is_empty());
    // A cliff: the cell past the step clashes as too steep.
    let cliff = |k: u32| {
        let (e, _) = local(centre(k)?);
        let mut g = flat(k)?;
        g.top_m = if e > 10.0 { 3 } else { 0 };
        Some(g)
    };
    let w = way(a, key(20.0, 0.0), &cliff, &cfg, RADIUS);
    assert!(w.clashes.iter().any(|(_, why)| *why == Refused::TooSteep));
}
