use super::chart::{Patch, chart};
use super::ground::{GroundAt, TownGround};
use super::pieces::{Meshes, cut_building};
use super::*;
use crate::topology::dual_sphere;
use glam::{Vec2, Vec3};
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

fn harbour() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/coast.json"
    ))
    .expect("coast.json");
    serde_json::from_str(&text).expect("coast.json parses")
}

fn walled() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/town.json"
    ))
    .expect("town.json");
    serde_json::from_str(&text).expect("town.json parses")
}

fn tundra() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/tundra.json"
    ))
    .expect("tundra.json");
    serde_json::from_str(&text).expect("tundra.json parses")
}

fn jungle() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/jungle.json"
    ))
    .expect("jungle.json");
    serde_json::from_str(&text).expect("jungle.json parses")
}

fn desert() -> Template {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/settlements/v1/desert.json"
    ))
    .expect("desert.json");
    serde_json::from_str(&text).expect("desert.json parses")
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
    // The same cells, found by the rule the town's layout names.
    let before = TownGround::new(patch, RADIUS_M, &cells, town.terrace as f32, floored)
        .by_chord(town.layout >= record::CHORD_LAYOUT);
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
    // Its stair towers and keep (slice 4c).
    "tower",
    "keep",
    // The harbour (slice 4d).
    "whitewash",
    "driftwood",
    // The desert (slice 4e).
    "sandstone",
    "adobe",
    // The tundra (slice 4g).
    "granite",
    "ice",
    "igloo",
];

