//! `building-planner` tasks 2.1 to 2.8: plans, the planner's checks, the
//! world's check, costs, the site diff, records, the world setting and the
//! presets.

use super::costs::CostTable;
use super::plan::*;
use super::planner::{Check, Tunables, check_plan, plan_ok};
use super::presets::{Presets, from_template};
use super::rules::Rule;
use super::site::*;
use super::tests::{kits, patch_at, plot, template};
use super::*;
use crate::records::Records;
use std::collections::{BTreeMap, BTreeSet};

const GROUND: u16 = 150;

fn tun() -> Tunables {
    let t = Tunables::shipped();
    t.validate().expect("building_planner.ron is valid");
    t
}

fn presets() -> Presets {
    Presets::shipped()
}

fn cottage() -> Plan {
    presets().get("cottage").unwrap().plan(Size::Standard)
}

fn pieces_of(c: &Charted) -> BTreeSet<Piece> {
    c.iter().map(|(_, p)| p.clone()).collect()
}

impl PartialOrd for Piece {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Piece {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.kit, &self.shape).cmp(&(&other.kit, &other.shape))
    }
}

/// Task 2.1: a plan's pieces in plan coordinates, charted onto real cells.
#[test]
fn a_room_is_walled_by_its_edges() {
    let mut plan = Plan::new("timber");
    plan.storeys[0] = [(0, 0), (1, 0)].into();
    let pieces = plan.pieces();
    let walls = pieces
        .iter()
        .filter(|p| matches!(p, PlanPiece::Wall { .. }))
        .count();
    // Two hexagons share one edge: ten outer edges, no wall between them.
    assert_eq!(walls, 10);
    assert_eq!(
        pieces
            .iter()
            .filter(|p| matches!(p, PlanPiece::Floor { .. }))
            .count(),
        2
    );
    assert!(pieces.iter().any(
        |p| matches!(p, PlanPiece::Roof { cells, shape: RoofShape::Gable, .. } if cells.len() == 2)
    ));
}

#[test]
fn every_preset_charts_at_all_six_turns_to_the_same_pieces_turned() {
    let (patch, at) = plot();
    for preset in &presets().presets {
        for size in Size::ALL {
            let plan = preset.plan(size);
            for t in 0..6u8 {
                let charted = chart_plan(patch, &plan, *at, t as usize, GROUND)
                    .unwrap_or_else(|e| panic!("{} {size:?} at turn {t}: {e}", preset.id));
                assert_eq!(charted.len(), plan.pieces().len());
                // The same plan turned k sixths, charted k sixths on, is the
                // same pieces on the same cells.
                for k in 1..6u8 {
                    let other =
                        chart_plan(patch, &plan.turned(k), *at, ((t + k) % 6) as usize, GROUND)
                            .unwrap();
                    assert_eq!(
                        pieces_of(&charted),
                        pieces_of(&other),
                        "{} {size:?}: turn {t} against the plan turned {k}",
                        preset.id
                    );
                }
            }
        }
    }
}

#[test]
fn the_flat_stand_in_is_a_true_hex_grid() {
    let patch = flat_patch(5);
    let cells = Cells::new(&patch);
    for (i, c) in patch.cells.iter().enumerate() {
        for (side, &n) in c.neighbors.iter().enumerate() {
            if n == usize::MAX {
                continue;
            }
            let back = patch.cells[n]
                .neighbors
                .iter()
                .position(|&m| m == i)
                .expect("neighbours list each other");
            assert_eq!(back, (side + 3) % 6);
            // The shared edge's two corners, the other way round.
            let a = c.corners[side];
            let b = c.corners[(side + 1) % 6];
            let other = &patch.cells[n];
            assert!(a.distance(other.corners[(back + 1) % 6]) < 1e-6);
            assert!(b.distance(other.corners[back]) < 1e-6);
        }
        assert!(!cells.pentagon(patch.keys[i]));
    }
}

#[test]
fn a_plan_round_trips_through_its_record_body() {
    let plan = presets().get("house").unwrap().plan(Size::Large);
    let body = BlueprintBody {
        name: "My house".into(),
        preset: Some(("house".into(), Size::Large)),
        plan: Some(plan.clone()),
    };
    let record = blueprint_record(1, &body);
    assert_eq!(record.read::<BlueprintBody>(), Some(body));
}

