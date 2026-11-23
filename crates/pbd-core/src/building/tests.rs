use super::blueprint::{blueprint, fit_roof, plain, room_at};
use super::cut::cut;
use super::rules::{Rule, can_place, check_layout, dependant};
use super::*;
use crate::records::Records;
use crate::settlement::chart::Patch;
use crate::settlement::{Kits, Template};
use crate::topology::dual_sphere;
use glam::{Vec2, Vec3};
use std::sync::OnceLock;

/// The gold-standard body, as the settlement tests cut on.
const RADIUS_M: f32 = 300.0;
/// The ground's first air layer everywhere on the test plot: 5 m over the
/// sea.
const GROUND: u16 = 150;

fn kits() -> Kits {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/kits.ron"
    ))
    .expect("kits.ron");
    ron::from_str(&text).expect("kits.ron parses")
}

fn template(name: &str) -> Template {
    let path = format!(
        "{}/../../assets/settlements/v1/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).expect("template")).expect("parses")
}

/// A patch of the level-7 sphere round a cell away from the pentagons, keyed
/// by index; and one round a pentagon.
fn patch_at(anchor: Vec3) -> (Patch, usize) {
    let cells = dual_sphere(7);
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
    let reach = (0.25f32).cos();
    let kept: Vec<usize> = (0..cells.len())
        .filter(|&i| cells[i].direction.dot(cells[centre].direction) > reach)
        .collect();
    let mut index = vec![usize::MAX; cells.len()];
    for (k, &i) in kept.iter().enumerate() {
        index[i] = k;
    }
    let local = kept
        .iter()
        .map(|&i| {
            let mut c = cells[i].clone();
            c.neighbors = c.neighbors.iter().map(|&n| index[n]).collect();
            c
        })
        .collect();
    let keys = kept.iter().map(|&i| i as u32).collect();
    (Patch { cells: local, keys }, index[centre])
}

fn plot() -> &'static (Patch, usize) {
    static P: OnceLock<(Patch, usize)> = OnceLock::new();
    P.get_or_init(|| patch_at(Vec3::new(0.3, 0.5, 0.8).normalize()))
}

fn flat(_: u32) -> Option<u16> {
    Some(GROUND)
}

/// The key of the patch cell at `i`, and its neighbours' keys.
fn key(i: usize) -> u32 {
    plot().0.keys[i]
}

fn nb(cells: &Cells, k: u32, s: usize) -> u32 {
    cells.neighbours(k)[s].expect("on the patch")
}

/// A room of one cell walled all round at `layer`, floored.
fn hut(cells: &Cells, k: u32, layer: u16) -> Vec<Piece> {
    let mut out = vec![Piece::new("timber", Shape::Floor { cell: k, layer })];
    for s in 0..6 {
        out.push(Piece::new(
            "timber",
            Shape::wall(k, nb(cells, k, s), layer, Opening::None),
        ));
    }
    out
}

#[test]
fn a_wall_seen_from_either_cell_is_one_slot() {
    let cells = Cells::new(&plot().0);
    let a = key(plot().1);
    let b = nb(&cells, a, 2);
    let one = Shape::wall(a, b, GROUND, Opening::None);
    let other = Shape::wall(b, a, GROUND, Opening::Door);
    assert_eq!(one.slot(), other.slot());
    assert_eq!(one.id(), other.id());
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let built = [Piece::new("timber", one)];
    assert_eq!(
        can_place(&site, &built, &Piece::new("timber", other)),
        Err(Rule::Slot)
    );
}

/// Task 2.2: every template building the pieces can make is accepted whole,
/// and piece by piece in the order a player would build it.
#[test]
fn every_plain_template_building_is_accepted_whole_and_piece_by_piece() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let mut built = 0;
    for name in ["village", "town", "coast", "desert", "tundra", "jungle"] {
        let t = template(name);
        for def in &t.buildings {
            let Some(kit) = kits.get(&def.kit) else {
                continue;
            };
            if !plain(def, kit) {
                continue;
            }
            let pieces = blueprint(patch, def, kit, *at, 0, GROUND)
                .unwrap_or_else(|e| panic!("{name}: {}: {e}", def.name));
            if let Err((i, rule)) = check_layout(&site, &pieces) {
                panic!(
                    "{name}: {} whole: {:?} breaks {rule:?}",
                    def.name, pieces[i].shape
                );
            }
            let mut so_far: Vec<Piece> = Vec::new();
            for piece in pieces {
                if let Err(rule) = can_place(&site, &so_far, &piece) {
                    panic!(
                        "{name}: {} piece by piece: {:?} refused by {rule:?}",
                        def.name, piece.shape
                    );
                }
                so_far.push(piece);
            }
            built += 1;
        }
    }
    assert!(built >= 10, "only {built} buildings tried");
}

