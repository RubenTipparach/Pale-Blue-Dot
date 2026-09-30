use super::chart::{Patch, chart};
use super::ground::{GroundAt, TownGround};
use super::pieces::{Meshes, cut_building};
use super::*;
use crate::topology::dual_sphere;
use glam::Vec3;
use std::collections::BTreeSet;
use std::sync::OnceLock;

/// The gold-standard body: level 7 on a 300 m radius gives the game's
/// 2.833 m cells (CLAUDE.md), so a town cut on it is cut at the game's
/// scale.
const RADIUS_M: f32 = 300.0;

fn kits() -> Kits {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/kits.ron"
    ))
    .expect("kits.ron");
    ron::from_str(&text).expect("kits.ron parses")
}

fn village() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/village.json"
    ))
    .expect("village.json");
    serde_json::from_str(&text).expect("village.json parses")
}

fn walled() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/town.json"
    ))
    .expect("town.json");
    serde_json::from_str(&text).expect("town.json parses")
}

/// A patch of the level-7 sphere round a cell well away from the
/// pentagons.
fn patch() -> &'static (Patch, usize) {
    static PATCH: OnceLock<(Patch, usize)> = OnceLock::new();
    PATCH.get_or_init(|| {
        let cells = dual_sphere(7);
        let anchor = Vec3::new(0.3, 0.5, 0.8).normalize();
        let centre = cells
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.direction
                    .dot(anchor)
                    .total_cmp(&b.1.direction.dot(anchor))
            })
            .map(|(i, _)| i)
            .unwrap();
        // Every cell within 0.4 rad (120 m at 300 m), renumbered.
        let reach = (0.4f32).cos();
        let kept: Vec<usize> = (0..cells.len())
            .filter(|&i| cells[i].direction.dot(cells[centre].direction) > reach)
            .collect();
        let mut index = vec![usize::MAX; cells.len()];
        for (k, &i) in kept.iter().enumerate() {
            index[i] = k;
        }
        let local: Vec<_> = kept
            .iter()
            .map(|&i| {
                let mut c = cells[i].clone();
                c.neighbors = c.neighbors.iter().map(|&n| index[n]).collect();
                c
            })
            .collect();
        let keys = kept.iter().map(|&i| i as u32).collect();
        (Patch { cells: local, keys }, index[centre])
    })
}

fn wanted(t: &Template) -> BTreeSet<(i32, i32)> {
    t.ground.iter().map(|g| (g.c, g.r)).collect()
}

#[test]
fn the_kits_and_the_village_load_and_every_building_has_its_kit() {
    let kits = kits();
    kits.validate().expect("valid kits");
    let village = village();
    assert_eq!(village.buildings.len(), 12);
    for b in &village.buildings {
        assert!(kits.get(&b.kit).is_some(), "{}: no kit {}", b.name, b.kit);
    }
    let mut bad = kits.clone();
    bad.kits[0].walls[0].thickness_m = 0.0;
    assert!(bad.validate().unwrap_err().contains("straw"));
}

/// The patch's cells run counter-clockwise about their outward normal, and
/// each side's neighbour lies across that side's edge: what the chart and
/// the cutter assume.
#[test]
fn a_cells_sides_run_counter_clockwise_with_their_neighbours_across() {
    let (patch, at) = patch();
    let cell = &patch.cells[*at];
    for s in 0..6 {
        let a = cell.corners[s] - cell.direction;
        let b = cell.corners[(s + 1) % 6] - cell.direction;
        assert!(
            a.cross(b).dot(cell.direction) > 0.0,
            "side {s} turns clockwise"
        );
        let mid = (cell.corners[s] + cell.corners[(s + 1) % 6]) * 0.5 - cell.direction;
        let across = patch.cells[cell.neighbors[s]].direction - cell.direction;
        assert!(
            mid.normalize().dot(across.normalize()) > 0.95,
            "side {s}'s neighbour is across it"
        );
    }
}

/// Task 2.1: layout neighbours are sphere neighbours and no cell is used
/// twice, over the whole village grid.
#[test]
fn the_chart_keeps_neighbours_neighbours_and_uses_no_cell_twice() {
    let (patch, at) = patch();
    let village = village();
    let (north, east) = crate::geo::north_east(patch.cells[*at].direction);
    let _ = north;
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    assert_eq!(chart.cells.len(), village.ground.len());
    let used: BTreeSet<usize> = chart.cells.values().map(|c| c.cell).collect();
    assert_eq!(used.len(), chart.cells.len(), "no cell twice");
    for (&(c, r), at) in &chart.cells {
        for d in 0..6 {
            let n = neighbour(c, r, d);
            let Some(there) = chart.cell(n.0, n.1) else {
                continue;
            };
            let side = chart.side(c, r, d).unwrap();
            assert_eq!(
                patch.cells[at.cell].neighbors[side], there,
                "({c}, {r}) edge {d}"
            );
        }
    }
    // The rows run east: the anchor's direction-0 neighbour is east of it.
    let east_of = chart.cell(26, 17).unwrap();
    let step = patch.cells[east_of].direction - patch.cells[*at].direction;
    assert!(step.normalize().dot(east) > 0.8);
}