/// Task 2.2: the planner's checks, one per rule, each with its reason.
#[test]
fn the_planner_names_each_broken_rule() {
    let t = tun();
    // A good cottage passes.
    assert!(
        check_plan(&cottage(), &t).is_empty(),
        "{:?}",
        check_plan(&cottage(), &t)
    );
    // Too big.
    let mut big = Plan::new("timber");
    big.storeys[0] = (-4..=4)
        .flat_map(|q| (-4..=4).map(move |r| (q, r)))
        .filter(|&c| distance(c) <= 3)
        .collect();
    let checks: Vec<Check> = check_plan(&big, &t).iter().map(|i| i.check).collect();
    assert!(checks.contains(&Check::PlanSize), "{checks:?}");
    assert_eq!(Check::PlanSize.key(), "planner.check.plan_size");
    let mut far = cottage();
    far.storeys[0].insert((5, 0));
    assert!(
        check_plan(&far, &t)
            .iter()
            .any(|i| i.check == Check::PlanSize)
    );
    // Too many storeys.
    let mut tall = cottage();
    for _ in 0..3 {
        tall.add_storey();
    }
    assert!(
        check_plan(&tall, &t)
            .iter()
            .any(|i| i.check == Check::Storeys)
    );
    // No way in: a warning, not a refusal.
    let mut shut = cottage();
    shut.edges.retain(|_, m| *m != Mark::Door);
    let issues = check_plan(&shut, &t);
    assert!(issues.iter().any(|i| i.check == Check::WayIn));
    assert!(!Check::WayIn.refuses());
    assert!(plan_ok(&shut, &t));
    assert_eq!(Check::WayIn.key(), "planner.check.way_in");
    // An upper storey with no stair to it is fine to the rules, but its
    // floor over nothing is held up by nothing.
    let mut over = cottage();
    over.add_storey();
    over.storeys[1] = [(9, 9)]
        .into_iter()
        .map(|c| (c.0 - 9 + 3, c.1 - 9))
        .collect();
    let issues = check_plan(&over, &t);
    assert!(
        issues
            .iter()
            .any(|i| i.check == Check::Rule(Rule::HeldUp) && i.storey == Some(1)),
        "{issues:?}"
    );
    assert_eq!(Check::Rule(Rule::HeldUp).key(), "build.refused.held_up");
    // A wall across a stair's flight.
    let mut house = presets().get("house").unwrap().plan(Size::Standard);
    let st = house.stairs[0];
    house.edges.insert(edge(0, st.foot, st.side), Mark::Wall);
    let issues = check_plan(&house, &t);
    assert!(
        issues
            .iter()
            .any(|i| i.check == Check::Rule(Rule::StairCut)),
        "{issues:?}"
    );
    // A stair with no landing: the storey above taken back off its head.
    let mut house = presets().get("house").unwrap().plan(Size::Standard);
    let st = house.stairs[0];
    let land = step(step(st.foot, st.side), st.side);
    house.storeys[1].remove(&land);
    house.prune();
    let issues = check_plan(&house, &t);
    assert!(
        issues.iter().any(|i| matches!(
            i.check,
            Check::Rule(Rule::Landing | Rule::HeldUp | Rule::RoofOverlap)
        )),
        "{issues:?}"
    );
    // A lantern on a stair.
    let mut house = presets().get("house").unwrap().plan(Size::Standard);
    let st = house.stairs[0];
    house.lanterns.push(PlanLantern {
        cell: st.foot,
        storey: 0,
    });
    let issues = check_plan(&house, &t);
    assert!(
        issues.iter().any(|i| i.check == Check::Rule(Rule::Clear)),
        "{issues:?}"
    );
    // Two slots crossing in one cell.
    let mut c = cottage();
    let cell = *c.storeys[0].iter().next().unwrap();
    c.slots = vec![
        PlanSlot {
            cell,
            storey: 0,
            side: 0,
            kind: crate::furniture::Kind::Bed,
        },
        PlanSlot {
            cell,
            storey: 0,
            side: 3,
            kind: crate::furniture::Kind::Bed,
        },
    ];
    let issues = check_plan(&c, &t);
    assert!(
        issues.iter().any(|i| matches!(
            i.check,
            Check::Rule(Rule::Overlap | Rule::Slot | Rule::InWall)
        )),
        "{issues:?}"
    );
    // Nothing painted.
    assert_eq!(check_plan(&Plan::new("timber"), &t)[0].check, Check::Empty);
}