/// Every turn of a blueprint is checked whole too.
#[test]
fn a_blueprint_turns_by_sixths_and_still_holds() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let t = template("village");
    let def = &t.buildings[1];
    let kit = kits.get(&def.kit).unwrap();
    for turn in 0..6 {
        let pieces = blueprint(patch, def, kit, *at, turn, GROUND).unwrap();
        assert_eq!(check_layout(&site, &pieces), Ok(()), "turn {turn}");
    }
}

/// One test per rule: each refused with that rule's name.
#[test]
fn each_rule_refuses_with_its_own_name() {
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let k = key(*at);
    let p = |shape: Shape| Piece::new("timber", shape);
    let refused = |built: &[Piece], piece: Shape| can_place(&site, built, &p(piece)).err();

    // Off the plot.
    assert_eq!(
        refused(
            &[],
            Shape::Floor {
                cell: u32::MAX - 1,
                layer: GROUND
            }
        ),
        Some(Rule::Outside)
    );
    // The slot.
    let floor = Shape::Floor {
        cell: k,
        layer: GROUND,
    };
    assert_eq!(
        refused(&[p(floor.clone())], floor.clone()),
        Some(Rule::Slot)
    );
    // Held up: a floor a storey up with nothing under it.
    let up = Shape::Floor {
        cell: k,
        layer: GROUND + STOREY,
    };
    assert_eq!(refused(&[], up.clone()), Some(Rule::HeldUp));
    assert_eq!(refused(&hut(&cells, k, GROUND), up), None);
    // Headroom: a floor in a stair's well.
    let head = nb(&cells, k, 0);
    let stair = Shape::Stair {
        foot: k,
        head,
        layer: GROUND,
    };
    let mut house = hut(&cells, k, GROUND);
    house.push(p(Shape::Floor {
        cell: head,
        layer: GROUND,
    }));
    house.push(p(stair.clone()));
    let over = Shape::Floor {
        cell: head,
        layer: GROUND + STOREY,
    };
    assert_eq!(refused(&house, over), Some(Rule::Headroom));
    // A wall across the flight.
    let across = Shape::wall(k, head, GROUND, Opening::None);
    let mut open = vec![
        p(Shape::Floor {
            cell: k,
            layer: GROUND,
        }),
        p(Shape::Floor {
            cell: head,
            layer: GROUND,
        }),
        p(stair.clone()),
    ];
    assert_eq!(refused(&open, across), Some(Rule::StairCut));
    // The landing walled at the top.
    let on = cells.straight_on(k, head).unwrap();
    open.push(p(Shape::wall(head, on, GROUND, Opening::None)));
    assert_eq!(
        refused(
            &open,
            Shape::wall(head, on, GROUND + STOREY, Opening::Window)
        ),
        Some(Rule::Landing)
    );
    // A roof over cells no walls close round.
    assert_eq!(
        refused(
            &[p(Shape::Floor {
                cell: k,
                layer: GROUND
            })],
            Shape::roof(vec![k], GROUND + STOREY, RoofShape::Cone)
        ),
        Some(Rule::Room)
    );
    // Two roofs, their eaves into each other.
    let mut two = hut(&cells, k, GROUND);
    let next = nb(&cells, k, 3);
    two.extend(hut(&cells, next, GROUND));
    two.push(p(Shape::roof(vec![k], GROUND + STOREY, RoofShape::Cone)));
    assert_eq!(
        refused(
            &two,
            Shape::roof(vec![next], GROUND + STOREY, RoofShape::Cone)
        ),
        Some(Rule::RoofOverlap)
    );
    // A lantern on a stair.
    assert_eq!(
        refused(
            &house,
            Shape::Lantern {
                cell: head,
                layer: GROUND + 1
            }
        ),
        Some(Rule::HeldUp)
    );
    assert_eq!(
        refused(
            &house,
            Shape::Lantern {
                cell: k,
                layer: GROUND
            }
        ),
        Some(Rule::Clear)
    );
}