/// Every building of the village cuts, on the game's scale, into finite
/// triangles standing on its terrace and under its roof's height.
#[test]
fn every_village_building_cuts_into_its_pieces() {
    let (patch, at) = patch();
    let village = village();
    let kits = kits();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let mut meshes = Meshes::new();
    let repeat = |_: &str| 2.0;
    let terrace = 10.0;
    let mut solids = Vec::new();
    for b in &village.buildings {
        solids.push(
            cut_building(
                &mut meshes,
                &repeat,
                patch,
                &chart,
                b,
                kits.get(&b.kit).unwrap(),
                RADIUS_M,
                terrace,
            )
            .unwrap_or_else(|e| panic!("{e}")),
        );
    }
    // Slice 2a: a body in a house's wall is held; one in its doorway, one
    // over the house and one in the yard outside are not.
    let house = &village.buildings[0];
    let body = |p: Vec3| solids[0].holds(p, 0.9, 0.3);
    let [c, r, d, _] = house.doors[0];
    let side = chart.side(c, r, d as usize).unwrap();
    let cell = &patch.cells[chart.cell(c, r).unwrap()];
    let (a, b) = (cell.corners[side], cell.corners[(side + 1) % 6]);
    let at = |dir: Vec3, up: f32| dir.normalize() * (RADIUS_M + terrace + up);
    let doorway = (a + b) * 0.5;
    assert!(body(at(doorway, 0.95)), "the door is shut");
    for d in &mut solids[0].doors {
        d.open = true;
    }
    let body = |p: Vec3| solids[0].holds(p, 0.9, 0.3);
    assert!(!body(at(doorway, 0.95)), "the doorway is open");
    let wall = a * 0.9 + b * 0.1;
    assert!(body(at(wall, 0.95)), "the wall beside it is solid");
    assert!(!body(at(wall, 12.0)), "the air over the house is not");
    let yard = cell.direction + (doorway - cell.direction) * 3.0;
    assert!(!body(at(yard, 0.95)), "the yard outside is clear");
    // What a head meets rising: the upper floor's slab in a two-storey
    // room, the lintel in its doorway, nothing in the yard.
    assert!(house.storeys >= 2, "{} has an upper floor", house.name);
    let ceiling = |dir: Vec3| solids[0].ceiling(at(dir, 0.9), 0.3);
    let room = ceiling(cell.direction).expect("a ceiling in the room") - RADIUS_M - terrace;
    assert!(
        (room - (crate::settlement::STOREY_M - pieces::SLAB_M)).abs() < 0.02,
        "the room's ceiling at {room} m"
    );
    let lintel = ceiling(doorway).expect("a lintel over the door") - RADIUS_M - terrace;
    let door_h = kits.get(&house.kit).unwrap().door_m.1;
    assert!(
        (lintel - door_h).abs() < 0.02,
        "the lintel at {lintel} m, the door {door_h} m"
    );
    assert_eq!(ceiling(yard), None, "open sky over the yard");
    let triangles: usize = meshes.values().map(|m| m.positions.len() / 3).sum();
    assert!(triangles > 5_000, "{triangles} triangles");
    for want in [
        "fieldstone",
        "thatch",
        "shingle",
        "halftimber",
        "boards",
        "slate",
        "reed",
        "timber",
        "flag",
    ] {
        assert!(meshes.contains_key(want), "no {want}");
    }
    for (name, m) in &meshes {
        assert_eq!(m.positions.len(), m.normals.len());
        assert_eq!(m.positions.len(), m.uvs.len());
        // Counter-clockwise seen from outside, as the renderer culls: every
        // triangle's own turn points along its normal.
        for t in m.positions.chunks(3).zip(m.normals.chunks(3)) {
            let [a, b, c] = [0, 1, 2].map(|k| Vec3::from_array(t.0[k]));
            let turn = (b - a).cross(c - a);
            if turn.length() > 1e-6 {
                assert!(
                    turn.dot(Vec3::from_array(t.1[0])) > 0.0,
                    "{name}: a triangle faces in"
                );
            }
        }
        for p in &m.positions {
            let p = Vec3::from_array(*p);
            assert!(p.is_finite(), "{name}");
            let h = p.length() - RADIUS_M - terrace;
            assert!(
                (-0.3..14.0).contains(&h),
                "{name}: a point {h:.2} m over the terrace"
            );
        }
    }
}

/// The ground: the terrace in the footprint, the natural height held within
/// k layers of it at the margin's ring k, and nothing past the margin.
#[test]
fn the_ground_terraces_the_footprint_and_eases_the_margin() {
    let (patch, at) = patch();
    let footprint: Vec<(usize, Option<crate::terrain::Material>)> = std::iter::once(*at)
        .chain(patch.cells[*at].neighbors.iter().copied())
        .map(|i| (i, None))
        .collect();
    let slope = patch.cells[*at].direction;
    // A natural ground rising 1 m for every metre east of the anchor.
    let (_, east) = crate::geo::north_east(slope);
    let natural = move |d: Vec3| ((d - slope).dot(east) * RADIUS_M).floor() + 20.0;
    let ground = TownGround::new(patch, RADIUS_M, &footprint, 20.0, natural);
    let (inside, margin) = ground.counts();
    assert_eq!(inside, 7);
    assert!(margin > 0);
    let centre = ground.at(slope).expect("the anchor is on the ground");
    assert_eq!(centre.ring, 0);
    assert_eq!(
        centre.height(35.0),
        20.0,
        "the terrace, whatever the natural height"
    );
    for i in 0..patch.cells.len() {
        let d = patch.cells[i].direction;
        if let Some(g) = ground.at(d) {
            let h = g.height(natural(d));
            assert!(
                (h - 20.0).abs() <= f32::from(g.ring) + 1e-3,
                "ring {} at {h}",
                g.ring
            );
            assert_ne!(g.ring, GroundAt::OUTSIDE);
        }
    }
    let far = (slope + east * 0.3).normalize();
    assert!(ground.at(far).is_none(), "off the ground");
}

/// Each building's plan on the sphere is the mockup's flat one: as long
/// along its rows and as deep across them, within the cells' own spread
/// (CLAUDE.md: about 9% either way). The roof is laid over this plan in the
/// building's frame, so a frame turned off the rows puts the roof crossways
/// over it; that was the owner's "the roof shouldnt extend pass the floor
/// plan like that" (2026-09-29).
#[test]
fn every_plan_is_the_mockups_plan() {
    let (patch, at) = patch();
    let village = village();
    let kits = kits();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let (w, rr, row) = (
        2.833f32,
        2.833f32 / 3f32.sqrt(),
        1.5 * 2.833f32 / 3f32.sqrt(),
    );
    for b in &village.buildings {
        let mut meshes = Meshes::new();
        let solids = cut_building(
            &mut meshes,
            &|_| 2.0,
            patch,
            &chart,
            b,
            kits.get(&b.kit).unwrap(),
            RADIUS_M,
            10.0,
        )
        .unwrap();
        let (mut lo, mut hi) = (glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MIN));
        let (mut mlo, mut mhi) = (glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MIN));
        for &[c, r] in &b.cells {
            let cell = &patch.cells[chart.cell(c, r).unwrap()];
            for d in 0..6 {
                let s = chart.side(c, r, d).unwrap();
                let p = solids.frame.plane(cell.corners[(s + 1) % 6]);
                lo = lo.min(p);
                hi = hi.max(p);
                let a = ((60 * d) as f32 - 30.0).to_radians();
                let cx = w * (c as f32 + 0.5 * (r.rem_euclid(2)) as f32);
                let q = glam::Vec2::new(cx + rr * a.cos(), row * r as f32 + rr * a.sin());
                mlo = mlo.min(q);
                mhi = mhi.max(q);
            }
        }
        let (game, mockup) = (hi - lo, mhi - mlo);
        let off = (game / mockup - glam::Vec2::ONE).abs();
        assert!(
            off.max_element() < 0.12,
            "{}: {:.2} x {:.2} m on the sphere, {:.2} x {:.2} in the mockup",
            b.name,
            game.x,
            game.y,
            mockup.x,
            mockup.y
        );
    }
}