#[test]
fn every_kit_a_saved_town_can_name_is_shipped() {
    let kits = kits();
    for name in KITS_SAVED_TOWNS_NAME {
        assert!(kits.get(name).is_some(), "kits.ron dropped {name}");
    }
    for b in village()
        .buildings
        .iter()
        .chain(&walled().buildings)
        .chain(&harbour().buildings)
        .chain(&desert().buildings)
        .chain(&tundra().buildings)
    {
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

/// Slice 4c: the walled town's curtain wall is cut from its template on
/// the town's stored chart. Every wall cell stands on its own cell's
/// ground and rises to the template's top, where the walker stands on the
/// wall walk; a body in a wall is held, and under a gate's vault it walks
/// through; merlons stand only on edges that look out.
#[test]
fn the_walled_towns_wall_stands_and_its_gates_open() {
    let template = walled();
    assert_eq!(template.masonry.len(), 116);
    let town = laid_village(&template);
    let (patch, _) = patch();
    let natural = slope();
    let b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        b.solids.len(),
        template.buildings.len() + template.masonry.len()
    );
    assert_eq!(b.rooms.len(), b.solids.len());
    let datum = record::datum(&template);
    let walls: BTreeSet<(i32, i32)> = template.masonry.iter().map(|m| (m.c, m.r)).collect();
    let mut gates = 0;
    for (m, s) in template
        .masonry
        .iter()
        .zip(&b.solids[template.buildings.len()..])
    {
        let top = town.terrace as f32 + (m.to - datum) as f32;
        let up = s.frame.origin.normalize();
        let ground = s.frame.origin.length() - RADIUS_M;
        let (floor, _) = s.stand(up * (RADIUS_M + top + 0.4), 1.0);
        let floor = floor.unwrap_or_else(|| panic!("({}, {}): no wall walk", m.c, m.r));
        assert!(
            (floor - (RADIUS_M + top)).abs() < 0.01,
            "({}, {}): the walk at {} m, not {top} m",
            m.c,
            m.r,
            floor - RADIUS_M
        );
        let body = up * (RADIUS_M + ground + 1.0);
        if s.solids[0].y0 > 0.5 {
            gates += 1;
            assert!(s.solids[0].y0 >= 3.99, "a gate's vault is 4 m up");
            assert!(!s.holds(body, 0.9, 0.3), "({}, {}): the passage", m.c, m.r);
        } else {
            assert!(s.holds(body, 0.9, 0.3), "({}, {}): the wall", m.c, m.r);
        }
        for &d in &m.merlons {
            assert!(
                !walls.contains(&neighbour(m.c, m.r, usize::from(d))),
                "({}, {}): a merlon on edge {d} faces more wall",
                m.c,
                m.r
            );
        }
    }
    assert_eq!(gates, 4, "two gates, two cells each");
}

/// Slice 4c: each stair tower's newel climbs to the wall walk, and its way
/// out stands at the walk's height on the edge where the wall is; the
/// keep's newel climbs to its roof, which is walked on over its ring.
#[test]
fn a_stair_tower_climbs_to_the_walk_and_the_keep_to_its_roof() {
    let template = walled();
    let town = laid_village(&template);
    let (patch, _) = patch();
    let natural = slope();
    let b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let masonry_at = |c: i32, r: i32| {
        template
            .masonry
            .iter()
            .position(|m| (m.c, m.r) == (c, r))
            .map(|k| &b.solids[template.buildings.len() + k])
    };
    let (mut towers, mut keeps) = (0, 0);
    for (def, s) in town.buildings.iter().zip(&b.solids) {
        let Some(n) = &def.newel else {
            continue;
        };
        let climb = s
            .surfaces
            .iter()
            .find_map(|x| match x {
                pieces::Surface::Newel { top, .. } => Some(*top),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{}: no newel", def.name));
        assert!(
            (climb - n.top_m).abs() < 0.01,
            "{}: climbs to {climb}",
            def.name
        );
        let floor = s.frame.origin.length();
        if def.cells.len() == 1 {
            towers += 1;
            let [c, r] = def.cells[0];
            let (d, y) = n.exits[0];
            let (c2, r2) = neighbour(c, r, usize::from(d));
            let wall = masonry_at(c2, r2)
                .unwrap_or_else(|| panic!("{}: no wall across its way out", def.name));
            let up = wall.frame.origin.normalize();
            let (walk, _) = wall.stand(up * (floor + y + 0.4), 1.0);
            let walk = walk.expect("the walk");
            assert!(
                (walk - (floor + y)).abs() < 0.05,
                "{}: the way out at {y} m, the walk at {} m",
                def.name,
                walk - floor
            );
        } else {
            keeps += 1;
            let roof = 3.0 * def.storeys as f32;
            let walked = s
                .surfaces
                .iter()
                .filter(|x| {
                    matches!(x, pieces::Surface::Floor { top, .. } if (*top - roof).abs() < 0.1)
                })
                .count();
            assert_eq!(walked, def.cells.len() - 1, "the keep's roof over its ring");
            assert!(
                n.exits.iter().any(|&(_, y)| (y - roof).abs() < 0.01),
                "a way onto the roof"
            );
        }
    }
    assert_eq!((towers, keeps), (2, 1));
}

/// Task 0.13: the walker's way through each gate and up each stair tower
/// onto the wall walk, asked of what the walker asks a town (`holds` and
/// `stand`): a body 1.8 m tall and 0.3 m round, its feet on the highest
/// top within a tread's reach.
#[test]
fn a_walker_goes_through_a_gate_and_up_a_tower_onto_the_walk() {
    walk_the_walls(&walled(), 4, 80, 200, 2);
}

/// Through a template's gates and into its walls, along its walk from cell
/// to cell, and up each of its towers onto the walk (slices 4c and 4g):
/// `gates` gate cells let the body by, more than `walls` wall cells stop
/// it, more than `seams` steps from cell to cell keep the feet on the walk,
/// and `towers` towers climb to it.
fn walk_the_walls(
    template: &Template,
    gates: usize,
    walls_into: usize,
    seams_min: usize,
    towers_n: usize,
) {
    use pieces::Surface;
    let town = laid_village(template);
    let (patch, _) = patch();
    let natural = slope();
    let b = record::build_town(
        &town,
        Some(template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let natural = slope();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, move |d| natural(d).floor())
        .unwrap_or_else(|e| panic!("{e}"));
    let centre = |(c, r): (i32, i32)| chart.cell(c, r).map(|i| patch.cells[i].direction);
    let held = |p: Vec3| b.solids.iter().any(|s| s.holds(p, 0.9, 0.3));
    let walls: BTreeSet<(i32, i32)> = template.masonry.iter().map(|m| (m.c, m.r)).collect();
    // The masonry is the last of the pieces.
    let n = b.solids.len() - template.masonry.len();

    // Across every wall cell from the cell before it to the cell after, on
    // the ground of the wall's cell: a gate lets the body by, a wall stops it.
    let (mut through, mut stopped) = (0, 0);
    for (m, s) in template.masonry.iter().zip(&b.solids[n..]) {
        let across = (0..3).find_map(|d| {
            let (a, z) = (neighbour(m.c, m.r, d), neighbour(m.c, m.r, d + 3));
            if walls.contains(&a) || walls.contains(&z) {
                return None;
            }
            Some((centre(a)?, centre(z)?))
        });
        let Some((a, z)) = across else {
            continue;
        };
        let feet = s.frame.origin.length();
        let stops = (0..=40).any(|k| held(a.lerp(z, k as f32 / 40.0).normalize() * (feet + 0.95)));
        if s.solids[0].y0 > 0.5 {
            assert!(
                !stops,
                "({}, {}): the gate's passage stops the walker",
                m.c, m.r
            );
            through += 1;
        } else {
            assert!(stops, "({}, {}): walked through the wall", m.c, m.r);
            stopped += 1;
        }
    }
    assert_eq!(through, gates, "every gate cell");
    assert!(stopped > walls_into, "{stopped} wall cells walked into");

    // Along the walk from every wall cell to each wall cell beside it, the
    // feet always on the walk: no seam between two cells' frames.
    let datum = record::datum(template);
    let mut seams = 0;
    for (m, s) in template.masonry.iter().zip(&b.solids[n..]) {
        let walk = RADIUS_M + town.terrace as f32 + (m.to - datum) as f32;
        for d in 0..6 {
            let next = neighbour(m.c, m.r, d);
            let Some(k) = template.masonry.iter().position(|x| (x.c, x.r) == next) else {
                continue;
            };
            let (a, z) = (
                s.frame.origin.normalize(),
                b.solids[n + k].frame.origin.normalize(),
            );
            for j in 0..=40 {
                let at = a.lerp(z, j as f32 / 40.0).normalize() * (walk + 0.01);
                let feet = [s, &b.solids[n + k]]
                    .iter()
                    .filter_map(|x| x.stand(at, 0.3).0)
                    .fold(f32::MIN, f32::max);
                assert!(
                    (feet - walk).abs() < 0.02,
                    "({}, {}) to {next:?}: the feet at {} m on a walk at {} m",
                    m.c,
                    m.r,
                    feet - RADIUS_M,
                    walk - RADIUS_M
                );
            }
            seams += 1;
        }
    }
    assert!(seams > seams_min, "{seams} steps between wall cells");

    // Up each tower's newel from its foot, the feet on each tread in turn,
    // then round its landing and out of its doorway onto the walk.
    let mut towers = 0;
    for (def, s) in town.buildings.iter().zip(&b.solids) {
        let Some(newel) = def.newel.as_ref().filter(|_| def.cells.len() == 1) else {
            continue;
        };
        let Some(&Surface::Newel {
            centre: c,
            start,
            sense,
            top,
            turn_m,
            landing,
            ..
        }) = s
            .surfaces
            .iter()
            .find(|x| matches!(x, Surface::Newel { .. }))
        else {
            panic!("{}: no newel", def.name);
        };
        let floor = s.frame.origin.length();
        let (ed, ey) = newel.exits[0];
        let [tc, tr] = def.cells[0];
        let out = neighbour(tc, tr, usize::from(ed));
        let wall = template
            .masonry
            .iter()
            .position(|m| (m.c, m.r) == out)
            .map(|k| &b.solids[n + k])
            .unwrap_or_else(|| panic!("{}: no wall at its doorway", def.name));
        // Feet on whatever the tower or the wall answers within a tread of
        // where they are; the body over them in nothing.
        let step = |plan: glam::Vec2, feet: f32, what: &str| -> f32 {
            let at = s.frame.world(Vec3::new(plan.x, feet, plan.y));
            let up = at.normalize();
            let next = [s, wall]
                .iter()
                .filter_map(|x| x.stand(at, 0.3).0)
                .fold(f32::MIN, f32::max)
                - floor;
            assert!(
                next > feet - 0.3,
                "{}, {what}: fell from {feet} m to {next} m",
                def.name
            );
            let body = up * (floor + next + 0.95);
            assert!(!held(body), "{}, {what}: held at {next} m", def.name);
            next
        };
        let tau = std::f32::consts::TAU;
        let r_walk = 0.72;
        let on = |phi: f32| {
            let a = start + sense * phi;
            c + glam::Vec2::new(a.cos(), a.sin()) * r_walk
        };
        let end = top / turn_m * tau;
        let mut feet = 0.0;
        for k in 1..=200 {
            let phi = end * k as f32 / 200.0;
            feet = step(on(phi), feet, "the newel");
        }
        assert!((feet - top).abs() < 0.01, "{}: up at {feet} m", def.name);
        assert!(
            (top - ey).abs() < 0.01,
            "{}: its way out at {ey} m",
            def.name
        );
        // The doorway's bearing on the landing: the wall cell's centre.
        let w = s.frame.local(
            centre(out).unwrap_or_else(|| panic!("{}: the walk is off the chart", def.name))
                * (floor + ey),
        );
        let w = glam::Vec2::new(w.x, w.z);
        let a_out = (w - c).y.atan2((w - c).x);
        // How far round from the landing's end the doorway is, folded to a
        // half turn either way: on the landing, or a few degrees short of
        // its end where the cell's real corners turn it (the ice towers').
        let past = (sense * (a_out - start) - end + std::f32::consts::PI).rem_euclid(tau)
            - std::f32::consts::PI;
        assert!(
            (-10f32.to_radians()..=landing + 1e-3).contains(&past),
            "{}: the doorway {:.0} degrees round from the landing's end, past its {:.0}",
            def.name,
            past.to_degrees(),
            landing.to_degrees()
        );
        // From the last tread straight out through the doorway: the landing
        // is 30 degrees, so close to the post its rail is a body's width away.
        let from = on(end);
        for k in 1..=40 {
            feet = step(from.lerp(w, k as f32 / 40.0), feet, "the doorway");
        }
        assert!(
            (feet - ey).abs() < 0.05,
            "{}: on the walk at {feet} m",
            def.name
        );
        towers += 1;
    }
    assert_eq!(towers, towers_n);
}

/// A coast across the test patch for the harbour (slice 4d): the ground
/// rising a metre in ten from the sea, the shore 15 m to one side of the
/// patch's centre.
fn coast() -> impl Fn(Vec3) -> f32 {
    let (patch, at) = patch();
    let centre = patch.cells[*at].direction;
    let (north, east) = crate::geo::north_east(centre);
    let inland = (north + east * 0.5).normalize();
    move |d: Vec3| ((d - centre).dot(inland) * RADIUS_M + 15.0) / 10.0
}

/// The harbour laid on the test coast where it lies best, and the share of
/// its cells that agree with the coast about the sea.
fn laid_harbour() -> (Template, record::Town, f32) {
    let template = harbour();
    let (patch, at) = patch();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let side = patch.side_toward(*at, east);
    let anchor = record::template_anchor(&template);
    let (cell, d0, share) = sea::placement(&template, patch, *at, side, anchor, coast(), 0.0)
        .unwrap_or_else(|e| panic!("{e}"));
    let town = record::lay_at_sea(&template, 9, patch, cell, d0, coast(), 0.0)
        .unwrap_or_else(|e| panic!("{e}"));
    (template, town, share)
}

/// Slice 4d: a harbour is turned and shifted so its sea lies over the
/// planet's, and lying it any other way agrees less.
#[test]
fn a_harbour_lies_with_its_sea_over_the_planets() {
    let started = std::time::Instant::now();
    let (template, _, share) = laid_harbour();
    println!(
        "the harbour lies with {:.1}% of its cells agreeing about the sea, placed and laid in {:.2} s",
        share * 100.0,
        started.elapsed().as_secs_f32()
    );
    assert!(share >= 0.75, "{:.0}% of its cells agree", share * 100.0);
    // The same anchor turned to each other side does worse.
    let (patch, at) = patch();
    let wanted: BTreeSet<(i32, i32)> = template.ground.iter().map(|g| (g.c, g.r)).collect();
    let anchor = record::template_anchor(&template);
    let natural = coast();
    let agree = |d0: usize| {
        let charted = chart(patch, anchor, *at, d0, &wanted).expect("charted");
        let n = template
            .ground
            .iter()
            .filter(|g| {
                charted
                    .cell(g.c, g.r)
                    .is_some_and(|i| (natural(patch.cells[i].direction) < 0.0) == (g.h < 0))
            })
            .count();
        n as f32 / template.ground.len() as f32
    };
    let worst = (0..6).map(agree).fold(f32::MAX, f32::min);
    assert!(
        worst < share - 0.2,
        "turned the worst way, {worst:.2} against {share:.2}"
    );
}

/// Slice 4d: a harbour stands on the sea. Its terrace is the sea's surface
/// and every footprint cell is at its own layer in the template; nothing
/// the template puts under the sea is laid; the beach is sand; and the
/// fish huts' cells are charted over the water.
#[test]
fn a_harbour_stands_on_the_sea() {
    let (template, town, _) = laid_harbour();
    assert_eq!(town.terrace, 0);
    assert!(!town.over_sea.is_empty());
    let ground: std::collections::BTreeMap<(i32, i32), &GroundCell> =
        template.ground.iter().map(|g| ((g.c, g.r), g)).collect();
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let laid: BTreeSet<(i32, i32)> = town.cells.iter().map(|c| (c.0, c.1)).collect();
    for (cell, &level) in town.cells.iter().zip(&town.levels) {
        let g = ground[&(cell.0, cell.1)];
        assert!(g.h >= 0, "({}, {}) laid under the sea", cell.0, cell.1);
        if built.contains(&(cell.0, cell.1)) {
            assert_eq!(i32::from(level), g.h, "({}, {})", cell.0, cell.1);
        }
        if g.top == "ivorysand" {
            assert_eq!(cell.4, Some(record::Top::Sand), "the beach");
        }
    }
    for c in 10..=36 {
        assert!(laid.contains(&(c, 15)), "the quay at ({c}, 15)");
    }
    let quay = town
        .cells
        .iter()
        .position(|c| (c.0, c.1) == (20, 15))
        .unwrap();
    assert_eq!(town.terrace_of(quay), 1.0, "the quay a metre over the sea");
    let over: BTreeSet<(i32, i32)> = town.over_sea.iter().map(|c| (c.0, c.1)).collect();
    assert!(over.is_disjoint(&laid));
    for b in template.buildings.iter().filter(|b| b.stilts.is_some()) {
        for &[c, r] in &b.cells {
            assert!(over.contains(&(c, r)), "{} at ({c}, {r})", b.name);
        }
    }
    // The ground: the footprint is the town's, and the water is not.
    let (patch, _) = patch();
    let (chart, tg) = record::ground_of(&town, patch, RADIUS_M, coast()).expect("ground");
    for &(c, r) in &over {
        let d = patch.cells[chart.cell(c, r).unwrap()].direction;
        assert_ne!(
            tg.at(d).map(|g| g.ring),
            Some(0),
            "({c}, {r}) over the water"
        );
    }
}

/// Slice 4d: a harbour is written in schema 3 with its cells over the sea,
/// and read back whole; a land template is not laid on the sea, nor a sea
/// one on land.
#[test]
fn a_harbour_is_stored_in_schema_3() {
    let (template, town, _) = laid_harbour();
    let records = record::to_records(&town);
    assert_eq!(records.last().unwrap().schema, record::SEA_SCHEMA);
    let mut store = crate::records::Records::new();
    for r in records {
        store.put(r);
    }
    assert_eq!(record::from_records(&store, 9), record::Stored::Town(town));
    let (patch, at) = patch();
    assert!(record::lay(&template, 9, patch, *at, 0, coast()).is_err());
    assert!(record::lay_at_sea(&walled(), 9, patch, *at, 0, coast(), 0.0).is_err());
}

/// The harbour laid on the test coast and cut whole: its buildings, then
/// its piers' stretches and its light, every door open.
/// The drawn sea's surface over the radius, as the game draws it: half a
/// metre under the layers' sea level (`water.ron`, `depth_offset_m`).
const SHEET_M: f32 = -0.5;

fn cut_harbour() -> (Template, record::Town, record::Built) {
    let (template, town, _) = laid_harbour();
    let (patch, _) = patch();
    let natural = coast();
    let mut b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    for s in &mut b.solids {
        for d in &mut s.doors {
            d.open = true;
        }
    }
    (template, town, b)
}

/// What a walker's feet find at `p` (planet-local, at the feet): the
/// highest floor any of the town's pieces answers within a tread's reach,
/// as a radius.
fn feet_on(b: &record::Built, p: Vec3) -> Option<f32> {
    b.solids
        .iter()
        .filter_map(|s| s.stand(p, 0.3).0)
        .max_by(f32::total_cmp)
}

/// Slice 4d: the walker along the harbour's main pier from the quay to its
/// head, the feet on the planks at 1 m over the sea all the way and the
/// body in nothing of the pier's.
#[test]
fn a_walker_goes_down_the_main_pier_to_its_head() {
    let (template, _, b) = cut_harbour();
    let (patch, _) = patch();
    let pier = template
        .piers
        .iter()
        .max_by(|a, z| {
            let l = |p: &Pier| (p.to[2] - p.from[2]).abs() + (p.to[0] - p.from[0]).abs();
            l(a).total_cmp(&l(z))
        })
        .expect("a pier");
    let deck = RADIUS_M + pier.from[1];
    let steps = 120;
    for k in 0..=steps {
        let t = k as f32 / steps as f32;
        let (x, z) = (
            pier.from[0] + (pier.to[0] - pier.from[0]) * t,
            pier.from[2] + (pier.to[2] - pier.from[2]) * t,
        );
        let d = sea::point(&b.chart, patch, x, z, template.grid.cell_m).expect("on the chart");
        let feet = feet_on(&b, d * (deck + 0.01)).unwrap_or_else(|| {
            panic!(
                "no planks at {:.1} m of the pier",
                t * (pier.to[2] - pier.from[2]).abs()
            )
        });
        assert!(
            (feet - deck).abs() < 0.03,
            "the deck at {} m",
            feet - RADIUS_M
        );
        // The pier's own pieces: its dressing (a crate at the head, on the
        // pier's middle) is the walker's to go round.
        let body = d * (deck + 0.95);
        let pieces = &b.solids[..b.solids.len() - b.dressing];
        assert!(
            !pieces.iter().any(|s| s.holds(body, 0.9, 0.3)),
            "held on the pier at {t:.2}"
        );
    }
}

/// Slice 4d: a boathouse has no wall on its seaward edges, so the walker
/// comes in from the beach's edge; its other edges are walls.
#[test]
fn a_walker_comes_into_a_boathouse_from_the_sea() {
    let (template, town, b) = cut_harbour();
    let (patch, _) = patch();
    let n = template
        .buildings
        .iter()
        .position(|x| !x.open.is_empty())
        .expect("a boathouse");
    let def = &template.buildings[n];
    let s = &b.solids[n];
    let floor = RADIUS_M + town.terrace as f32 + def.base as f32;
    let (mut open_edges, mut walls) = (0, 0);
    for &[c, r] in &def.cells {
        let centre = patch.cells[b.chart.cell(c, r).unwrap()].direction;
        for d in 0..6 {
            let (c2, r2) = neighbour(c, r, d);
            let door = def.doors.iter().any(|x| x[..3] == [c, r, d as i32]);
            if def.cells.contains(&[c2, r2]) || door {
                continue;
            }
            let beyond = patch.cells[b.chart.cell(c2, r2).unwrap()].direction;
            let held = (0..=20).any(|k| {
                let p = centre.lerp(beyond, k as f32 / 20.0).normalize() * (floor + 0.95);
                s.holds(p, 0.9, 0.3)
            });
            let open = def.open.contains(&[c, r, d as i32]);
            assert_eq!(!held, open, "({c}, {r}) edge {d}: open {open}, held {held}");
            if open {
                open_edges += 1;
            } else {
                walls += 1;
            }
        }
    }
    assert_eq!(open_edges, def.open.len());
    assert!(walls >= 4, "{walls} walls walked into");
}

/// Slice 4d: the walker from the hut pier up each fish hut's porch stair
/// onto its deck and in at its door, the feet never falling and the body
/// in nothing.
#[test]
fn a_walker_climbs_a_fish_huts_porch_and_goes_in() {
    let (template, _, b) = cut_harbour();
    let (patch, _) = patch();
    let mut huts = 0;
    for (n, def) in template.buildings.iter().enumerate() {
        let Some(st) = &def.stilts else {
            continue;
        };
        let s = &b.solids[n];
        let Some(&pieces::Surface::Flight { foot, dir, len, .. }) = s
            .surfaces
            .iter()
            .find(|x| matches!(x, pieces::Surface::Flight { .. }))
        else {
            panic!("{}: no porch stair", def.name);
        };
        let floor = s.frame.origin.length();
        let plan = |c: i32, r: i32| {
            s.frame
                .plane(patch.cells[b.chart.cell(c, r).unwrap()].direction)
        };
        let [dc, dr] = st.deck[0];
        let [hc, hr] = def.cells[0];
        // The way: the pier before the stair's foot, up the stair, across the
        // deck, in at the door.
        let way = [
            foot - dir * 0.5,
            foot + dir * len,
            plan(dc, dr),
            plan(hc, hr),
        ];
        let start = s.frame.world(Vec3::new(way[0].x, -1.0 + 0.01, way[0].y));
        let mut feet = feet_on(&b, start).expect("the pier at the stair's foot") - floor;
        for leg in way.windows(2) {
            for k in 1..=40 {
                let p = leg[0].lerp(leg[1], k as f32 / 40.0);
                let at = s.frame.world(Vec3::new(p.x, feet, p.y));
                let next = feet_on(&b, at).map(|f| f - floor);
                let next = next.unwrap_or_else(|| panic!("{}: nothing underfoot at {p}", def.name));
                assert!(
                    next > feet - 0.3,
                    "{}: fell from {feet} to {next}",
                    def.name
                );
                let body = s.frame.world(Vec3::new(p.x, next + 0.95, p.y));
                assert!(
                    !b.solids.iter().any(|x| x.holds(body, 0.9, 0.3)),
                    "{}: held at {p} on {next}",
                    def.name
                );
                feet = next;
            }
        }
        assert!(
            feet.abs() < 0.05,
            "{}: inside on its floor, {feet}",
            def.name
        );
        huts += 1;
    }
    assert_eq!(huts, 2);
}

/// Task 5.2: a town's street lamps stand in the first layer over their
/// cells' terrace, one a column, and the ground answers a lamp only at its
/// own cell. A harbour's lanterns over the water stand at its piers' height.
#[test]
fn a_towns_lamps_stand_over_their_cells() {
    let template = walled();
    let town = laid_village(&template);
    let (patch, _) = patch();
    let (chart, ground) = record::ground_of(&town, patch, RADIUS_M, slope()).expect("ground");
    let lamps = record::lamps_of(&town, &template, &chart, patch);
    assert_eq!(lamps.len(), template.lamps.len(), "a lamp a street lamp");
    let terrace: std::collections::BTreeMap<(i32, i32), f32> = town
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| ((c.0, c.1), town.terrace_of(i)))
        .collect();
    for (&[c, r], l) in template.lamps.iter().zip(&lamps) {
        assert_eq!(l.altitude_m, terrace[&(c, r)], "({c}, {r}) on its terrace");
        assert_eq!(l.material, crate::terrain::Material::LanternPost);
    }
    let ground = ground.with_lamps(lamps.clone());
    for (&[c, r], l) in template.lamps.iter().zip(&lamps) {
        assert_eq!(ground.lamp(l.direction), Some(l), "({c}, {r})");
        let beside = (0..6)
            .map(|d| neighbour(c, r, d))
            .find(|n| !template.lamps.contains(&[n.0, n.1]))
            .unwrap();
        let d = patch.cells[chart.cell(beside.0, beside.1).unwrap()].direction;
        assert_eq!(ground.lamp(d), None, "none beside ({c}, {r})");
    }
    // The same on a body of the game's radius, where half a metre's cosine
    // is 1.0 in f32 and a lamp found by the dot product is a coin toss
    // (design, "Finding: half of every town's lamps stand nowhere").
    let (chart, ground) = record::ground_of(&town, patch, 4800.0, slope()).expect("ground");
    let ground = ground.with_lamps(lamps.clone());
    for (&[c, r], l) in template.lamps.iter().zip(&lamps) {
        assert_eq!(ground.lamp(l.direction), Some(l), "({c}, {r}) at 4800 m");
        let beside = (0..6)
            .map(|d| neighbour(c, r, d))
            .find(|n| !template.lamps.contains(&[n.0, n.1]))
            .unwrap();
        let d = patch.cells[chart.cell(beside.0, beside.1).unwrap()].direction;
        assert_eq!(ground.lamp(d), None, "none beside ({c}, {r}) at 4800 m");
    }
    // The harbour: its street lamps on their terraces, its lanterns over the
    // piers a metre over the sea.
    let (template, town, _) = laid_harbour();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, coast()).expect("ground");
    let lamps = record::lamps_of(&town, &template, &chart, patch);
    assert!(
        lamps.len() >= template.lamps.len() + template.lanterns.len() - 2,
        "{} lamps",
        lamps.len()
    );
    let over: BTreeSet<(i32, i32)> = town.over_sea.iter().map(|c| (c.0, c.1)).collect();
    let on_piers = template
        .lanterns
        .iter()
        .filter(|l| over.contains(&sea::cell_at(l[0], l[2], template.grid.cell_m)))
        .count();
    assert!(on_piers > 0, "lanterns over the water");
    assert!(
        lamps.iter().filter(|l| l.altitude_m == 1.0).count() >= on_piers,
        "a pier's lantern at the pier's metre"
    );
}

/// The harbour's dressing pieces (task 4.2c), the last of its cut pieces,
/// each with what the template says it is: its things in order, then the
/// shipyard's hull, planks and slip.
fn dressing_of(b: &record::Built) -> &[pieces::BuildingSolids] {
    &b.solids[b.solids.len() - b.dressing..]
}

/// Task 4.2c: every dressing thing of the harbour stands, on the chart, on
/// what is under it: a pier's deck where it is on a pier, else the ground
/// as the town laid it.
#[test]
fn a_harbours_dressing_stands_on_what_is_under_it() {
    let (template, _, b) = cut_harbour();
    let natural = coast();
    assert_eq!(b.dressing_skipped, 0, "things left off the chart");
    assert_eq!(b.dressing, template.dressing.len() + 3);
    let rest = &b.solids[..b.solids.len() - b.dressing];
    let mut on_decks = 0;
    for (k, s) in dressing_of(&b).iter().enumerate() {
        if s.surfaces
            .iter()
            .any(|x| matches!(x, pieces::Surface::Ramp { .. }))
        {
            continue;
        }
        let d = s.frame.origin.normalize();
        let at = s.frame.origin.length();
        let n = natural(d).floor();
        let ground = RADIUS_M + b.ground.at(d).map_or(n, |g| g.height(n));
        let deck = rest
            .iter()
            .filter_map(|x| x.stand(d * (at + 0.01), 0.3).0)
            .max_by(f32::total_cmp);
        let on_deck = deck.is_some_and(|h| (h - at).abs() < 0.02);
        assert!(
            on_deck || (ground - at).abs() < 0.02,
            "thing {k} at {:.2} m stands on nothing: the ground at {:.2} m, a deck at {:?}",
            at - RADIUS_M,
            ground - RADIUS_M,
            deck.map(|h| h - RADIUS_M)
        );
        on_decks += usize::from(on_deck);
    }
    // The pier heads' barrels and crates and the main pier's bollards.
    assert!(on_decks >= 18, "{on_decks} things on the piers");
}

/// Task 4.2c: a walker is held by a stall's counter and not by the street
/// in front of it, and goes round a barrel, held at it and free a step off
/// it on every side.
#[test]
fn a_walker_is_held_by_a_stall_and_goes_round_a_barrel() {
    let (template, _, b) = cut_harbour();
    let (patch, _) = patch();
    let cell_m = template.grid.cell_m;
    let body = |s: &pieces::BuildingSolids, x: f32, z: f32| {
        let d = sea::point(&b.chart, patch, x, z, cell_m).expect("on the chart");
        s.holds(d * (s.frame.origin.length() + 0.95), 0.9, 0.3)
    };
    let (mut stalls, mut barrels) = (0, 0);
    for (dress, s) in template.dressing.iter().zip(dressing_of(&b)) {
        match *dress {
            Dress::Stall { x, z, .. } => {
                assert!(body(s, x, z - 0.3), "a stall's counter holds the walker");
                assert!(
                    !body(s, x, z + 1.3),
                    "the street in front of a stall is open"
                );
                stalls += 1;
            }
            Dress::Barrel { x, z, .. } => {
                assert!(body(s, x, z), "a barrel holds the walker");
                // A step off it in its own frame's metres: it stands at the
                // frame's middle.
                for k in 0..8 {
                    let a = k as f32 / 8.0 * std::f32::consts::TAU;
                    let p = s.frame.world(Vec3::new(0.7 * a.cos(), 0.95, 0.7 * a.sin()));
                    assert!(!s.holds(p, 0.9, 0.3), "held 0.7 m off a barrel");
                }
                barrels += 1;
            }
            _ => {}
        }
    }
    assert_eq!((stalls, barrels), (4, 5));
}

/// Task 4.2c: the walker down the shipyard's slip from the beach into the
/// water, the feet going down with it and never falling, the body in
/// nothing, ending under the sea's surface.
#[test]
fn a_walker_goes_down_the_slip_into_the_water() {
    let (_, town, b) = cut_harbour();
    let s = dressing_of(&b)
        .iter()
        .find(|s| {
            s.surfaces
                .iter()
                .any(|x| matches!(x, pieces::Surface::Ramp { .. }))
        })
        .expect("the slip");
    let Some(&pieces::Surface::Ramp {
        foot, dir, len, to, ..
    }) = s.surfaces.first()
    else {
        panic!("the slip is a ramp");
    };
    let floor = s.frame.origin.length();
    // From half a metre down it: its head lies against the hull in frame.
    let mut feet = to * 0.05;
    for k in 1..=80 {
        let t = 0.05 + 0.95 * k as f32 / 80.0;
        let p = foot + dir * (len * t);
        let at = s.frame.world(Vec3::new(p.x, feet + 0.01, p.y));
        let next = feet_on(&b, at).map(|f| f - floor);
        let next = next.unwrap_or_else(|| panic!("nothing underfoot at {:.1} m", len * t));
        assert!(next > feet - 0.1, "fell from {feet} to {next}");
        let body = s.frame.world(Vec3::new(p.x, next + 0.95, p.y));
        assert!(
            !b.solids.iter().any(|x| x.holds(body, 0.9, 0.3)),
            "held at {:.1} m down the slip",
            len * t
        );
        feet = next;
    }
    // On the ramp at its foot, or on the seabed where the bed comes up over
    // it.
    assert!(
        feet > to - 0.03 && feet < to + 0.3,
        "at its foot, {feet} against {to}"
    );
    let sea = RADIUS_M + town.terrace as f32;
    assert!(floor + feet < sea - 1.0, "its foot in the water");
}

/// `sail-the-cog` design 6, step 1: the walker from the main pier up the
/// gangplank through the gangway onto the cog's deck, 1.9 m over the sea,
/// across it to the stair and up onto the aftcastle, the feet never
/// falling and the body in nothing.
#[test]
fn a_walker_boards_the_moored_cog_and_climbs_to_its_aftcastle() {
    let (_, _, b) = cut_harbour();
    walk_aboard(&b);
}

/// `sail-the-cog` step 3, part 3: the craft stands where the template moors
/// the ship. `cog::berth` is the ship piece's own frame's origin and its
/// bow, and with the craft's cut stood there in the ship's place, a walker
/// still comes up the town's gangplank, across the deck and up the stair.
#[test]
fn the_craft_cog_stands_at_its_berth_and_is_boarded_up_the_towns_gangplank() {
    let (template, town, mut b) = cut_harbour();
    let n = b.cog.expect("the cog stands");
    let (patch, _) = patch();
    let chart = record::chart_of(&town, patch).unwrap();
    let cog = template.cog.as_ref().expect("the harbour moors a cog");
    let (origin, bow) =
        pieces::cog::berth(patch, &chart, &template, cog, RADIUS_M, SHEET_M).expect("on the chart");
    assert!(origin.distance(b.solids[n].frame.origin) < 1e-3);
    let towns_bow = pieces::cog::bow(patch, &chart, &template, cog).unwrap();
    assert!(bow.distance(towns_bow) < 1e-6);
    let up = origin.normalize();
    assert!(
        bow.dot(up).abs() < 1e-5,
        "the bow lies in the tangent plane"
    );
    let z = -bow;
    let mut craft = pieces::cog::sailing(&mut Meshes::new(), &|_: &str| 2.0, cog.gang_side);
    craft.frame = pieces::Frame {
        origin,
        x: up.cross(z),
        y: up,
        z,
    };
    b.solids[n] = craft;
    walk_aboard(&b);
}

/// From the pier up the gangplank, across the cog's deck and up its stair
/// to the aftcastle, the feet never falling and the body never held.
fn walk_aboard(b: &record::Built) {
    let n = b.cog.expect("the cog stands");
    let (ship, plank) = (&b.solids[n], &b.solids[n + 1]);
    assert!(!b.cog_meshes.is_empty(), "the ship is drawn apart");
    let Some(&pieces::Surface::Ramp {
        foot: pa,
        dir: pd,
        len: pl,
        to: rise,
        ..
    }) = plank.surfaces.first()
    else {
        panic!("the gangplank is a ramp");
    };
    let Some(&pieces::Surface::Flight {
        foot: sf,
        dir: sd,
        len: sl,
        ..
    }) = ship
        .surfaces
        .iter()
        .find(|x| matches!(x, pieces::Surface::Flight { .. }))
    else {
        panic!("the cog has its stair");
    };
    let sea = RADIUS_M + SHEET_M;
    let deck = ship.frame.origin.length() + 1.9;
    assert!(
        (deck - sea - 1.9).abs() < 0.01,
        "the deck 1.9 m over the drawn sea"
    );
    let at = |f: &pieces::Frame, p: Vec2| f.world(Vec3::new(p.x, 0.0, p.y)).normalize();
    let way = [
        at(&plank.frame, pa - pd * 0.6),
        at(&plank.frame, pa + pd * (pl + 1.0)),
        at(&ship.frame, sf - sd * 0.4),
        at(&ship.frame, sf + sd * (sl + 1.0)),
    ];
    let mut feet = feet_on(b, way[0] * (plank.frame.origin.length() + 0.01))
        .expect("the pier at the gangplank's foot");
    assert!((feet - plank.frame.origin.length()).abs() < 0.03);
    // The pier is a layer over the layers' sea level, the deck 1.9 m over
    // the drawn sea half a metre under it.
    assert!(
        (rise - 0.4).abs() < 0.02,
        "the gangplank climbs {rise} m to the deck"
    );
    for (leg, name) in way
        .windows(2)
        .zip(["the gangplank", "the deck", "the stair"])
    {
        for k in 1..=60 {
            let d = leg[0].lerp(leg[1], k as f32 / 60.0).normalize();
            let next = feet_on(b, d * (feet + 0.01)).unwrap_or_else(|| {
                let l = ship.frame.local(d * (feet + 0.01));
                let e = plank.frame.local(d * (feet + 0.01));
                let end = pa + pd * pl;
                panic!(
                    "{name}: nothing underfoot at {k}: in the ship ({:.3}, {:.3}, {:.3}), {:.3} past the plank's end",
                    l.x, l.y, l.z, (Vec2::new(e.x, e.z) - end).dot(pd)
                )
            });
            assert!(next > feet - 0.3, "{name}: fell from {feet} to {next}");
            let body = d * (next + 0.95);
            assert!(
                !b.solids.iter().any(|x| x.holds(body, 0.9, 0.3)),
                "{name}: held at {k}, the feet {:.2} m over the sea",
                next - sea
            );
            feet = next;
        }
        if name == "the gangplank" {
            assert!((feet - deck).abs() < 0.03, "on the deck, {}", feet - sea);
        }
    }
    assert!(
        (feet - deck - 1.6).abs() < 0.03,
        "on the aftcastle, {:.2} m over the sea",
        feet - sea
    );
}

/// `sail-the-cog` step 3: the ship the craft carries is the moored one cut
/// in the craft's own frame, without its yard: its deck amidships 1.9 m over
/// the waterline, its aftcastle aft (+z), where the helm is, and its yard and
/// furled sail apart, in the rig the craft turns.
#[test]
fn the_cog_cut_for_its_craft_lies_bow_forward_without_its_yard() {
    let repeat = |_: &str| 2.0;
    let frame = pieces::Frame {
        origin: Vec3::ZERO,
        x: Vec3::X,
        y: Vec3::Y,
        z: Vec3::Z,
    };
    let mut moored = Meshes::new();
    pieces::cog::ship_in(&mut moored, &repeat, frame, -std::f32::consts::FRAC_PI_2, 1);
    let mut sailing = Meshes::new();
    let ship = pieces::cog::sailing(&mut sailing, &repeat, 1);
    let top = |x: f32, z: f32| {
        let p = Vec2::new(x, z);
        ship.solids
            .iter()
            .filter(|s| {
                let n = s.outline.len();
                (0..n).all(|i| {
                    let (a, b) = (s.outline[i], s.outline[(i + 1) % n]);
                    (b - a).perp_dot(p - a) >= 0.0
                })
            })
            .map(|s| s.y1)
            .fold(f32::MIN, f32::max)
    };
    // The hull is solid to a hand under the deck's floor at 1.9 m, and the
    // aftcastle's deck 1.6 m over that.
    let (waist, castle) = (top(0.0, 0.0), top(0.0, 5.6));
    assert!((1.75..=1.9).contains(&waist), "the hull's top at {waist}");
    assert!((castle - 3.5).abs() < 0.05, "aftcastle at {castle}");
    let Some(&pieces::Surface::Flight { foot, dir, .. }) = ship
        .surfaces
        .iter()
        .find(|x| matches!(x, pieces::Surface::Flight { .. }))
    else {
        panic!("the cog has its stair");
    };
    assert!(
        dir.y > 0.99,
        "the stair climbs aft to the aftcastle: from {foot} along {dir}"
    );
    let count = |m: &Meshes, k: &str| m.get(k).map_or(0, |b| b.positions.len());
    let rig = pieces::cog::rig(&repeat, false);
    assert_eq!(count(&sailing, "linen"), 0, "the sail is in the rig");
    assert_eq!(count(&moored, "linen"), count(&rig, "linen"));
    assert_eq!(
        count(&moored, "timber"),
        count(&sailing, "timber") + count(&rig, "timber")
    );
    assert!(count(&pieces::cog::rig(&repeat, true), "linen") > count(&rig, "linen"));
}

/// `sail-the-cog` step 3, part 4: the cog's stern lantern stands on its
/// aftcastle, over its deck, with a flame in its cage.
#[test]
fn the_cogs_stern_lantern_stands_on_its_aftcastle() {
    let repeat = |_: &str| 2.0;
    let ship = pieces::cog::sailing(&mut Meshes::new(), &repeat, -1);
    let at = pieces::cog::LANTERN;
    let p = Vec2::new(at.x, at.z);
    let feet = ship
        .solids
        .iter()
        .filter(|s| {
            let n = s.outline.len();
            (0..n).all(|i| {
                let (a, b) = (s.outline[i], s.outline[(i + 1) % n]);
                (b - a).perp_dot(p - a) >= 0.0
            })
        })
        .map(|s| s.y1)
        .fold(f32::MIN, f32::max);
    assert!(
        (feet - 3.5).abs() < 0.01,
        "on the aftcastle's deck, at {feet}"
    );
    assert!(at.y > feet + 1.0, "over the deck, at {}", at.y);
    let lantern = pieces::cog::lantern(&repeat);
    assert!(
        lantern
            .get("flame")
            .is_some_and(|m| !m.positions.is_empty())
    );
    assert!(
        lantern
            .get("timber")
            .is_some_and(|m| !m.positions.is_empty())
    );
}

/// The desert, laid and cut (slice 4e), every door open.
fn cut_desert() -> (Template, record::Town, record::Built) {
    let template = desert();
    let town = laid_village(&template);
    let (patch, _) = patch();
    let natural = slope();
    let mut b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    for s in &mut b.solids {
        for d in &mut s.doors {
            d.open = true;
        }
    }
    (template, town, b)
}

/// Slice 4e: the desert lays with its dunes wild and every other cell
/// built, the plaza at the datum and the oasis a level under it, its tops
/// the terrain's sand and stone; and it cuts whole, its buildings, its five
/// roof stairs and its dressing, its lanterns and braziers lamps.
#[test]
fn the_desert_lays_with_its_dunes_wild_and_its_oasis_a_level_down() {
    let template = desert();
    assert!(template.terraced);
    assert_eq!(template.wild, ["The dunes"]);
    let kits = kits();
    for b in &template.buildings {
        assert!(kits.get(&b.kit).is_some(), "{}: no kit {}", b.name, b.kit);
    }
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    for g in &template.ground {
        assert_eq!(
            built.contains(&(g.c, g.r)),
            g.area != "The dunes",
            "({}, {}) in {}",
            g.c,
            g.r,
            g.area
        );
    }
    let town = laid_village(&template);
    let area: std::collections::BTreeMap<(i32, i32), (&str, &str)> = template
        .ground
        .iter()
        .map(|g| ((g.c, g.r), (g.area.as_str(), g.top.as_str())))
        .collect();
    let (mut oasis, mut plaza) = (0, 0);
    for (cell, &level) in town.cells.iter().zip(&town.levels) {
        let (a, top) = area[&(cell.0, cell.1)];
        match a {
            "The oasis" => {
                assert_eq!(level, -1, "the oasis a level down");
                oasis += 1;
            }
            "The plaza" => {
                assert_eq!(level, 0, "the plaza at the datum");
                plaza += 1;
            }
            _ => {}
        }
        let want = match top {
            "flag" => Some(record::Top::Stone),
            "sand" | "sand2" => Some(record::Top::Sand),
            _ => None,
        };
        assert_eq!(cell.4, want, "({}, {}) {top}", cell.0, cell.1);
    }
    assert_eq!((oasis, plaza), (7, 54));
    let (_, _, b) = cut_desert();
    let n = template.buildings.len();
    assert_eq!(n, 11);
    assert_eq!(
        b.solids.len(),
        n + template.stairs.len() + template.dressing.len()
    );
    assert_eq!(
        (template.stairs.len(), b.dressing, b.dressing_skipped),
        (5, 13, 0)
    );
    let (patch, _) = patch();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, slope()).expect("ground");
    let lamps = record::lamps_of(&town, &template, &chart, patch);
    let braziers = lamps
        .iter()
        .filter(|l| l.material == crate::terrain::Material::Brazier)
        .count();
    assert_eq!(
        (lamps.len(), braziers),
        (7, 5),
        "two lanterns, five braziers"
    );
}

/// Slice 4e: the walker climbs a sandstone house's outside stair from the
/// ground at its foot onto the roof through the parapet's gap, the feet on
/// the steps and the body in nothing; walks across the roof; and is held by
/// the parapet at its far side.
#[test]
fn a_walker_climbs_a_sandstone_houses_stair_onto_its_roof() {
    let (template, _, b) = cut_desert();
    let n = template.buildings.len();
    let held = |p: Vec3| b.solids.iter().any(|x| x.holds(p, 0.9, 0.3));
    for (k, s) in b.solids[n..n + template.stairs.len()].iter().enumerate() {
        let Some(&pieces::Surface::Flight { foot, dir, len, .. }) = s
            .surfaces
            .iter()
            .find(|x| matches!(x, pieces::Surface::Flight { .. }))
        else {
            panic!("stair {k}: no flight");
        };
        let floor = s.frame.origin.length();
        let at = |p: Vec2, y: f32| s.frame.world(Vec3::new(p.x, y, p.y));
        // From the ground before its foot, up the flight to its head, and on
        // over the roof.
        let (from, head) = (foot - dir * 0.6, foot + dir * len);
        let mut feet = 0.0f32;
        let mut roof = None;
        // 4.5 cm steps: a gap at the head the width of a tread's nosing
        // shows.
        let steps = 400;
        for i in 1..=steps {
            let p = from.lerp(head + dir * 12.0, i as f32 / steps as f32);
            let next = feet_on(&b, at(p, feet)).map_or(0.0, |f| f - floor);
            assert!(
                next > feet - 0.3,
                "stair {k}: fell from {feet} to {next} at {p}"
            );
            if held(at(p, next + 0.95)) {
                // Past the head, only the far parapet holds the body.
                assert!(
                    roof.is_some() && (p - head).length() > 2.0,
                    "stair {k}: held at {p} on {next}"
                );
                break;
            }
            if (p - foot).dot(dir) > len + 0.3 {
                roof.get_or_insert(next);
                // The roof is 3.3 m over the floor in the house's own frame;
                // read in the stair's, a few metres off on the 300 m test
                // body, it is up to 6 cm higher.
                assert!((next - 3.3).abs() < 0.1, "stair {k}: on the roof at {next}");
            }
            feet = next;
            assert!(i < steps, "stair {k}: never met the far parapet");
        }
        assert!(roof.is_some(), "stair {k}: never on the roof");
    }
}

/// Slice 4e: the walker goes in at a domed house's door, stands on its
/// floor under its dome, and is held by its walls.
#[test]
fn a_walker_goes_into_a_domed_house_under_its_dome() {
    let (template, _, b) = cut_desert();
    let (patch, _) = patch();
    let held = |p: Vec3| b.solids.iter().any(|x| x.holds(p, 0.9, 0.3));
    let mut houses = 0;
    for (def, s) in template.buildings.iter().zip(&b.solids) {
        if def.roof != "dome" || def.cells.len() != 1 {
            continue;
        }
        let floor = s.frame.origin.length();
        let plan = |(c, r): (i32, i32)| {
            s.frame
                .plane(patch.cells[b.chart.cell(c, r).unwrap()].direction)
        };
        let [c, r, d, _] = def.doors[0];
        let (outside, inside) = (plan(neighbour(c, r, d as usize)), plan((c, r)));
        for k in 0..=40 {
            let p = outside.lerp(inside, k as f32 / 40.0);
            let feet =
                feet_on(&b, s.frame.world(Vec3::new(p.x, 0.0, p.y))).map_or(0.0, |f| f - floor);
            assert!(feet.abs() < 0.1, "{}: feet at {feet}", def.name);
            let body = s.frame.world(Vec3::new(p.x, feet + 0.95, p.y));
            assert!(!held(body), "{}: held at the door, {p}", def.name);
        }
        let room = s.frame.world(Vec3::new(inside.x, 1.6, inside.y));
        assert!(s.shelters(room), "{}: under its dome", def.name);
        let back = plan(neighbour(c, r, (d as usize + 3) % 6));
        let out = (0..=40).any(|k| {
            let p = inside.lerp(back, k as f32 / 40.0);
            held(s.frame.world(Vec3::new(p.x, 0.95, p.y)))
        });
        assert!(out, "{}: walked out through its back wall", def.name);
        houses += 1;
    }
    assert_eq!(houses, 5);
}

/// The tundra, laid and cut (slice 4g), every door open.
fn cut_tundra() -> (Template, record::Town, record::Built) {
    let template = tundra();
    let town = laid_village(&template);
    let (patch, _) = patch();
    let natural = slope();
    let mut b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    for s in &mut b.solids {
        for d in &mut s.doors {
            d.open = true;
        }
    }
    (template, town, b)
}

/// Slice 4g: the tundra lays with its open ground wild and every other
/// cell built, the camp at the datum and the frozen lake a level under it,
/// its tops the terrain's snow; and it cuts whole, its buildings, its lake's
/// ice and its wall, its torches and fire lamps.
#[test]
fn the_tundra_lays_with_its_open_ground_wild_and_its_lake_a_level_down() {
    let template = tundra();
    assert!(template.terraced);
    assert_eq!(template.wild, ["The tundra"]);
    let kits = kits();
    for b in &template.buildings {
        assert!(kits.get(&b.kit).is_some(), "{}: no kit {}", b.name, b.kit);
    }
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let fires: BTreeSet<(i32, i32)> = template
        .fires
        .iter()
        .map(|f| sea::cell_at(f.x, f.z, template.grid.cell_m))
        .collect();
    for g in &template.ground {
        let wanted = g.area != "The tundra"
            || template
                .buildings
                .iter()
                .any(|b| b.cells.contains(&[g.c, g.r]))
            || fires.contains(&(g.c, g.r));
        assert_eq!(
            built.contains(&(g.c, g.r)),
            wanted,
            "({}, {}) in {}",
            g.c,
            g.r,
            g.area
        );
    }
    let town = laid_village(&template);
    let area: std::collections::BTreeMap<(i32, i32), (&str, &str)> = template
        .ground
        .iter()
        .map(|g| ((g.c, g.r), (g.area.as_str(), g.top.as_str())))
        .collect();
    let mut lake = 0;
    for (cell, &level) in town.cells.iter().zip(&town.levels) {
        let (a, top) = area[&(cell.0, cell.1)];
        if a == "The frozen lake" {
            assert_eq!(level, -1, "the lake a level down");
            lake += 1;
        }
        if a == "The ice castle" {
            assert_eq!(level, 0, "the castle at the datum");
        }
        let want = match top {
            "snow" | "ice" => Some(record::Top::Snow),
            _ => None,
        };
        assert_eq!(cell.4, want, "({}, {}) {top}", cell.0, cell.1);
    }
    assert_eq!(lake, 55);
    let (_, _, b) = cut_tundra();
    assert_eq!(template.buildings.len(), 9);
    assert_eq!(
        b.solids.len(),
        template.buildings.len() + 1 + template.masonry.len(),
        "buildings, the lake's ice and the wall"
    );
    assert_eq!(b.dressing_skipped, 0);
    let ice = &b.solids[template.buildings.len()];
    assert_eq!(
        ice.surfaces.len(),
        55,
        "a floor of ice over every cell of the lake"
    );
    let (patch, _) = patch();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, slope()).expect("ground");
    let lamps = record::lamps_of(&town, &template, &chart, patch);
    let torches = lamps
        .iter()
        .filter(|l| l.material == crate::terrain::Material::Torch)
        .count();
    assert_eq!(
        (lamps.len(), torches),
        (5, 4),
        "four torches and the camp's fire"
    );
}