#[test]
fn the_tools_draw_the_plan() {
    let mut p = Plan::new("timber");
    let aim = |cell, side| Aim {
        cell,
        side,
        at_edge: false,
    };
    p.apply(Tool::Room, 0, aim((0, 0), 0), None, 4).unwrap();
    p.apply(Tool::Room, 0, aim((1, 0), 0), Some(true), 4)
        .unwrap();
    assert_eq!(p.storeys[0].len(), 2);
    // A door on the outer edge, clicked from outside (M6 version 3).
    let outside = step((0, 0), 3);
    p.apply(
        Tool::Door,
        0,
        Aim {
            cell: outside,
            side: 0,
            at_edge: true,
        },
        None,
        4,
    )
    .unwrap();
    assert_eq!(p.edges.get(&edge(0, (0, 0), 3)), Some(&Mark::Door));
    // A window on an inside edge is refused.
    assert_eq!(
        p.apply(Tool::Window, 0, aim((0, 0), 0), None, 4),
        Err(NoUse::InsideWindow)
    );
    // Erase takes the door back, then the cell.
    p.apply(
        Tool::Erase,
        0,
        Aim {
            cell: (0, 0),
            side: 3,
            at_edge: true,
        },
        None,
        4,
    )
    .unwrap();
    assert!(p.edges.is_empty());
    p.apply(Tool::Lantern, 0, aim((1, 0), 0), None, 4).unwrap();
    p.apply(Tool::Erase, 0, aim((1, 0), 0), None, 4).unwrap();
    assert!(p.lanterns.is_empty() && p.storeys[0].len() == 2);
    p.apply(Tool::Erase, 0, aim((1, 0), 0), None, 4).unwrap();
    assert_eq!(p.storeys[0].len(), 1);
    // Painting past the reach is refused.
    assert_eq!(
        p.apply(Tool::Room, 0, aim((5, 0), 0), None, 4),
        Err(NoUse::TooFar)
    );
}

/// Task 2.3: the whole site against the world, first failure named.
#[test]
fn the_world_refuses_a_site_for_one_reason() {
    let t = tun();
    let plan = cottage();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let flat = |_: u32| Some(GROUND);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let free = |_: u32, _: u16| false;
    let charted = chart_plan(patch, &plan, *at, 0, GROUND).unwrap();
    let world = World {
        site: &site,
        blocked: &free,
        pieces: &[],
    };
    assert_eq!(check_site(&world, &charted, GROUND, &t, &plan), Ok(()));

    // A slope past max_footing.
    let low_cell = match charted[0].1.shape {
        Shape::Floor { cell, .. } => cell,
        _ => unreachable!(),
    };
    let slope = move |c: u32| Some(if c == low_cell { GROUND - 3 } else { GROUND });
    let steep = Site {
        cells: &cells,
        ground: &slope,
    };
    let world = World {
        site: &steep,
        blocked: &free,
        pieces: &[],
    };
    let err = check_site(&world, &charted, GROUND, &t, &plan).unwrap_err();
    assert_eq!(err.0, Refusal::Steep);
    assert_eq!(err.0.key(), "site.refused.steep");

    // A tree on one ground cell: only that block, never its column.
    let tree = move |c: u32, l: u16| c == low_cell && l == GROUND;
    let world = World {
        site: &site,
        blocked: &tree,
        pieces: &[],
    };
    let (why, hit) = check_site(&world, &charted, GROUND, &t, &plan).unwrap_err();
    assert_eq!(why, Refusal::InTheWay);
    assert!(hit.iter().all(|&i| charted[i].1.shape.layer() == GROUND));
    // The same tree a storey down from a second storey's floor is no
    // matter: a block above the tree is not in the way.
    let high_tree = move |c: u32, l: u16| c == low_cell && l == GROUND + 20;
    let world = World {
        site: &site,
        blocked: &high_tree,
        pieces: &[],
    };
    assert_eq!(check_site(&world, &charted, GROUND, &t, &plan), Ok(()));

    // A piece already in a slot.
    let standing = vec![Piece::new("stone", charted[0].1.shape.clone())];
    let world = World {
        site: &site,
        blocked: &free,
        pieces: &standing,
    };
    assert_eq!(
        check_site(&world, &charted, GROUND, &t, &plan)
            .unwrap_err()
            .0,
        Refusal::Taken
    );
    // Its own piece built is its own.
    let mine = vec![charted[0].1.clone()];
    let world = World {
        site: &site,
        blocked: &free,
        pieces: &mine,
    };
    assert_eq!(check_site(&world, &charted, GROUND, &t, &plan), Ok(()));

    // A pentagon.
    let five = crate::topology::icosahedron().0[0];
    let (ppatch, pat) = patch_at(five);
    let pcells = Cells::new(&ppatch);
    let psite = Site {
        cells: &pcells,
        ground: &flat,
    };
    let near = ppatch.cells[pat].neighbors[0];
    match chart_plan(&ppatch, &plan, near, 0, GROUND) {
        Err(_) => {}
        Ok(charted) => {
            let world = World {
                site: &psite,
                blocked: &free,
                pieces: &[],
            };
            assert_eq!(
                check_site(&world, &charted, GROUND, &t, &plan)
                    .unwrap_err()
                    .0,
                Refusal::Pentagon
            );
        }
    }
    assert!(chart_plan(&ppatch, &plan, pat, 0, GROUND).is_err());
}