/// Two convex outlines overlap by more than `slack`: no axis of either
/// separates them.
fn overlap(a: &[glam::Vec2], b: &[glam::Vec2], slack: f32) -> bool {
    for poly in [a, b] {
        for i in 0..poly.len() {
            let e = poly[(i + 1) % poly.len()] - poly[i];
            let n = glam::Vec2::new(-e.y, e.x).normalize();
            let span = |p: &[glam::Vec2]| {
                p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
                    (lo.min(q.dot(n)), hi.max(q.dot(n)))
                })
            };
            let (a0, a1) = span(a);
            let (b0, b1) = span(b);
            if a1 <= b0 + slack || b1 <= a0 + slack {
                return false;
            }
        }
    }
    true
}

/// "roof should not intersect like that" (the owner, 2026-09-29), and the
/// mockup's own rule (`tenebris-towns` section 2): every roof's plan, eaves
/// included, is clear of every other roof's, laid out on the sphere.
#[test]
fn no_two_roofs_cut_into_each_other() {
    let (patch, at) = patch();
    let village = village();
    let kits = kits();
    let (north, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let anchor = patch.cells[*at].direction;
    let plans: Vec<(String, Vec<glam::Vec2>)> = village
        .buildings
        .iter()
        .map(|b| {
            let mut meshes = Meshes::new();
            let s = cut_building(
                &mut meshes,
                &|_| 2.0,
                patch,
                &chart,
                b,
                kits.get(&b.kit).unwrap(),
                RADIUS_M,
                10.0,
            )
            .unwrap();
            let plan = s
                .roof_plan
                .iter()
                .map(|p| {
                    let w = s.frame.world(Vec3::new(p.x, 0.0, p.y)) - anchor * RADIUS_M;
                    glam::Vec2::new(w.dot(east), w.dot(north))
                })
                .collect();
            (b.name.clone(), plan)
        })
        .collect();
    for i in 0..plans.len() {
        for j in i + 1..plans.len() {
            assert!(
                !overlap(&plans[i].1, &plans[j].1, 0.01),
                "{} and {} cut into each other",
                plans[i].0,
                plans[j].0
            );
        }
    }
}

/// The village laid at the test patch's anchor, as a world would lay it,
/// over a gently sloping natural ground.
fn laid_village(template: &Template) -> record::Town {
    let (patch, at) = patch();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    record::lay(template, 7, patch, *at, d0, slope()).unwrap_or_else(|e| panic!("{e}"))
}

/// A natural ground rising a metre every 20 east of the anchor.
fn slope() -> impl Fn(Vec3) -> f32 {
    let (patch, at) = patch();
    let centre = patch.cells[*at].direction;
    let (_, east) = crate::geo::north_east(centre);
    move |d: Vec3| (d - centre).dot(east) * RADIUS_M / 20.0 + 12.4
}

fn built(town: &record::Town) -> record::Built {
    let (patch, _) = patch();
    let natural = slope();
    record::build(town, patch, &kits(), &|_: &str| 2.0, RADIUS_M, move |d| {
        natural(d).floor()
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// Slice 3a: a town goes into its records and comes back whole.
#[test]
fn a_town_round_trips_through_its_records() {
    let town = laid_village(&village());
    assert_eq!(town.buildings.len(), village().buildings.len());
    assert!(
        town.buildings
            .iter()
            .all(|b| b.state == record::BuildingState::Standing)
    );
    let records = record::to_records(&town);
    assert_eq!(
        records.last().map(|r| r.kind.as_str()),
        Some(record::SETTLEMENT_RECORD),
        "the settlement is written last"
    );
    let mut store = crate::records::Records::new();
    for r in records {
        store.put(r);
    }
    assert_eq!(record::from_records(&store, 7), record::Stored::Town(town));
    assert_eq!(record::from_records(&store, 8), record::Stored::None);
}

/// A town built from its definition is the town the template lays: the
/// same pieces, solids and ground as cutting the template's own buildings
/// on the same chart and terrace.
#[test]
fn a_town_built_from_its_record_is_the_town_its_template_lays() {
    let (patch, at) = patch();
    let template = village();
    let town = laid_village(&template);
    let from_record = built(&town);
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let charted = chart(
        patch,
        record::template_anchor(&template),
        *at,
        d0,
        &wanted(&template),
    )
    .unwrap();
    for (c, r) in town.cells.iter().map(|x| (x.0, x.1)) {
        assert_eq!(from_record.chart.cells[&(c, r)], charted.cells[&(c, r)]);
    }
    let mut levels: Vec<i32> = template
        .ground
        .iter()
        .filter(|g| template.built_cells().contains(&[g.c, g.r]))
        .map(|g| g.h)
        .collect();
    levels.sort();
    let datum = levels[levels.len() / 2];
    let kits = kits();
    let mut meshes = Meshes::new();
    for b in &template.buildings {
        cut_building(
            &mut meshes,
            &|_: &str| 2.0,
            patch,
            &charted,
            b,
            kits.get(&b.kit).unwrap(),
            RADIUS_M,
            town.terrace as f32 + (b.base - datum) as f32,
        )
        .unwrap();
    }
    assert!(meshes == from_record.meshes, "the same pieces");
    // The ground as slice 1 laid it, straight from the template: what a
    // world opened by the merged build stood on.
    let built_cells: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let mut footprint = built_cells.clone();
    let mut ring = built_cells;
    for _ in 0..record::YARD_RINGS {
        let mut next = BTreeSet::new();
        for &(c, r) in &ring {
            for d in 0..6 {
                let n = neighbour(c, r, d);
                if charted.cells.contains_key(&n) && footprint.insert(n) {
                    next.insert(n);
                }
            }
        }
        ring = next;
    }
    let lanes: BTreeSet<(i32, i32)> = template
        .ground
        .iter()
        .filter(|g| g.top != "grass" && g.top != "sand")
        .map(|g| (g.c, g.r))
        .collect();
    let cells: Vec<(usize, Option<crate::terrain::Material>)> = footprint
        .iter()
        .map(|&(c, r)| {
            let top = lanes
                .contains(&(c, r))
                .then_some(crate::terrain::Material::Dirt);
            (charted.cell(c, r).unwrap(), top)
        })
        .collect();
    let floored = |d: Vec3| slope()(d).floor();
    let before = TownGround::new(patch, RADIUS_M, &cells, town.terrace as f32, floored);
    let config = crate::planet_gen::TerrainConfig::default();
    assert_eq!(
        ground::Ground::new(config, vec![before]).digest(),
        ground::Ground::new(config, vec![from_record.ground.clone()]).digest(),
        "the same ground"
    );
    assert_eq!(from_record.solids.len(), template.buildings.len());
    let (footprint, margin) = from_record.ground.counts();
    assert_eq!(footprint, town.cells.len());
    assert!(margin > 0, "the slope is eased over a margin");
}

/// Decision 8: a made town is its records, and a revised template changes
/// only the towns laid after it.
#[test]
fn a_revised_template_leaves_a_made_town_as_it_was() {
    let template = village();
    let made = laid_village(&template);
    let mut store = crate::records::Records::new();
    for r in record::to_records(&made) {
        store.put(r);
    }
    let mut revised = template.clone();
    revised.buildings.remove(1);
    revised.buildings[0].doors[0][2] = (revised.buildings[0].doors[0][2] + 3) % 6;
    revised.buildings[0].storeys += 1;
    let fresh = laid_village(&revised);
    assert_ne!(fresh, made, "the revision is a different town");
    let record::Stored::Town(kept) = record::from_records(&store, 7) else {
        panic!("the made town reads");
    };
    assert_eq!(kept, made);
    assert!(built(&kept).meshes == built(&made).meshes);
}

/// A settlement that is there but cannot be read whole is named, never
/// taken for no town (which would lay a new one over it).
#[test]
fn a_damaged_settlement_record_is_named_not_remade() {
    let town = laid_village(&village());
    let records = record::to_records(&town);
    let put = |skip: Option<usize>, change: &dyn Fn(&mut crate::records::Record)| {
        let mut store = crate::records::Records::new();
        for (i, r) in records.iter().enumerate() {
            if Some(i) == skip {
                continue;
            }
            let mut r = r.clone();
            change(&mut r);
            store.put(r);
        }
        record::from_records(&store, 7)
    };
    let damaged = |s: record::Stored| matches!(s, record::Stored::Damaged(_));
    assert!(damaged(put(Some(0), &|_| {})), "a building missing");
    assert!(damaged(put(None, &|r| r.schema = 9)), "an unknown schema");
    assert!(
        damaged(put(None, &|r| if r.kind == record::SETTLEMENT_RECORD {
            r.schema = record::TERRACED_SCHEMA
        })),
        "a terraced schema with no levels"
    );
    assert!(
        damaged(put(None, &|r| if r.kind == record::SETTLEMENT_RECORD {
            r.body = "(nothing)".into()
        })),
        "a body that does not read"
    );
}

/// What a town costs its save: measured, and held under a bound.
#[test]
fn a_towns_records_are_small() {
    let town = laid_village(&village());
    let bytes: Vec<(String, usize)> = record::to_records(&town)
        .iter()
        .map(|r| (r.kind.clone(), r.body.len()))
        .collect();
    let settlement: usize = bytes
        .iter()
        .filter(|b| b.0 == "settlement")
        .map(|b| b.1)
        .sum();
    let buildings: usize = bytes
        .iter()
        .filter(|b| b.0 == "building")
        .map(|b| b.1)
        .sum();
    println!(
        "{} cells in {settlement} bytes; {} buildings in {buildings} bytes",
        town.cells.len(),
        town.buildings.len()
    );
    assert!(
        settlement + buildings < 32_000,
        "{settlement} + {buildings} bytes"
    );
}

/// Every kit a saved town can name, by the build that first stored it. A
/// stored building looks its kit up by name each time it is cut, so a kit
/// once named here stays in `kits.ron` for good, as a generator version
/// does (slice 3a). A new template's kits are added here when it ships.
const KITS_SAVED_TOWNS_NAME: &[&str] = &[
    "ashlar",
    "halftimber",
    "stone",
    "straw",
    "timber",
    // The walled town (slice 4b).
    "brick",
    "brickbuff",
    "brickdark",
    "clay",
    "marble",
    "mud",
];

#[test]
fn every_kit_a_saved_town_can_name_is_shipped() {
    let kits = kits();
    for name in KITS_SAVED_TOWNS_NAME {
        assert!(kits.get(name).is_some(), "kits.ron dropped {name}");
    }
    for b in village().buildings.iter().chain(&walled().buildings) {
        assert!(
            KITS_SAVED_TOWNS_NAME.contains(&b.kit.as_str()),
            "{} names {}, which is not listed as a kit a saved town can name",
            b.name,
            b.kit
        );
    }
}

/// Each of the village's buildings cut on the test patch, with its solids.
fn cut_village() -> (Template, Vec<pieces::BuildingSolids>) {
    let (patch, at) = patch();
    let village = village();
    let kits = kits();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let mut meshes = Meshes::new();
    let solids = village
        .buildings
        .iter()
        .map(|b| {
            cut_building(
                &mut meshes,
                &|_: &str| 2.0,
                patch,
                &chart,
                b,
                kits.get(&b.kit).unwrap(),
                RADIUS_M,
                10.0,
            )
            .unwrap_or_else(|e| panic!("{e}"))
        })
        .collect();
    (village, solids)
}

/// The top of every surface at a point of a building's plan.
fn tops(b: &pieces::BuildingSolids, p: glam::Vec2) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    for s in &b.surfaces {
        s.intervals(p, &mut out);
    }
    out
}

/// Slice 2b: one stair cell is a newel, two a straight flight; no floor
/// over a newel, and a flight's cells floored only either side of it.
#[test]
fn every_stair_is_cut_and_nothing_floors_its_well() {
    use pieces::Surface;
    let (village, solids) = cut_village();
    for (def, b) in village.buildings.iter().zip(&solids) {
        let newels = b
            .surfaces
            .iter()
            .filter(|s| matches!(s, Surface::Newel { .. }))
            .count();
        let flights = b
            .surfaces
            .iter()
            .filter(|s| matches!(s, Surface::Flight { .. }))
            .count();
        let floors = b
            .surfaces
            .iter()
            .filter(|s| matches!(s, Surface::Floor { .. }))
            .count();
        match def.stair_cells.len() {
            0 => assert_eq!((newels, flights, floors), (0, 0, 0), "{}", def.name),
            1 => {
                assert_eq!((newels, flights), (1, 0), "{}", def.name);
                // Every other cell floored upstairs, and not the newel's.
                assert_eq!(floors, def.cells.len() - 1, "{}", def.name);
                let Some(Surface::Newel { centre, .. }) = b
                    .surfaces
                    .iter()
                    .find(|s| matches!(s, Surface::Newel { .. }))
                else {
                    unreachable!()
                };
                let over = tops(b, *centre + glam::Vec2::new(0.6, 0.0));
                assert!(
                    over.iter().all(|t| t.1 <= 3.0 + 1e-3),
                    "{}: nothing over the newel but its own sheets: {over:?}",
                    def.name
                );
            }
            2 => {
                assert_eq!((newels, flights), (0, 1), "{}", def.name);
                // Two side parts in each of the flight's two cells.
                assert_eq!(floors, def.cells.len() - 2 + 4, "{}", def.name);
                let Some(Surface::Flight { foot, dir, len, .. }) = b
                    .surfaces
                    .iter()
                    .find(|s| matches!(s, Surface::Flight { .. }))
                else {
                    unreachable!()
                };
                let mid = *foot + *dir * (len / 2.0);
                let over = tops(b, mid);
                assert_eq!(
                    over.len(),
                    1,
                    "{}: only the flight over its strip",
                    def.name
                );
            }
            n => panic!("{}: {n} stair cells", def.name),
        }
    }
}

/// A straight flight answers its pitch line: from its foot at the floor to
/// its landing at the storey, rising evenly, then flat.
#[test]
fn a_flight_answers_its_pitch_line() {
    use pieces::Surface;
    let (_, solids) = cut_village();
    let b = solids
        .iter()
        .find(|b| {
            b.surfaces
                .iter()
                .any(|s| matches!(s, Surface::Flight { .. }))
        })
        .expect("a house with a flight");
    let Some(Surface::Flight {
        foot,
        dir,
        len,
        risers,
        ..
    }) = b
        .surfaces
        .iter()
        .find(|s| matches!(s, Surface::Flight { .. }))
    else {
        unreachable!()
    };
    let run = len / *risers as f32;
    let landing = (*risers - 1) as f32 * run;
    // The flight's own sheet: at its foot it meets the floor over the cell
    // before it, a storey up, which a walker at the foot does not reach.
    let flight = b
        .surfaces
        .iter()
        .find(|s| matches!(s, Surface::Flight { .. }))
        .unwrap();
    let top_at = |u: f32| {
        let mut out = Vec::new();
        flight.intervals(*foot + *dir * u, &mut out);
        out.iter().map(|t| t.1).fold(f32::MIN, f32::max)
    };
    assert!(top_at(0.01).abs() < 0.01, "the foot at the floor");
    assert!(
        (top_at(landing) - STOREY_M).abs() < 1e-3,
        "the landing at the storey"
    );
    assert!(
        (top_at(len - 0.01) - STOREY_M).abs() < 1e-3,
        "flat across the landing"
    );
    let mut last = top_at(0.0);
    let mut u = 0.05;
    while u < *len {
        let t = top_at(u);
        assert!(
            t >= last - 1e-4 && t - last < 0.05,
            "at {u} m: {last} to {t}"
        );
        last = t;
        u += 0.05;
    }
}

/// A newel answers one sheet a turn on its pitch line, 3 m a turn, with
/// 2.74 m clear under the turn above; nothing inside its post.
#[test]
fn a_newel_answers_a_sheet_a_turn() {
    use pieces::Surface;
    let (_, solids) = cut_village();
    let b = solids
        .iter()
        .find(|b| {
            b.surfaces
                .iter()
                .any(|s| matches!(s, Surface::Newel { .. }))
        })
        .expect("a house with a newel");
    let Some(Surface::Newel {
        centre,
        start,
        sense,
        top,
        ..
    }) = b
        .surfaces
        .iter()
        .find(|s| matches!(s, Surface::Newel { .. }))
    else {
        unreachable!()
    };
    assert!(tops(b, *centre).is_empty(), "the post");
    let at = |phi: f32| {
        let a = start + sense * phi;
        *centre + glam::Vec2::new(a.cos(), a.sin()) * 0.72
    };
    let tau = std::f32::consts::TAU;
    let mut last = 0.0f32;
    for k in 1..=60 {
        let phi = k as f32 / 60.0 * tau * 0.97;
        let here = tops(b, at(phi));
        let sheet = here
            .iter()
            .map(|t| t.1)
            .filter(|t| (t - last).abs() < 0.5)
            .fold(f32::MIN, f32::max);
        let want = phi / tau * STOREY_M;
        assert!(
            (sheet - want).abs() < 1e-3,
            "at {phi:.2} rad: {sheet} for {want}"
        );
        assert!(sheet - last < 0.2, "a jump at {phi:.2}");
        last = sheet;
    }
    // The village's newels climb one turn, so over the foot is the flat
    // landing: 30 degrees at the storey, which costs the turn below some
    // headroom (`tenebris-towns` section 3), never down to the body's 1.8 m.
    assert!(*top >= STOREY_M, "a storey at least");
    let mut k = 0.0;
    while k < 0.55 {
        let here = tops(b, at(k));
        let lowest = here.iter().map(|t| t.1).fold(f32::MAX, f32::min);
        if let Some(above) = here
            .iter()
            .filter(|t| t.1 > lowest + 1.0)
            .map(|t| t.0)
            .reduce(f32::min)
        {
            let clear = above - lowest;
            assert!(
                clear > 2.4 && clear <= STOREY_M - pieces::TREAD_M + 1e-3,
                "at {k}: {lowest} to {above}"
            );
        }
        k += 0.05;
    }
}

/// Door leaves: one per door, shut by default, a solid across its doorway
/// when shut and none when open.
#[test]
fn a_shut_door_holds_and_an_open_one_does_not() {
    let (village, mut solids) = cut_village();
    for (def, b) in village.buildings.iter().zip(&solids) {
        assert_eq!(b.doors.len(), def.doors.len(), "{}", def.name);
        assert!(b.doors.iter().all(|d| !d.open));
    }
    let b = &mut solids[0];
    let d = b.doors[0].clone();
    let at = b.frame.world(Vec3::new(d.middle.x, 0.95, d.middle.y));
    assert!(b.holds(at, 0.9, 0.3), "shut");
    assert!(b.push_normal(at, 0.9, 0.3).is_some(), "and a way out of it");
    b.doors[0].open = true;
    assert!(!b.holds(at, 0.9, 0.3), "open");
    assert!(
        !b.doors[0].mesh(b.frame, 1.5, &|_: &str| 2.0).is_empty(),
        "drawn"
    );
}

/// Where a ray from `from` along `dir` first meets a triangle, and which of
/// the two sets it is in: the rooms' (true) or the outside's.
fn first_hit(from: Vec3, dir: Vec3, tris: &[([Vec3; 3], bool)]) -> Option<(f32, bool, usize)> {
    let mut best: Option<(f32, bool, usize)> = None;
    for (k, (t, inside)) in tris.iter().enumerate() {
        let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
        let p = dir.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-9 {
            continue;
        }
        let s = from - t[0];
        let u = s.dot(p) / det;
        let q = s.cross(e1);
        let v = dir.dot(q) / det;
        let d = e2.dot(q) / det;
        if u < 0.0 || v < 0.0 || u + v > 1.0 || d <= 1e-4 {
            continue;
        }
        if best.is_none_or(|(b, _, _)| d < b) {
            best = Some((d, *inside, k));
        }
    }
    best
}

/// `sun-shadows` decision 7: the cutter tells a room's faces from the
/// town's outside. From the middle of every ground-floor room, whatever a
/// ray meets inside the building's plan is a room face (its walls, floor,
/// ceiling, beams and stair); from the yard and from above, whatever a ray
/// meets outside the plan is not (its outer walls, eaves and roof). A ray
/// through a door or a window is let through: what it meets on the far side
/// is the other side's.
#[test]
fn a_rooms_faces_are_what_is_seen_from_inside_it() {
    let (patch, at) = patch();
    let village = village();
    let kits = kits();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let rays: Vec<Vec3> = (0..400)
        .map(|i| {
            // A Fibonacci sphere.
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / 400.0;
            let a = i as f32 * 2.399_963;
            let r = (1.0 - y * y).sqrt();
            Vec3::new(r * a.cos(), y, r * a.sin())
        })
        .collect();
    let mut seen = [0usize; 2];
    for b in &village.buildings {
        let kit = kits.get(&b.kit).unwrap();
        let mut meshes = Meshes::new();
        let cut = cut_building(
            &mut meshes,
            &|_: &str| 2.0,
            patch,
            &chart,
            b,
            kit,
            RADIUS_M,
            10.0,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert!(!cut.rooms.is_empty(), "{} has no rooms", b.name);
        let f = cut.frame;
        let mut tris: Vec<([Vec3; 3], bool)> = Vec::new();
        for (set, inside) in [(&meshes, false), (&cut.rooms, true)] {
            for m in set.values() {
                for t in m.positions.chunks(3) {
                    tris.push((
                        t.iter()
                            .map(|p| f.local(Vec3::from_array(*p)))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        inside,
                    ));
                }
            }
        }
        let hexes: Vec<Vec<glam::Vec2>> = b
            .cells
            .iter()
            .map(|&[c, r]| {
                let cell = &patch.cells[chart.cell(c, r).unwrap()];
                cell.corners.iter().map(|p| f.plane(*p)).collect()
            })
            .collect();
        let in_plan = |p: Vec3| {
            let q = glam::Vec2::new(p.x, p.z);
            hexes.iter().any(|h| {
                let n = h.len();
                let turn = (h[1] - h[0]).perp_dot(h[2] - h[0]).signum();
                (0..n).all(|k| turn * (h[(k + 1) % n] - h[k]).perp_dot(q - h[k]) >= -1e-3)
            })
        };
        let storey = if kit.hut {
            2.6
        } else {
            crate::settlement::STOREY_M
        };
        let top = b.storeys.max(1) as f32 * storey * b.tall.max(1) as f32;
        // Under a ceiling, nothing over it is a room's.
        let open_roof = b.roof == "cone" && b.cells.len() == 1;
        if !open_roof {
            for (t, inside) in &tris {
                assert!(
                    !inside || t.iter().all(|v| v.y <= top + 0.02),
                    "{}: a room face over the ceiling: {t:?}",
                    b.name
                );
            }
        }
        let centre_of = |c: i32, r: i32| {
            let q = f.plane(patch.cells[chart.cell(c, r).unwrap()].direction);
            Vec3::new(q.x, 0.0, q.y)
        };
        // Inside: the middle of each room cell that is not the stair's.
        for &[c, r] in b.cells.iter().filter(|x| !b.stair_cells.contains(x)) {
            let eye = centre_of(c, r) + Vec3::Y * 1.5;
            for d in &rays {
                if let Some((t, inside, k)) = first_hit(eye, *d, &tris) {
                    let p = eye + *d * t;
                    // A reveal or a jamb stands across the wall line.
                    let across = tris[k].0.iter().any(|v| in_plan(*v))
                        && tris[k].0.iter().any(|v| !in_plan(*v));
                    if in_plan(p) && p.y < top - 0.05 && !across {
                        assert!(
                            inside,
                            "{}: from its room at {eye}, an outside face at {p}: {:?}",
                            b.name, tris[k]
                        );
                        seen[1] += 1;
                    }
                }
            }
        }
        // Outside: from the yard, eight metres off every cell, and from
        // above the ridge.
        let middle = hexes.iter().flatten().fold(glam::Vec2::ZERO, |s, p| s + *p)
            / hexes.iter().map(Vec::len).sum::<usize>() as f32;
        let mut eyes: Vec<Vec3> = b
            .cells
            .iter()
            .map(|&[c, r]| {
                let q = centre_of(c, r);
                let away = (glam::Vec2::new(q.x, q.z) - middle).normalize_or(glam::Vec2::X);
                Vec3::new(q.x + away.x * 8.0, 1.5, q.z + away.y * 8.0)
            })
            .collect();
        eyes.push(Vec3::new(middle.x, top + 20.0, middle.y));
        for eye in eyes {
            for d in &rays {
                if let Some((t, inside, k)) = first_hit(eye, *d, &tris) {
                    let p = eye + *d * t;
                    let across = tris[k].0.iter().any(|v| in_plan(*v))
                        && tris[k].0.iter().any(|v| !in_plan(*v));
                    if !in_plan(p) && !across {
                        assert!(
                            !inside,
                            "{}: from outside at {eye}, a room face at {p}: {:?}",
                            b.name, tris[k]
                        );
                        seen[0] += 1;
                    }
                }
            }
        }
    }
    assert!(
        seen[0] > 1000 && seen[1] > 1000,
        "rays that met faces: {seen:?}"
    );
}

/// `cities-in-the-world` decision 7a: every village house with a chimney has
/// its hearth by it, every stair its sconces, and about half the windows a
/// candle; every light burns inside its building, over its floor and under
/// its roof, and burns as its kind does.
#[test]
fn every_house_has_its_hearth_its_sconces_and_its_candles() {
    use pieces::LightKind;
    let (patch, at) = patch();
    let (village, solids) = cut_village();
    let kits = kits();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    let (mut windows, mut candles) = (0usize, 0usize);
    for (b, cut) in village.buildings.iter().zip(&solids) {
        let kit = kits.get(&b.kit).unwrap();
        let count = |kind| cut.lights.iter().filter(|l| l.kind == kind).count();
        let hearth = b
            .chimney
            .filter(|c| b.cells.contains(c) && !b.stair_cells.contains(c));
        assert_eq!(
            count(LightKind::Hearth),
            usize::from(hearth.is_some()),
            "{}: a hearth under its chimney",
            b.name
        );
        let sconces = match b.stair_cells.len() {
            2 => 1,
            1 => b.storeys.max(1) as usize - 1,
            _ => 0,
        };
        assert_eq!(
            count(LightKind::Sconce),
            sconces,
            "{}: its stair's sconces",
            b.name
        );
        windows += b.windows.len();
        candles += count(LightKind::Candle);
        let f = cut.frame;
        let hexes: Vec<Vec<glam::Vec2>> = b
            .cells
            .iter()
            .map(|&[c, r]| {
                let cell = &patch.cells[chart.cell(c, r).unwrap()];
                cell.corners.iter().map(|p| f.plane(*p)).collect()
            })
            .collect();
        let storey = if kit.hut {
            2.0
        } else {
            crate::settlement::STOREY_M
        };
        let top = b.storeys.max(1) as f32 * storey * b.tall.max(1) as f32;
        for l in &cut.lights {
            let p = f.local(l.at);
            let q = glam::Vec2::new(p.x, p.z);
            let over = hexes.iter().any(|h| {
                let turn = (h[1] - h[0]).perp_dot(h[2] - h[0]).signum();
                (0..h.len()).all(|k| turn * (h[(k + 1) % h.len()] - h[k]).perp_dot(q - h[k]) >= 0.0)
            });
            assert!(
                over && p.y > 0.0 && p.y < top,
                "{}: a {:?} outside at {p}",
                b.name,
                l.kind
            );
            assert!(l.below_m > 0.0 && l.above_m > 0.0 && l.power > 0.0);
            assert_eq!(l.kind.all_day(), l.kind != LightKind::Candle);
        }
    }
    let share = candles as f32 / windows as f32;
    assert!(
        (0.35..0.75).contains(&share),
        "{candles} candles behind {windows} windows"
    );
    // The mockup's colours, in linear light: warm, red over blue.
    for kind in [LightKind::Hearth, LightKind::Sconce, LightKind::Candle] {
        let [r, g, b] = kind.colour();
        assert!(
            r == 1.0 && r > g && g > b && b > 0.0,
            "{kind:?} {r} {g} {b}"
        );
    }
    assert!((pieces::srgb_linear(0xff9a4a)[1] - 0.3231).abs() < 1e-3);
}

/// A house's roof keeps the rain off whoever is under it (the owner,
/// 2026-09-30, on drops on the lens indoors): a room of every building is
/// sheltered, and the yard beside it and the air over its roof are not.
#[test]
fn a_roof_shelters_its_rooms_and_not_the_yard() {
    let (patch, at) = patch();
    let (village, solids) = cut_village();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let chart = chart(patch, (25, 17), *at, d0, &wanted(&village)).expect("charted");
    for (b, cut) in village.buildings.iter().zip(&solids) {
        let f = cut.frame;
        let [c, r] = b.cells[0];
        let q = f.plane(patch.cells[chart.cell(c, r).unwrap()].direction);
        let room = f.world(Vec3::new(q.x, 1.6, q.y));
        assert!(cut.shelters(room), "{}: its room", b.name);
        let over = f.world(Vec3::new(q.x, cut.top_m + 6.0, q.y));
        assert!(!cut.shelters(over), "{}: over its roof", b.name);
        let yard = f.world(Vec3::new(q.x, 1.6, q.y) + Vec3::new(40.0, 0.0, 0.0));
        assert!(!cut.shelters(yard), "{}: the yard", b.name);
    }
}

/// Slice 4a: a village turns by its site, to any of six sides, and lays and
/// cuts whole at every one of them.
#[test]
fn the_village_lays_and_cuts_at_each_of_its_six_turns() {
    let (patch, at) = patch();
    let template = village();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let sides = patch.cells[*at].corners.len();
    let base = patch.side_toward(*at, east);
    let mut charts = BTreeSet::new();
    let mut counts = BTreeSet::new();
    for t in 0..6 {
        let d0 = (base + t) % sides;
        let town = record::lay(&template, 7, patch, *at, d0, slope())
            .unwrap_or_else(|e| panic!("turn {t}: {e}"));
        let b = built(&town);
        assert_eq!(b.solids.len(), template.buildings.len(), "turn {t}");
        counts.insert(town.cells.len());
        let mut keys: Vec<u32> = town.cells.iter().map(|c| c.2).collect();
        keys.sort_unstable();
        charts.insert(keys);
    }
    assert_eq!(counts.len(), 1, "every turn lays the same footprint");
    assert_eq!(charts.len(), 6, "each turn stands on its own cells");
}

/// The turn is the site's, and the six are dealt about evenly.
#[test]
fn a_sites_turn_is_its_own_and_the_six_are_dealt_evenly() {
    let mut seen = [0u32; 6];
    for site in 0..6000 {
        assert_eq!(record::turn(site), record::turn(site));
        seen[record::turn(site)] += 1;
    }
    assert!(
        seen.iter().all(|&n| (900..1100).contains(&n)),
        "turns dealt {seen:?}"
    );
}

/// Task 4.4: a player's edit in a town's footprint or margin is found, and
/// one outside it is not.
#[test]
fn an_edit_in_a_towns_ground_is_found_and_one_outside_is_not() {
    let town = laid_village(&village());
    let ground = built(&town).ground;
    let edit = |cell: u32| {
        let mut edits = crate::edits::Edits::new();
        edits.set(crate::edits::Edit {
            cell,
            layer: 60,
            material: crate::terrain::Material::Air,
        });
        edits
    };
    assert!(!ground.touches(&crate::edits::Edits::new()));
    assert!(ground.touches(&edit(town.cells[0].2)), "in the footprint");
    let (patch, _) = patch();
    let inside: BTreeSet<u32> = town.cells.iter().map(|c| c.2).collect();
    let anchor = ground.anchor();
    let mut by_distance: Vec<usize> = (0..patch.cells.len()).collect();
    by_distance.sort_by(|&a, &b| {
        patch.cells[b]
            .direction
            .dot(anchor)
            .total_cmp(&patch.cells[a].direction.dot(anchor))
    });
    let margin = by_distance
        .iter()
        .map(|&i| patch.keys[i])
        .find(|k| !inside.contains(k) && ground.touches(&edit(*k)))
        .expect("a margin cell");
    assert!(!inside.contains(&margin));
    let far = patch.keys[*by_distance.last().unwrap()];
    assert!(!ground.touches(&edit(far)), "far outside");
}

/// An unsettled site reads as unsettled, never as a town not yet laid, so
/// no later build lays one over the player's work.
#[test]
fn an_unsettled_site_stays_unsettled() {
    let mut store = crate::records::Records::new();
    store.put(record::unsettled_record(9, "dug"));
    assert_eq!(record::from_records(&store, 9), record::Stored::Unsettled);
    assert_eq!(record::from_records(&store, 10), record::Stored::None);
}

/// Slice 4a: the world's ground finds a town through the cube's grid, and
/// finds exactly what a look at every town finds, in the town, round its
/// edge and past it.
#[test]
fn the_grid_finds_what_every_town_would() {
    let town = laid_village(&village());
    let one = built(&town).ground;
    let ground = ground::Ground::new(
        crate::planet_gen::TerrainConfig::default(),
        vec![one.clone()],
    );
    let anchor = one.anchor();
    let (north, east) = crate::geo::north_east(anchor);
    let mut checked = 0;
    for ring in 0..60 {
        let rho = ring as f32 * 0.012;
        for k in 0..90 {
            let a = k as f32 * std::f32::consts::TAU / 90.0;
            let d =
                (anchor * rho.cos() + (east * a.cos() + north * a.sin()) * rho.sin()).normalize();
            assert_eq!(ground.at(d), one.at(d), "at {rho:.3} rad, {a:.2}");
            checked += usize::from(one.at(d).is_some());
        }
    }
    assert!(checked > 1000, "{checked} directions on the town's ground");
}

/// Slice 4b: the walled town lays on its levels. Every built cell stands at
/// its own height over the datum, a yard at the level of the built cell
/// nearest it, every building on the level of its cells, and the ground
/// the town is built on answers each cell's terrace.
#[test]
fn the_walled_town_lays_on_its_levels() {
    let template = walled();
    assert!(template.terraced);
    let kits = kits();
    for b in &template.buildings {
        assert!(kits.get(&b.kit).is_some(), "{}: no kit {}", b.name, b.kit);
    }
    let town = laid_village(&template);
    assert_eq!(town.levels.len(), town.cells.len());
    let spread: BTreeSet<i8> = town.levels.iter().copied().collect();
    assert!(spread.len() >= 3, "levels {spread:?}");
    let height: std::collections::BTreeMap<(i32, i32), i32> =
        template.ground.iter().map(|g| ((g.c, g.r), g.h)).collect();
    let on: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let level: std::collections::BTreeMap<(i32, i32), i8> = town
        .cells
        .iter()
        .zip(&town.levels)
        .map(|(c, &l)| ((c.0, c.1), l))
        .collect();
    // The datum is where most of what is built stands: a built cell's
    // height less its level.
    let first = town
        .cells
        .iter()
        .position(|c| on.contains(&(c.0, c.1)))
        .expect("a built cell");
    let datum = height[&(town.cells[first].0, town.cells[first].1)] - i32::from(town.levels[first]);
    for (&at, &l) in &level {
        if on.contains(&at) {
            assert_eq!(i32::from(l), height[&at] - datum, "built cell {at:?}");
        }
    }
    let (lo, hi) = (
        on.iter().filter_map(|a| level.get(a)).min().unwrap(),
        on.iter().filter_map(|a| level.get(a)).max().unwrap(),
    );
    assert!(
        town.levels.iter().all(|l| l >= lo && l <= hi),
        "no yard below the lowest built cell or above the highest"
    );
    for b in &town.buildings {
        for &[c, r] in &b.cells {
            assert_eq!(
                i32::from(level[&(c, r)]),
                b.floor,
                "{} at ({c}, {r})",
                b.name
            );
        }
    }
    let b = built(&town);
    assert_eq!(b.solids.len(), template.buildings.len());
    let (patch, _) = patch();
    for (i, cell) in town.cells.iter().enumerate() {
        let at = b.chart.cells[&(cell.0, cell.1)].cell;
        let g = b
            .ground
            .at(patch.cells[at].direction)
            .expect("on its ground");
        assert_eq!(g.ring, 0);
        assert_eq!(g.terrace, town.terrace_of(i), "({}, {})", cell.0, cell.1);
    }
}

/// A terraced town goes into its records in schema 2 and comes back with
/// its levels; a town on one level stays in schema 1, as it always was.
#[test]
fn a_terraced_town_is_stored_in_schema_2_and_a_flat_one_in_schema_1() {
    for (template, schema) in [
        (walled(), record::TERRACED_SCHEMA),
        (village(), record::RECORD_SCHEMA),
    ] {
        let town = laid_village(&template);
        let records = record::to_records(&town);
        assert_eq!(records.last().unwrap().schema, schema, "{}", template.scene);
        let mut store = crate::records::Records::new();
        for r in records {
            store.put(r);
        }
        assert_eq!(record::from_records(&store, 7), record::Stored::Town(town));
    }
}