/// Slice 4g: the walker goes in through each igloo's tunnel to the middle
/// of its dome, standing up all the way, and is held by its wall going on
/// out the back; a walker outside is held before the dome's shell.
#[test]
fn a_walker_goes_into_an_igloo_through_its_tunnel() {
    let (template, _, b) = cut_tundra();
    let (patch, _) = patch();
    let held = |p: Vec3| b.solids.iter().any(|x| x.holds(p, 0.9, 0.3));
    let mut igloos = 0;
    for (def, s) in template.buildings.iter().zip(&b.solids) {
        if def.kit != "igloo" {
            continue;
        }
        let [c, r, d, _] = def.doors[0];
        let plan = |(c, r): (i32, i32)| {
            s.frame
                .plane(patch.cells[b.chart.cell(c, r).unwrap()].direction)
        };
        let middle = plan((c, r));
        let way = (plan(neighbour(c, r, d as usize)) - middle).normalize();
        let body = |p: glam::Vec2| s.frame.world(Vec3::new(p.x, 0.95, p.y));
        // From 5 m out, along the tunnel to the middle.
        for k in 0..=50 {
            let p = middle + way * (5.0 * (1.0 - k as f32 / 50.0));
            assert!(
                !held(body(p)),
                "{}: held at {:.2} m out",
                def.name,
                (p - middle).length()
            );
        }
        let ceiling = s.ceiling(s.frame.world(Vec3::new(middle.x, 0.95, middle.y)), 0.3);
        assert!(
            ceiling.is_none(),
            "{}: something over the walker's head",
            def.name
        );
        // On out the back: the wall holds the walker inside the dome.
        let back = (0..=40).find(|&k| held(body(middle - way * (2.5 * k as f32 / 40.0))));
        let back = back.map(|k| 2.5 * k as f32 / 40.0);
        assert!(
            back.is_some_and(|m| m < 1.7),
            "{}: walked out the back, held at {back:?}",
            def.name
        );
        // From outside, at the side: held before the shell at 2.3 m.
        let side = glam::Vec2::new(-way.y, way.x);
        let from_out =
            (0..=40).find(|&k| held(body(middle + side * (5.0 - 3.0 * k as f32 / 40.0))));
        let from_out = from_out.map(|k| 5.0 - 3.0 * k as f32 / 40.0);
        assert!(
            from_out.is_some_and(|m| m > 2.3),
            "{}: walked into the dome's shell, held at {from_out:?}",
            def.name
        );
        let i = template
            .buildings
            .iter()
            .position(|x| std::ptr::eq(x, def))
            .unwrap();
        assert_eq!(b.lights[i].len(), 1, "{}: its lamp", def.name);
        igloos += 1;
    }
    assert_eq!(igloos, 5);
}