#[test]
fn nothing_is_built_on_a_pentagon_or_beside_one() {
    // Round the icosahedron's vertex at +Y-ish: the cell there is a
    // pentagon.
    let (patch, at) = patch_at(Vec3::new(0.0, 1.0, 1.618_034).normalize());
    let cells = Cells::new(&patch);
    let k = patch.keys[at];
    assert!(cells.pentagon(k), "the anchor is a pentagon");
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let refused = |cell: u32| {
        can_place(
            &site,
            &[],
            &Piece::new(
                "timber",
                Shape::Floor {
                    cell,
                    layer: GROUND,
                },
            ),
        )
        .err()
    };
    assert_eq!(refused(k), Some(Rule::Pentagon));
    let beside = cells.neighbours(k)[0].unwrap();
    assert_eq!(refused(beside), Some(Rule::Pentagon));
}

/// Task 3.2: a refused removal names what depends on it.
#[test]
fn removing_a_floor_under_a_lantern_names_the_lantern() {
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let k = key(*at);
    let mut house = hut(&cells, k, GROUND);
    let lantern = Piece::new(
        "timber",
        Shape::Lantern {
            cell: k,
            layer: GROUND,
        },
    );
    house.push(lantern.clone());
    let floor = house[0].clone();
    assert_eq!(dependant(&house, &floor), Some(&lantern));
    let roof = Piece::new(
        "timber",
        Shape::roof(vec![k], GROUND + STOREY, RoofShape::Cone),
    );
    house.push(roof.clone());
    let wall = house[1].clone();
    assert_eq!(dependant(&house, &wall), Some(&roof));
    assert_eq!(dependant(&house, &lantern), None);
}

fn two_by_four(cells: &Cells, k: u32) -> Vec<u32> {
    // Two rows of four: four along side 0, and the four across side 1 of
    // each.
    let mut row = vec![k];
    for _ in 0..3 {
        let last = *row.last().unwrap();
        row.push(nb(cells, last, 0));
    }
    let mut other: Vec<u32> = row.iter().map(|&c| nb(cells, c, 1)).collect();
    row.append(&mut other);
    row.sort_unstable();
    row.dedup();
    row
}

fn walled(cells: &Cells, room: &[u32], layer: u16) -> Vec<Piece> {
    let mut out = Vec::new();
    for &c in room {
        out.push(Piece::new("timber", Shape::Floor { cell: c, layer }));
        for n in cells.neighbours(c).into_iter().flatten() {
            if !room.contains(&n) {
                out.push(Piece::new(
                    "timber",
                    Shape::wall(c, n, layer, Opening::None),
                ));
            }
        }
    }
    out
}

/// Task 4.3: the roof tool finds a two-by-four room and fits a gable over
/// it that passes the overlap check.
#[test]
fn the_roof_tool_roofs_a_two_by_four_house() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let k = key(*at);
    let room = two_by_four(&cells, k);
    assert_eq!(room.len(), 8);
    let house = walled(&cells, &room, GROUND);
    let mut found = room_at(&site, &house, room[3], GROUND).expect("closed");
    found.sort_unstable();
    assert_eq!(found, room);
    let roof = Piece::new(
        "timber",
        fit_roof(kits.get("timber").unwrap(), found, GROUND + STOREY),
    );
    assert!(matches!(
        roof.shape,
        Shape::Roof {
            shape: RoofShape::Gable,
            ..
        }
    ));
    assert_eq!(can_place(&site, &house, &roof), Ok(()));
    // Open a side: no room.
    let out = cells
        .neighbours(k)
        .into_iter()
        .flatten()
        .find(|n| !room.contains(n))
        .expect("an outer side");
    let gap: Vec<Piece> = house
        .iter()
        .filter(|p| p.shape != Shape::wall(k, out, GROUND, Opening::None))
        .cloned()
        .collect();
    assert_eq!(room_at(&site, &gap, k, GROUND), None);
}