/// Task 2.4: costs read and validated, every kit costed or locked.
#[test]
fn every_kit_has_a_cost_or_a_lock() {
    let costs = CostTable::shipped();
    costs.validate().expect("building_costs.ron is valid");
    for kit in &kits().kits {
        let costed = costs.families.contains_key(&kit.name);
        let locked = costs.locked.contains_key(&kit.name);
        assert!(
            costed ^ locked,
            "kit {} is costed {costed}, locked {locked}",
            kit.name
        );
        if locked {
            let key = costs.kit(&kit.name).unwrap_err().0;
            assert!(crate::locale::current_locale().get(&key).is_some(), "{key}");
        }
    }
    let plan = cottage();
    let (patch, at) = plot();
    let charted = chart_plan(patch, &plan, *at, 0, GROUND).unwrap();
    let bill = costs.bill(charted.iter().map(|(_, p)| p), 1.0).unwrap();
    assert!(bill["planks"] > 20, "{bill:?}");
    let doubled = costs.bill(charted.iter().map(|(_, p)| p), 2.0).unwrap();
    assert_eq!(doubled["planks"], bill["planks"] * 2);
    // Scaled down, never below one.
    let tiny = costs.bill(charted.iter().map(|(_, p)| p), 0.01).unwrap();
    assert!(tiny.values().all(|&n| n >= 1));
}

/// Task 2.5: the site diff, dropping a room and adding a storey.
#[test]
fn changing_a_site_marks_what_is_built_and_drops_the_rest() {
    let t = tun();
    let costs = CostTable::shipped();
    let (patch, at) = plot();
    let old_plan = presets().get("cottage").unwrap().plan(Size::Large);
    let old = chart_plan(patch, &old_plan, *at, 0, GROUND).unwrap();
    let old_pieces: Vec<Piece> = old.iter().map(|(_, p)| p.clone()).collect();
    // The ground floors and walls are built.
    let built: BTreeMap<u64, Piece> = old_pieces
        .iter()
        .filter(|p| matches!(p.shape, Shape::Floor { .. } | Shape::Wall { .. }))
        .map(|p| (p.id(), p.clone()))
        .collect();
    // Drop a room: the cell furthest along.
    let mut smaller = old_plan.clone();
    let gone = *smaller.storeys[0].iter().max().unwrap();
    smaller.storeys[0].remove(&gone);
    smaller.prune();
    let new = chart_plan(patch, &smaller, *at, 0, GROUND).unwrap();
    let new_pieces: Vec<Piece> = new.iter().map(|(_, p)| p.clone()).collect();
    let d = diff(&old_pieces, &new_pieces, &built, &costs, &t);
    assert!(
        d.marked
            .iter()
            .any(|p| matches!(p.shape, Shape::Floor { .. }))
    );
    assert!(d.marked.iter().all(|p| built.contains_key(&p.id())));
    assert!(d.gone.iter().all(|p| !built.contains_key(&p.id())));
    assert!(d.refund.get("planks").copied().unwrap_or(0) > 0);
    // Nothing built is lost without being marked.
    for p in &old_pieces {
        if built.contains_key(&p.id()) {
            assert!(d.unchanged.contains(p) || d.marked.contains(p), "{p:?}");
        }
    }
    // Add a storey: the ground stays, the old roof is taken down and a
    // new storey's pieces are new ghosts.
    let mut taller = old_plan.clone();
    taller.add_storey();
    super::presets::auto_stairs(&mut taller);
    let new = chart_plan(patch, &taller, *at, 0, GROUND).unwrap();
    let new_pieces: Vec<Piece> = new.iter().map(|(_, p)| p.clone()).collect();
    let mut with_roof = built.clone();
    for p in old_pieces
        .iter()
        .filter(|p| matches!(p.shape, Shape::Roof { .. }))
    {
        with_roof.insert(p.id(), p.clone());
    }
    let d = diff(&old_pieces, &new_pieces, &with_roof, &costs, &t);
    assert!(
        d.marked
            .iter()
            .any(|p| matches!(p.shape, Shape::Roof { .. }))
    );
    assert!(
        d.new
            .iter()
            .any(|p| p.shape.layer() == GROUND + STOREY && matches!(p.shape, Shape::Wall { .. }))
    );
    // A refund share of nought gives nothing back.
    let none = Tunables {
        refund_share: 0.0,
        ..t.clone()
    };
    assert!(
        diff(&old_pieces, &new_pieces, &with_roof, &costs, &none)
            .refund
            .is_empty()
    );
}