/// Slice 4g: an igloo's inside is its dome's air, so its candle lights it
/// (design, "Finding: the igloo's candle lit nothing"): every face cut as
/// its room's has its air under the dome, the dome's whole inside is among
/// them, and so is a floor of snow over the ground.
#[test]
fn an_igloos_room_is_the_air_under_its_dome() {
    let (template, _, b) = cut_tundra();
    let mut igloos = 0;
    for (i, def) in template.buildings.iter().enumerate() {
        if def.kit != "igloo" {
            continue;
        }
        let s = &b.solids[i];
        let local = |p: &[f32; 3]| s.frame.local(Vec3::from_array(*p));
        let turn = |n: &[f32; 3]| {
            let n = Vec3::from_array(*n);
            Vec3::new(n.dot(s.frame.x), n.dot(s.frame.y), n.dot(s.frame.z))
        };
        // The dome's middle in the frame: the room's light stands in it.
        let lamp = s.frame.local(b.lights[i][0].at);
        let (mut inward, mut floor) = (0, false);
        let mut middle = glam::Vec2::ZERO;
        let room = &b.rooms[i];
        // The dome's centre is the middle of its floor, a 24-gon whose box
        // is square on it.
        if let Some(snow) = room.get("snow") {
            let (lo, hi) = snow.positions.iter().fold(
                (glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MIN)),
                |(lo, hi), p| {
                    let q = local(p);
                    let q = glam::Vec2::new(q.x, q.z);
                    (lo.min(q), hi.max(q))
                },
            );
            middle = (lo + hi) / 2.0;
            floor = snow
                .positions
                .iter()
                .zip(&snow.normals)
                .any(|(p, n)| turn(n).y > 0.9 && local(p).y.abs() < 0.05);
        }
        assert!(floor, "{}: a floor of snow under its dome", def.name);
        for (material, buf) in room {
            for (tri, n) in buf.positions.chunks(3).zip(buf.normals.chunks(3)) {
                let centre = tri.iter().map(local).sum::<Vec3>() / 3.0;
                let n = turn(&n[0]);
                let air = centre + n * 0.05;
                let across = glam::Vec2::new(air.x, air.z).distance(middle) / 2.3;
                let up = air.y.max(0.0) / 2.5;
                // A triangle's middle is not its face's, so a hair over.
                assert!(
                    across * across + up * up < 1.02,
                    "{}: its room's {material} face at {centre} has its air outside the dome",
                    def.name
                );
                let out = glam::Vec2::new(centre.x, centre.z) - middle;
                if material == "snowblock" && out.dot(glam::Vec2::new(n.x, n.z)) < 0.0 {
                    inward += 1;
                }
            }
        }
        assert!(
            inward > 500,
            "{}: {inward} faces of its dome's inside are its room's",
            def.name
        );
        let lamp_off = glam::Vec2::new(lamp.x, lamp.z).distance(middle);
        assert!(lamp_off < 2.0, "{}: its candle under the dome", def.name);
        igloos += 1;
    }
    assert_eq!(igloos, 5);
}