/// Task 3.1 in the core: a piece's record round trips, and a removed piece
/// is no longer built.
#[test]
fn a_piece_record_round_trips_and_a_removed_one_is_gone() {
    let piece = Piece::new(
        "brick",
        Shape::Wall {
            a: 3,
            b: 9,
            layer: 160,
            opening: Opening::Window,
        },
    );
    let mut records = Records::new();
    let line = placed(&piece).line(&crate::records::Author::Player(0));
    let (_, back) = crate::records::Record::parse_line(&line).expect("parses");
    records.put(back);
    let got = from_records(&records);
    assert_eq!(got.get(&piece.id()), Some(&piece));
    records.put(removed(&piece));
    assert!(from_records(&records).is_empty());
}

/// Task 1.1's test in the core: a two-storey house built piece by piece, its
/// stair climbed with no jump over 0.1 m at 5 cm steps, and its upper floor
/// stood on.
#[test]
fn a_two_storey_house_cuts_and_its_stair_climbs_without_a_jump() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let t = template("village");
    let def = t
        .buildings
        .iter()
        .find(|b| b.storeys == 2)
        .expect("a two-storey house");
    let kit = kits.get(&def.kit).unwrap();
    let pieces = blueprint(patch, def, kit, *at, 0, GROUND).unwrap();
    let all: Vec<&Piece> = pieces.iter().collect();
    let anchor = key(*at);
    let c = cut(&cells, anchor, &all, &kits, &|_| 2.0, RADIUS_M).expect("cut");
    assert!(c.meshes.len() >= 4, "{:?}", c.meshes.keys());
    let b = &c.solids;
    let Some(super::cut::FLAME) = c.meshes.keys().find(|k| *k == "flame").map(String::as_str)
    else {
        panic!("a lantern's flame");
    };
    let flight = b
        .surfaces
        .iter()
        .find_map(|s| match s {
            crate::settlement::pieces::Surface::Flight {
                foot,
                dir,
                len,
                base,
                ..
            } => Some((*foot, *dir, *len, *base)),
            _ => None,
        })
        .expect("a flight");
    let (foot, dir, len, base) = flight;
    let mut feet = base;
    let steps = (len / 0.05) as usize;
    for i in 0..=steps {
        let p = foot + dir * (i as f32 * 0.05);
        let (floor, _) = b.stand_in(Vec3::new(p.x, feet + 0.05, p.y), 0.35);
        let floor = floor.expect("on the flight");
        assert!(
            (floor - feet).abs() <= 0.1,
            "a jump of {} m at {i}",
            floor - feet
        );
        feet = floor;
    }
    assert!(feet > base + 2.9, "climbed {} m", feet - base);
    // The walls hold a walker in, and the upper floor holds one up.
    let some_floor = b
        .surfaces
        .iter()
        .filter_map(|s| match s {
            crate::settlement::pieces::Surface::Floor { outline, top, .. } => {
                Some((outline[0] + outline[3]) * 0.5).map(|m: Vec2| (m, *top))
            }
            _ => None,
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    assert!(some_floor.1 > base + 2.9);
    assert!(!b.solids.is_empty() && b.top_m > base + 5.9);
}

/// Task 5.1: `can_place` timed over a big building's pieces, every piece
/// checked against all the rest, as the ghost does each frame.
#[test]
fn time_can_place_over_the_walled_towns_biggest_building() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let t = template("town");
    let (def, kit) = t
        .buildings
        .iter()
        .filter_map(|b| Some((b, kits.get(&b.kit)?)))
        .filter(|(b, k)| plain(b, k))
        .max_by_key(|(b, _)| b.cells.len() * b.storeys as usize)
        .expect("a plain building");
    let pieces = blueprint(patch, def, kit, *at, 0, GROUND).unwrap();
    let start = std::time::Instant::now();
    let mut checks = 0;
    for piece in &pieces {
        let others: Vec<Piece> = pieces.iter().filter(|p| *p != piece).cloned().collect();
        let _ = can_place(&site, &others, piece);
        checks += 1;
    }
    let each = start.elapsed().as_secs_f64() / f64::from(checks) * 1e6;
    println!(
        "can_place: {} pieces of {}, {each:.1} us a check (debug build)",
        pieces.len(),
        def.name
    );
}

