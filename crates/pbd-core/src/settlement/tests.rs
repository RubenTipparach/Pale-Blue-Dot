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