/// Slice 4g: the walker through the ice castle's gate, into its wall, along
/// its walk and up each of its two towers onto it, as the walled town's.
#[test]
fn a_walker_goes_through_the_ice_gate_and_up_a_tower_onto_the_walk() {
    walk_the_walls(&tundra(), 1, 10, 40, 2);
}

fn cut_jungle() -> (Template, record::Town, record::Built) {
    let template = jungle();
    let town = laid_village(&template);
    let (patch, _) = patch();
    let natural = slope();
    let mut b = record::build_town(
        &town,
        Some(&template),
        patch,
        &kits(),
        &|_: &str| 2.0,
        RADIUS_M,
        SHEET_M,
        move |d| natural(d).floor(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    for s in &mut b.solids {
        for d in &mut s.doors {
            d.open = true;
        }
    }
    (template, town, b)
}

/// Slice 4i: the jungle lays with its floor and its stream wild, on one
/// terrace; its footprint is what it builds on or over: its buildings, its
/// platforms, the cells under its bridges, the cells it keeps its trees
/// off and its stilt huts' decks. It cuts whole, and its torches stand on
/// their platforms 8 m up.
#[test]
fn the_jungle_lays_with_its_floor_wild_and_its_platforms_built() {
    let template = jungle();
    assert!(!template.terraced);
    assert_eq!(template.wild, ["The jungle floor", "The stream"]);
    let kits = kits();
    for b in &template.buildings {
        assert!(kits.get(&b.kit).is_some(), "{}: no kit {}", b.name, b.kit);
    }
    assert_eq!(template.kapoks.len(), 3);
    assert_eq!(template.platforms.len(), 3);
    assert_eq!(template.bridges.len(), 3);
    let cell_m = template.grid.cell_m;
    let built: BTreeSet<[i32; 2]> = template.built_cells().into_iter().collect();
    let must = template
        .platforms
        .iter()
        .flat_map(|p| p.cells.iter().copied())
        .chain(template.bridges.iter().flat_map(|b| b.cells(cell_m)))
        .chain(template.cleared.iter().copied())
        .chain(
            template
                .buildings
                .iter()
                .flat_map(|b| b.cells.iter().copied()),
        );
    for c in must {
        assert!(built.contains(&c), "{c:?} is not built");
    }
    for g in &template.ground {
        if g.area == "The stream" {
            assert!(
                !built.contains(&[g.c, g.r]),
                "the stream at ({}, {})",
                g.c,
                g.r
            );
        }
    }
    let wild = template
        .ground
        .iter()
        .filter(|g| !built.contains(&[g.c, g.r]))
        .count();
    println!(
        "the jungle builds {} cells of {}; {wild} keep the planet's ground",
        built.len(),
        template.ground.len()
    );
    assert!(built.len() < 200 && wild > template.ground.len() * 8 / 10);
    let town = laid_village(&template);
    assert!(town.levels.iter().all(|&l| l == 0), "laid on one terrace");
    let (_, _, b) = cut_jungle();
    let spans: usize = template
        .bridges
        .iter()
        .map(|x| ((x.length() / 1.4).round() as usize).max(2))
        .sum();
    assert_eq!(
        b.solids.len(),
        template.buildings.len() + 3 + 3 + spans,
        "the buildings, the kapoks, the platforms and the bridges' spans"
    );
    assert_eq!(b.dressing_skipped, 0);
    let (patch, _) = patch();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, slope()).expect("ground");
    let lamps = record::lamps_of(&town, &template, &chart, patch);
    let terrace = town.terrace as f32;
    let count = |m: crate::terrain::Material, at: f32| {
        lamps
            .iter()
            .filter(|l| l.material == m && (l.altitude_m - terrace - at).abs() < 0.01)
            .count()
    };
    use crate::terrain::Material::{Brazier, LanternHanging, Torch};
    assert_eq!(count(Torch, 8.0), 6, "two torches on each platform");
    assert_eq!(count(Torch, 0.0), 2, "a torch at each stilt hut");
    assert_eq!(count(Brazier, 8.0), 1, "the platform's fire");
    assert_eq!(count(Brazier, 0.0), 1, "the clearing's fire pit");
    let hanging = lamps
        .iter()
        .filter(|l| l.material == LanternHanging)
        .count();
    // One column holds one lamp: the third bridge crosses over the
    // clearing's fire pit, and the lantern over it is the pit's column.
    let lanterns: usize = template.bridges.iter().map(|x| x.lanterns.len()).sum();
    assert_eq!((hanging, lanterns), (8, 9), "the bridges' lanterns");
}

/// Feet on whatever a piece answers within a tread of where they are, the
/// body over them in nothing, along `path` (directions) from `feet`, metres
/// over the radius: where they end up.
fn walk_along(b: &record::Built, path: &[Vec3], mut feet: f32, what: &str) -> f32 {
    let held = |p: Vec3| b.solids.iter().any(|s| s.holds(p, 0.9, 0.3));
    for w in path.windows(2) {
        for k in 1..=60 {
            let d = w[0].lerp(w[1], k as f32 / 60.0).normalize();
            let next = b
                .solids
                .iter()
                .filter_map(|x| x.stand(d * feet, 0.3).0)
                .fold(f32::MIN, f32::max);
            assert!(
                next > feet - 0.3,
                "{what}: fell from {:.2} m to {:.2} m",
                feet - RADIUS_M,
                next - RADIUS_M
            );
            assert!(
                !held(d * (next + 0.95)),
                "{what}: held at {:.2} m",
                next - RADIUS_M
            );
            feet = next;
        }
    }
    feet
}

/// Slice 4i: a walker goes in at the pole tower's door, climbs its newel
/// and steps out onto the first platform 8 m up; walks round the platform
/// to a bridge, and across it, down its sag and up again, to the next
/// platform; and cannot step off a bridge's side.
#[test]
fn a_walker_climbs_the_pole_tower_and_crosses_a_rope_bridge() {
    use pieces::Surface;
    let (template, town, b) = cut_jungle();
    let (patch, _) = patch();
    let natural = slope();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, move |d| natural(d).floor())
        .unwrap_or_else(|e| panic!("{e}"));
    let cell_m = template.grid.cell_m;
    let centre = |c: i32, r: i32| {
        chart
            .cell(c, r)
            .map(|i| patch.cells[i].direction)
            .unwrap_or_else(|| panic!("({c}, {r}) is off the chart"))
    };
    let point = |x: f32, z: f32| {
        sea::point(&chart, patch, x, z, cell_m)
            .unwrap_or_else(|| panic!("({x}, {z}) is off the chart"))
    };
    let terrace = RADIUS_M + town.terrace as f32;
    let datum = record::datum(&template) as f32;
    let deck = terrace + 9.0 - datum + pieces::LIFT_M;
    // A platform is flat in its frame at its middle: 4 m out, on this 300 m
    // body, its floor stands 2.5 cm over the middle's (1.7 mm on the game's
    // 4800 m one).
    let flat = 0.04;
    let held = |p: Vec3| b.solids.iter().any(|s| s.holds(p, 0.9, 0.3));

    // Up the tower's newel, as the walls' test climbs a tower.
    let (k, def) = town
        .buildings
        .iter()
        .enumerate()
        .find(|(_, d)| d.newel.is_some())
        .expect("a tower");
    let s = &b.solids[k];
    let Some(&Surface::Newel {
        centre: c,
        start,
        sense,
        top,
        turn_m,
        ..
    }) = s
        .surfaces
        .iter()
        .find(|x| matches!(x, Surface::Newel { .. }))
    else {
        panic!("no newel");
    };
    let floor = s.frame.origin.length();
    let end = top / turn_m * std::f32::consts::TAU;
    let on = |phi: f32| {
        let a = start + sense * phi;
        let p = c + Vec2::new(a.cos(), a.sin()) * 0.72;
        s.frame.world(Vec3::new(p.x, 0.0, p.y)).normalize()
    };
    let path: Vec<Vec3> = (0..=200).map(|k| on(end * k as f32 / 200.0)).collect();
    let feet = walk_along(&b, &path, floor, "the newel");
    assert!(
        (feet - floor - top).abs() < 0.01,
        "up at {:.2} m",
        feet - floor
    );
    // Out of its doorway onto the platform's cell, and to its middle.
    let newel = def.newel.as_ref().expect("newel");
    let [tc, tr] = def.cells[0];
    let out = neighbour(tc, tr, usize::from(newel.exits[0].0));
    let feet = walk_along(&b, &[on(end), centre(out.0, out.1)], feet, "the doorway");
    assert!(
        (feet - deck).abs() < flat,
        "on the platform at {:.2} m",
        feet - terrace
    );

    // Round the platform's ring to the cell its bridge comes in on.
    let first = template
        .platforms
        .iter()
        .position(|p| p.cells.iter().any(|&[c, r]| (c, r) == out))
        .expect("the tower's platform");
    let platform = &template.platforms[first];
    let huts: BTreeSet<[i32; 2]> = template
        .buildings
        .iter()
        .filter(|b| b.raised)
        .flat_map(|b| b.cells.iter().copied())
        .collect();
    let ring: Vec<[i32; 2]> = platform.cells[1..].to_vec();
    let on_platform = |x: f32, z: f32| {
        let (c, r) = sea::cell_at(x, z, cell_m);
        ring.contains(&[c, r])
    };
    let (bridge, from_start) = template
        .bridges
        .iter()
        .find_map(|x| {
            if on_platform(
                x.from[0] - (x.to[0] - x.from[0]) * 0.01,
                x.from[2] - (x.to[2] - x.from[2]) * 0.01,
            ) {
                Some((x, true))
            } else if on_platform(
                x.to[0] + (x.to[0] - x.from[0]) * 0.01,
                x.to[2] + (x.to[2] - x.from[2]) * 0.01,
            ) {
                Some((x, false))
            } else {
                None
            }
        })
        .expect("a bridge from the tower's platform");
    let t_of = |k: usize| {
        if from_start {
            k as f32 / 100.0
        } else {
            1.0 - k as f32 / 100.0
        }
    };
    let (sx, sz) = bridge.plan(t_of(0));
    let (ex, ez) = bridge.plan(t_of(100));
    // The ring cells, in order round from the doorway's cell to the
    // bridge's, the way that passes no hut.
    let (bc, br) = {
        let (dx, dz) = (sx - (ex - sx) * 0.01, sz - (ez - sz) * 0.01);
        sea::cell_at(dx, dz, cell_m)
    };
    let tree = platform.cells[0];
    let order: Vec<[i32; 2]> = {
        let mut order = vec![[out.0, out.1]];
        // Neighbouring ring cells share an edge; walk them round.
        while order.len() < ring.len() {
            let last = *order.last().unwrap();
            let next = ring.iter().find(|&&x| {
                !order.contains(&x)
                    && (0..6).any(|d| neighbour(last[0], last[1], d) == (x[0], x[1]))
            });
            match next {
                Some(&n) => order.push(n),
                None => break,
            }
        }
        order
    };
    let to = order
        .iter()
        .position(|&x| x == [bc, br])
        .expect("the bridge's cell on the ring");
    let one_way: Vec<[i32; 2]> = order[..=to].to_vec();
    let other_way: Vec<[i32; 2]> = std::iter::once(order[0])
        .chain(order[to..].iter().rev().copied())
        .collect();
    let route = if one_way.iter().all(|c| !huts.contains(c)) {
        one_way
    } else {
        other_way
    };
    assert!(
        route.iter().all(|c| !huts.contains(c) && *c != tree),
        "{route:?}"
    );
    let mut path: Vec<Vec3> = route.iter().map(|&[c, r]| centre(c, r)).collect();
    path.push(point(sx, sz));
    let feet = walk_along(&b, &path, feet, "round the platform");
    assert!(
        (feet - deck).abs() < flat,
        "at the bridge at {:.2} m",
        feet - terrace
    );

    // Across the bridge, the feet on its sagging walk all the way.
    let mut feet = feet;
    let mut lowest = f32::MAX;
    for k in 1..=100 {
        let t = t_of(k);
        let (x, z) = bridge.plan(t);
        let (px, pz) = bridge.plan(t_of(k - 1));
        feet = walk_along(&b, &[point(px, pz), point(x, z)], feet, "the bridge");
        let want = terrace + bridge.height(t) - datum;
        // At its far end the feet are on the far platform's lapped edge.
        let within = if k == 100 { flat } else { 0.03 };
        assert!(
            (feet - want).abs() < within,
            "{:.0}% across: the feet at {:.2} m on a walk at {:.2} m",
            t * 100.0,
            feet - terrace,
            want - terrace
        );
        lowest = lowest.min(feet);
    }
    assert!(
        deck - lowest > bridge.sag_m - 0.05,
        "down the sag: {:.2} m under the deck",
        deck - lowest
    );
    // On into the far platform's cell.
    let far = template
        .platforms
        .iter()
        .find(|p| {
            p.cells.iter().any(|&[c, r]| {
                (c, r) == sea::cell_at(ex + (ex - sx) * 0.01, ez + (ez - sz) * 0.01, cell_m)
            })
        })
        .expect("the far platform");
    let (fc, fr) = sea::cell_at(ex + (ex - sx) * 0.01, ez + (ez - sz) * 0.01, cell_m);
    assert!(far.cells.contains(&[fc, fr]));
    let feet = walk_along(
        &b,
        &[point(ex, ez), centre(fc, fr)],
        feet,
        "onto the far platform",
    );
    assert!(
        (feet - deck).abs() < flat,
        "on the far platform at {:.2} m",
        feet - terrace
    );

    // Off the side of every bridge: the ropes hold the walker in.
    for x in &template.bridges {
        let (dx, dz) = (x.to[0] - x.from[0], x.to[2] - x.from[2]);
        let len = x.length();
        let (nx, nz) = (-dz / len, dx / len);
        for k in 1..10 {
            let t = k as f32 / 10.0;
            let (px, pz) = x.plan(t);
            let feet = terrace + x.height(t) - datum;
            assert!(
                !held(point(px, pz) * (feet + 0.95)),
                "held on the walk at {t}"
            );
            for s in [-1.0f32, 1.0] {
                let off = s * (x.width_m / 2.0 + 0.05);
                let side = point(px + nx * off, pz + nz * off);
                assert!(held(side * (feet + 0.95)), "stepped off the side at {t}");
            }
        }
    }
}