/// `player-building` design 8: a house whose ground storey stands two
/// layers over its ground is cut with a fieldstone footing under every
/// ground floor, solid from the ground to the slab, and none under the
/// upper storey's floors; on ground at its own layer it has none.
#[test]
fn a_floor_over_lower_ground_stands_on_a_footing() {
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let t = template("village");
    let def = t
        .buildings
        .iter()
        .find(|b| b.storeys == 2)
        .expect("a two-storey house");
    let kit = kits.get(&def.kit).unwrap();
    let pieces = blueprint(patch, def, kit, *at, 0, GROUND).unwrap();
    let all: Vec<&Piece> = pieces.iter().collect();
    let anchor = key(*at);
    // Its ground floor: the template's base may raise it over GROUND.
    let lowest = pieces
        .iter()
        .filter_map(|p| match p.shape {
            Shape::Floor { layer, .. } => Some(layer),
            _ => None,
        })
        .min()
        .unwrap();
    let ground_floors = pieces
        .iter()
        .filter(|p| matches!(p.shape, Shape::Floor { layer, .. } if layer == lowest))
        .count();
    assert!(ground_floors > 0);
    let low = super::cut::cut_on(&cells, anchor, &all, &kits, &|_| 2.0, RADIUS_M, &|_| {
        Some(lowest - 2)
    })
    .expect("cut");
    assert!(low.meshes.contains_key(super::cut::FOOTING), "a footing");
    let deepest = low
        .solids
        .solids
        .iter()
        .map(|s| s.y0)
        .fold(f32::MAX, f32::min);
    assert!(
        deepest < (lowest - GROUND) as f32 - 1.9 && deepest > (lowest - GROUND) as f32 - 2.5,
        "solid down to the ground two layers under: {deepest}"
    );
    let level = super::cut::cut_on(&cells, anchor, &all, &kits, &|_| 2.0, RADIUS_M, &|_| {
        Some(lowest)
    })
    .expect("cut");
    // One solid footing a ground floor, and none on level ground or with no
    // ground known (the kit's own walls may be fieldstone too).
    assert_eq!(
        low.solids.solids.len(),
        level.solids.solids.len() + ground_floors,
        "a footing a ground floor"
    );
    let plain = cut(&cells, anchor, &all, &kits, &|_| 2.0, RADIUS_M).expect("cut");
    assert_eq!(plain.solids.solids.len(), level.solids.solids.len());
    let floor_y = level
        .solids
        .solids
        .iter()
        .map(|s| s.y0)
        .fold(f32::MAX, f32::min);
    assert!(
        floor_y > deepest + 1.5,
        "nothing under the floor on level ground"
    );
}

// ---- `furniture` design 3 and 4: furniture placed by the building rules.

fn furniture(k: u32, side: u8, kind: crate::furniture::Kind) -> Piece {
    Piece::new(
        "timber",
        Shape::Furniture {
            cell: k,
            layer: GROUND,
            side,
            kind,
        },
    )
}

/// A piece stands on the floor of a walled room; a second on its side, one
/// that crosses it, one in a doorway or under a low floor, and one on a
/// stair are refused with their rule.
#[test]
fn furniture_is_placed_by_the_building_rules() {
    use crate::furniture::{Kind, MIDDLE};
    let cells = Cells::new(&plot().0);
    let site = Site {
        cells: &cells,
        ground: &flat,
    };
    let k = key(plot().1);
    let mut room = hut(&cells, k, GROUND);
    let bed = furniture(k, 0, Kind::Bed);
    assert_eq!(can_place(&site, &room, &bed), Ok(()));
    room.push(bed);
    assert_eq!(
        can_place(&site, &room, &furniture(k, 3, Kind::Chest)),
        Ok(())
    );
    assert_eq!(
        can_place(&site, &room, &furniture(k, 0, Kind::Crate)),
        Err(Rule::Slot)
    );
    assert_eq!(
        can_place(&site, &room, &furniture(k, MIDDLE, Kind::Table)),
        Err(Rule::Overlap)
    );
    assert_eq!(
        can_place(&site, &room, &furniture(k, 1, Kind::Closet)),
        Err(Rule::Overlap)
    );
    // A door in side 4's wall: nothing in its way in.
    let door = Shape::wall(k, nb(&cells, k, 4), GROUND, Opening::Door);
    let mut doorway = hut(&cells, k, GROUND);
    doorway.retain(|p| p.shape.slot() != door.slot());
    doorway.push(Piece::new("timber", door.clone()));
    assert_eq!(
        can_place(&site, &doorway, &furniture(k, 1, Kind::Chest)),
        Ok(())
    );
    assert_eq!(
        can_place(&site, &doorway, &furniture(k, 4, Kind::Chest)),
        Err(Rule::Doorway)
    );
    // And a door is refused where a piece already stands in its way.
    let mut blocked = hut(&cells, k, GROUND);
    blocked.push(furniture(k, 4, Kind::Chest));
    blocked.retain(|p| p.shape.slot() != door.slot());
    assert_eq!(
        can_place(&site, &blocked, &Piece::new("timber", door)),
        Err(Rule::Doorway)
    );
    // A floor a metre over a closet leaves it no headroom.
    let low = [Piece::new(
        "timber",
        Shape::Floor {
            cell: k,
            layer: GROUND + 1,
        },
    )];
    assert_eq!(
        can_place(&site, &low, &furniture(k, 2, Kind::Closet)),
        Err(Rule::Headroom)
    );
    // Nothing stands on a stair's cells.
    let stair = [Piece::new(
        "timber",
        Shape::Stair {
            foot: k,
            head: nb(&cells, k, 1),
            layer: GROUND,
        },
    )];
    assert_eq!(
        can_place(&site, &stair, &furniture(k, 2, Kind::Stool)),
        Err(Rule::Clear)
    );
    // With nothing under it, nothing holds it up.
    let air = Site {
        cells: &cells,
        ground: &|_| Some(GROUND - 5),
    };
    assert_eq!(
        can_place(&air, &[], &furniture(k, 2, Kind::Stool)),
        Err(Rule::HeldUp)
    );
}