/// Task 2.6: records; an old save has neither kind and reads none.
#[test]
fn an_old_save_has_no_sites_and_a_site_reads_back() {
    let mut records = Records::new();
    records.put(placed(&Piece::new(
        "timber",
        Shape::Floor {
            cell: 7,
            layer: GROUND,
        },
    )));
    assert!(sites(&records).is_empty());
    assert!(blueprints(&records).is_empty());
    assert_eq!(next_id(&records, SITE_RECORD), 1);
    let body = SiteBody {
        name: "blueprint.preset.cottage".into(),
        preset: Some(("cottage".into(), Size::Standard)),
        plan: cottage(),
        chart: SiteChart {
            origin: 1234,
            turn: 2,
            base: GROUND,
        },
        marked: BTreeMap::new(),
        scaffold: BTreeSet::new(),
        cleared: false,
    };
    let record = site_record(1, &body);
    // Through the journal's line and back, as a reopened world reads it.
    let line = record.line(&crate::records::Author::Player(0));
    let (_, back) = crate::records::Record::parse_line(&line).expect("a record line");
    records.put(back);
    assert_eq!(sites(&records).get(&1), Some(&body));
    assert_eq!(next_id(&records, SITE_RECORD), 2);
    // Built is derived from the piece records.
    let (patch, at) = plot();
    let charted = chart_plan(patch, &body.plan, *at, 0, GROUND).unwrap();
    let mut pieces = BTreeMap::new();
    let p = progress(&charted, 0, &pieces);
    assert_eq!((p.built, p.finished), (0, false));
    for (_, piece) in &charted {
        if !matches!(piece.shape, Shape::Furniture { .. }) {
            pieces.insert(piece.id(), piece.clone());
        }
    }
    let p = progress(&charted, 0, &pieces);
    assert!(p.finished && p.built == p.total);
    // A cleared site is not shown.
    let cleared = SiteBody {
        cleared: true,
        ..body
    };
    records.put(site_record(1, &cleared));
    assert!(sites(&records).is_empty());
}

/// Task 2.7: a world with no setting is Free.
#[test]
fn a_missing_setting_reads_free() {
    assert_eq!(BuildingSetting::of_saved(None), BuildingSetting::Free);
    assert_eq!(
        BuildingSetting::of_saved(Some(BuildingSetting::Planned)),
        BuildingSetting::Planned
    );
    assert_eq!(BuildingSetting::NEW_WORLD, BuildingSetting::Planned);
}