/// Slice 4i: a walker goes into each tree hut through its door from its
/// platform, on the hut's raised floor; and up each stilt hut's porch stair
/// from the jungle floor onto its deck and in at its door.
#[test]
fn a_walker_goes_into_the_tree_huts_and_up_onto_the_stilt_huts() {
    use pieces::Surface;
    let (_template, town, b) = cut_jungle();
    let (patch, _) = patch();
    let natural = slope();
    let (chart, _) = record::ground_of(&town, patch, RADIUS_M, move |d| natural(d).floor())
        .unwrap_or_else(|e| panic!("{e}"));
    let centre = |c: i32, r: i32| {
        chart
            .cell(c, r)
            .map(|i| patch.cells[i].direction)
            .unwrap_or_else(|| panic!("({c}, {r}) is off the chart"))
    };
    let (mut trees, mut stilted) = (0, 0);
    for (def, s) in town.buildings.iter().zip(&b.solids) {
        let floor = s.frame.origin.length() + pieces::LIFT_M;
        let [c, r, d, _] = def.doors[0];
        if def.raised {
            let (oc, or) = neighbour(c, r, d as usize);
            let feet = walk_along(&b, &[centre(oc, or), centre(c, r)], floor, &def.name);
            assert!(
                (feet - floor).abs() < 0.02,
                "{}: in at {:.2} m",
                def.name,
                feet - floor
            );
            trees += 1;
        }
        if def.stilts.is_some() {
            let Some(&Surface::Flight {
                foot,
                dir,
                len,
                base,
                ..
            }) = s
                .surfaces
                .iter()
                .find(|x| matches!(x, Surface::Flight { .. }))
            else {
                panic!("{}: no porch stair", def.name);
            };
            let at = |p: Vec2| s.frame.world(Vec3::new(p.x, 0.0, p.y)).normalize();
            let start = s.frame.origin.length() + base;
            let (oc, or) = neighbour(c, r, d as usize);
            let path = [
                at(foot + dir * 0.05),
                at(foot + dir * len),
                centre(oc, or),
                centre(c, r),
            ];
            let feet = walk_along(&b, &path, start, &def.name);
            assert!(
                (feet - floor).abs() < 0.02,
                "{}: in at {:.2} m over its floor",
                def.name,
                feet - floor
            );
            stilted += 1;
        }
    }
    assert_eq!((trees, stilted), (3, 2));
}