/// A furniture piece is a `piece` record like any other, and a piece record
/// written before furniture reads as it did.
#[test]
fn furniture_is_a_piece_record_and_old_pieces_read_as_before() {
    use crate::furniture::Kind;
    let k = key(plot().1);
    let old = Piece::new(
        "timber",
        Shape::Floor {
            cell: k,
            layer: GROUND,
        },
    );
    let old_line = serde_json::to_string(&PieceBody {
        piece: Some(old.clone()),
    })
    .unwrap();
    // The floor's body as an older build wrote it: unchanged by this one.
    let want = format!(
        "{{\"piece\":{{\"kit\":\"timber\",\"shape\":{{\"Floor\":{{\"cell\":{k},\"layer\":{GROUND}}}}}}}}}"
    );
    assert_eq!(old_line, want);
    let chest = furniture(k, 2, Kind::BandedChest);
    let mut records = Records::new();
    records.put(placed(&old));
    records.put(placed(&chest));
    let back = from_records(&records);
    assert_eq!(back.get(&old.id()), Some(&old));
    assert_eq!(back.get(&chest.id()), Some(&chest));
    // A piece taken away is gone.
    records.put(removed(&chest));
    assert_eq!(from_records(&records).len(), 1);
}

/// A bed in a room is cut into its linen and wool and stands as a solid; a
/// stool is cut but walked up to, not into.
#[test]
fn furniture_is_cut_with_the_room() {
    use crate::furniture::Kind;
    let kits = kits();
    let (patch, at) = plot();
    let cells = Cells::new(patch);
    let k = key(*at);
    let mut room = hut(&cells, k, GROUND);
    let bare_pieces: Vec<&Piece> = room.iter().collect();
    let bare = cut(&cells, k, &bare_pieces, &kits, &|_| 2.0, RADIUS_M).expect("cut");
    room.push(furniture(k, 0, Kind::Bed));
    room.push(furniture(k, 3, Kind::Stool));
    let all: Vec<&Piece> = room.iter().collect();
    let c = cut(&cells, k, &all, &kits, &|_| 2.0, RADIUS_M).expect("cut");
    assert!(c.meshes.contains_key("linen") && c.meshes.contains_key("wool"));
    assert_eq!(c.solids.solids.len(), bare.solids.solids.len() + 1);
    // The bed stands in its cell, its front a tangent.
    let (bed_at, front) =
        super::cut::furniture_stand(&cells, k, GROUND, 0, Kind::Bed, RADIUS_M).expect("stands");
    let up = bed_at.normalize();
    assert!(front.dot(up).abs() < 1e-3, "a tangent");
    let centre = cells.cell(k).unwrap().direction.normalize() * bed_at.length();
    let off = (bed_at - centre).length();
    assert!((0.05..1.4).contains(&off), "{off} m from the middle");
}