/// Task 2.8: every preset passes the planner's checks at every size, and
/// the templates' plain buildings join the library.
#[test]
fn every_preset_passes_at_every_size() {
    let t = tun();
    let p = presets();
    assert_eq!(p.presets.len(), 10);
    let ids: BTreeSet<&str> = p.presets.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids.len(), 10, "preset ids are unique");
    let costs = CostTable::shipped();
    for preset in &p.presets {
        assert!(costs.kit(&preset.kit).is_ok(), "{}", preset.id);
        assert_eq!(preset.sizes.len(), 3, "{} has three sizes", preset.id);
        assert!(
            crate::locale::current_locale()
                .get(&format!("blueprint.preset.{}", preset.id))
                .is_some()
        );
        for size in Size::ALL {
            let plan = preset.plan(size);
            let issues = check_plan(&plan, &t);
            assert!(issues.is_empty(), "{} {size:?}: {issues:?}", preset.id);
            assert!(
                !plan.slots.is_empty() || preset.slots.is_empty(),
                "{} {size:?} has slots",
                preset.id
            );
        }
    }
    let kits = kits();
    let mut joined = 0;
    let mut plain = 0;
    for name in ["village", "walled", "harbour", "desert", "tundra", "jungle"] {
        for def in &template(name).buildings {
            let Some(kit) = kits.get(&def.kit) else {
                continue;
            };
            if !super::blueprint::plain(def, kit) {
                continue;
            }
            plain += 1;
            if from_template(def, kit).is_some_and(|plan| plan_ok(&plan, &t)) {
                joined += 1;
            }
        }
    }
    assert!(plain > 0);
    assert!(
        joined * 2 >= plain,
        "{joined} of {plain} template buildings pass"
    );
}

/// Design 7: a house standing outside any site reads back as the plan it
/// was built from, so adopting it changes nothing that stands.
#[test]
fn an_adopted_house_charts_back_to_its_own_pieces() {
    let (patch, at) = plot();
    for id in ["cottage", "house", "townhouse"] {
        let plan = presets().get(id).unwrap().plan(Size::Standard);
        let charted = chart_plan(patch, &plan, *at, 0, GROUND).unwrap();
        let standing: Vec<Piece> = charted.iter().map(|(_, p)| p.clone()).collect();
        let back = adopt(patch, *at, GROUND, &standing);
        let again = chart_plan(patch, &back, *at, 0, GROUND).unwrap();
        assert_eq!(pieces_of(&again), pieces_of(&charted), "{id}");
    }
}

/// Design 6: scaffolding stands outside every outer edge of the storey,
/// its boards solid at the storey's floor line.
#[test]
fn scaffolding_rings_the_storey() {
    let (patch, at) = plot();
    let plan = presets().get("house").unwrap().plan(Size::Standard);
    let cells = Cells::new(patch);
    let pc = PlanCells::new(patch, *at, 0, 4);
    let bays: Vec<super::scaffold::Bay> = scaffold_bays(&plan, 1)
        .into_iter()
        .map(|(h, d)| super::scaffold::Bay {
            inside: pc.key(h).unwrap(),
            outside: pc.across(h, d).unwrap(),
        })
        .collect();
    assert_eq!(
        bays.len(),
        plan.storeys[1].len() * 6 - 2 * shared(&plan.storeys[1])
    );
    let origin = patch.keys[*at];
    let (meshes, solids) =
        super::scaffold::scaffold(&cells, origin, &bays, GROUND, GROUND + STOREY, 300.0).unwrap();
    for tex in ["plank", "poles", "rope"] {
        assert!(
            meshes.get(tex).is_some_and(|m| !m.positions.is_empty()),
            "{tex}"
        );
    }
    let boards = solids
        .solids
        .iter()
        .filter(|s| (s.y1 - 3.0).abs() < 1e-3)
        .count();
    assert_eq!(boards, bays.len());
    let poles = &meshes["poles"];
    assert!(
        poles
            .normals
            .iter()
            .all(|n| n.iter().all(|v| v.is_finite()))
    );
    let cost = scaffold_cost(14, &tun());
    assert_eq!(cost, vec![("planks".into(), 28), ("rope".into(), 5)]);
    // The lime line and the stakes are on the ground storey's outside.
    assert!(!outline(&plan).is_empty());
    assert!(!stake_spots(&plan).is_empty());
}

/// Edges shared by two cells of a room.
fn shared(cells: &BTreeSet<Hex>) -> usize {
    cells
        .iter()
        .flat_map(|&c| (0..6u8).map(move |d| (c, d)))
        .filter(|&(c, d)| cells.contains(&step(c, d)))
        .count()
        / 2
}