/// Slice 4i: the lamp height rule moves no lamp of a town that already
/// stands. Every lamp of the village, the walled town, the harbour, the
/// desert and the tundra is where the old rule put it: a lamp on a cell of
/// the town at its cell's terrace, one off them at the town's terrace and
/// its height over the datum.
#[test]
fn no_lamp_of_an_older_town_moves() {
    let (patch, _) = patch();
    let towns = [
        (village(), laid_village(&village())),
        (walled(), laid_village(&walled())),
        {
            let (t, town, _) = laid_harbour();
            (t, town)
        },
        (desert(), laid_village(&desert())),
        (tundra(), laid_village(&tundra())),
    ];
    let mut checked = 0;
    for (template, town) in &towns {
        let natural = slope();
        let (chart, _) = record::ground_of(town, patch, RADIUS_M, move |d| natural(d).floor())
            .unwrap_or_else(|e| panic!("{e}"));
        let terrace: std::collections::BTreeMap<(i32, i32), f32> = town
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| ((c.0, c.1), town.terrace_of(i)))
            .collect();
        let datum = record::datum(template) as f32;
        let cell_m = template.grid.cell_m;
        let old = |x: f32, y: f32, z: f32| {
            let (c, r) = sea::cell_at(x, z, cell_m);
            let i = chart.cell(c, r)?;
            let at = terrace
                .get(&(c, r))
                .copied()
                .unwrap_or(town.terrace as f32 + (y - datum).floor());
            Some((patch.cells[i].direction, at))
        };
        let mut want: Vec<(Vec3, f32)> = template
            .lamps
            .iter()
            .filter_map(|&[c, r]| {
                let i = chart.cell(c, r)?;
                let at = terrace.get(&(c, r)).copied().unwrap_or(town.terrace as f32);
                Some((patch.cells[i].direction, at))
            })
            .chain(
                template
                    .lanterns
                    .iter()
                    .filter_map(|l| old(l[0], l[1], l[2])),
            )
            .chain(template.fires.iter().filter_map(|f| old(f.x, f.y, f.z)))
            .collect();
        let mut seen = BTreeSet::new();
        want.retain(|(d, _)| seen.insert((d.x.to_bits(), d.y.to_bits())));
        let lamps = record::lamps_of(town, template, &chart, patch);
        assert_eq!(lamps.len(), want.len(), "{}", template.scene);
        for (l, (d, at)) in lamps.iter().zip(&want) {
            assert_eq!(l.direction, *d, "{}", template.scene);
            assert_eq!(
                l.altitude_m, *at,
                "{}: a lamp moved from {at} m to {} m",
                template.scene, l.altitude_m
            );
            checked += 1;
        }
    }
    println!("{checked} lamps of five towns where they stood");
    assert!(checked >= 80);
}

/// Design, "Finding: a column can take its neighbour's ground": a town of
/// layout 2 finds every column's own cell by the nearest chord; a town
/// stored at layout 1 keeps the dot lookup it was played with, and reads
/// its layout back from its records.
#[test]
fn a_towns_layout_names_how_its_ground_finds_a_cell() {
    let (patch, _) = patch();
    let town = laid_village(&village());
    assert_eq!(town.layout, record::LAYOUT_VERSION);
    assert_eq!(record::LAYOUT_VERSION, 2);
    let mut old = town.clone();
    old.layout = 1;
    let mut store = crate::records::Records::new();
    for r in record::to_records(&old) {
        store.put(r);
    }
    let record::Stored::Town(kept) = record::from_records(&store, old.site) else {
        panic!("the layout-1 town reads back");
    };
    assert_eq!(kept.layout, 1, "a layout-1 town stays layout 1");
    for (t, chord) in [(&town, true), (&old, false)] {
        let (_, g) = record::ground_of(t, patch, RADIUS_M, slope()).expect("ground");
        let cells = g.cells();
        let mut checked = 0;
        for c in cells.iter().filter(|c| c.ring != GroundAt::OUTSIDE) {
            let d = c.centre;
            let nearest = cells
                .iter()
                .enumerate()
                .max_by(|(i, a), (j, b)| {
                    let (x, y) = if chord {
                        (
                            -(a.centre - d).length_squared(),
                            -(b.centre - d).length_squared(),
                        )
                    } else {
                        (a.centre.dot(d), b.centre.dot(d))
                    };
                    x.total_cmp(&y).then(j.cmp(i))
                })
                .map(|(_, n)| n.centre);
            let found = g.at(d).map(|x| x.centre);
            if nearest.is_some_and(|n| {
                cells
                    .iter()
                    .any(|x| x.centre == n && x.ring != GroundAt::OUTSIDE)
            }) {
                assert_eq!(found, nearest, "layout {}: the column at {d:?}", t.layout);
            }
            if chord {
                assert_eq!(found, Some(d), "layout 2 finds its own cell");
            }
            checked += 1;
        }
        assert!(checked > 500);
    }
}
