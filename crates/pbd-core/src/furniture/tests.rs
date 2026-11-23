use super::store::{STORE_KIND, Store, StoreBody, Trade, item_name, item_of, loot};
use super::*;
use crate::goods::Good;
use crate::inventory::{Item, Slots, Stack};
use crate::records::Records;
use glam::DVec3;

/// Design 4: every piece fits against every side of the standard cell and
/// in its middle, clear of the walls.
#[test]
fn every_piece_fits_against_every_side() {
    for kind in Kind::ALL {
        for side in 0..=MIDDLE {
            assert!(fits(&footprint(kind, side)), "{kind:?} at {side}");
        }
    }
}

/// Design 3: a table in the middle with a chair on every side, and a bed
/// with a chest across from it, share one cell; two pieces on one side, or a
/// bed beside a closet, do not.
#[test]
fn pieces_share_a_cell_without_crossing() {
    let table = footprint(Kind::Table, MIDDLE);
    for side in 0..6 {
        assert!(
            !overlaps(&table, &footprint(Kind::Chair, side)),
            "chair {side}"
        );
    }
    assert!(!overlaps(
        &footprint(Kind::Bed, 0),
        &footprint(Kind::Chest, 3)
    ));
    assert!(overlaps(
        &footprint(Kind::Bed, 0),
        &footprint(Kind::Table, MIDDLE)
    ));
    assert!(overlaps(
        &footprint(Kind::Bed, 0),
        &footprint(Kind::Closet, 1)
    ));
    assert!(overlaps(
        &footprint(Kind::Chest, 2),
        &footprint(Kind::Crate, 2)
    ));
}

/// Design 4: nothing stands in the clear space inside a door, and a piece
/// across the room from the door does not.
#[test]
fn a_door_keeps_its_way_clear() {
    for kind in Kind::ALL {
        assert!(in_doorway(kind, 2, &[2]), "{kind:?} against the door");
    }
    assert!(!in_doorway(Kind::Chest, 5, &[2]));
    assert!(!in_doorway(Kind::Chair, 0, &[3]));
    assert!(
        in_doorway(Kind::Bed, 0, &[3]),
        "a bed's foot reaches the door"
    );
}

/// Design 7: a place holds one body, a body one place, a bench two.
#[test]
fn one_body_to_a_place() {
    let mut o = Occupancy::default();
    let a = Who::Person { site: 1, index: 0 };
    let b = Who::Person { site: 1, index: 1 };
    assert_eq!(o.claim((10, 0), a), Ok(()));
    assert_eq!(o.claim((10, 0), Who::Player), Err(a));
    assert_eq!(o.claim((10, 0), a), Ok(()));
    // A new claim lets the last go.
    assert_eq!(o.claim((11, 0), a), Ok(()));
    assert_eq!(o.holder((10, 0)), None);
    assert_eq!(o.held_by(a), Some((11, 0)));
    // A bench seats two, and no third.
    assert_eq!(o.free_place(20, Kind::Bench.places()), Some(0));
    o.claim((20, 0), b).unwrap();
    assert_eq!(o.free_place(20, Kind::Bench.places()), Some(1));
    o.claim((20, 1), Who::Player).unwrap();
    assert_eq!(o.free_place(20, Kind::Bench.places()), None);
    o.release(Who::Player);
    assert_eq!(o.free_place(20, Kind::Bench.places()), Some(1));
    assert_eq!(Kind::Table.places(), 0);
    assert_eq!(Kind::Bed.places(), 1);
}

/// Design 7: by night two people wanting one bed take one each when there
/// are two, and one goes without when there is one; the player's place is
/// never taken.
#[test]
fn the_schedule_never_doubles_a_bed() {
    let a = Who::Person { site: 3, index: 0 };
    let b = Who::Person { site: 3, index: 1 };
    let here = DVec3::new(0.0, 0.0, 300.0);
    let bed = |id: u64, dx: f64| Spot {
        id,
        kind: Kind::Bed,
        at: here + DVec3::new(dx, 0.0, 0.0),
    };
    let people = [(a, here, Want::Bed), (b, here, Want::Bed)];
    let mut o = Occupancy::default();
    let one = assign(&people, &[bed(1, 1.0)], &mut o, 20.0);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].0, a);
    let mut o = Occupancy::default();
    let two = assign(&people, &[bed(1, 1.0), bed(2, 3.0)], &mut o, 20.0);
    assert_eq!(two.len(), 2);
    assert_ne!(two[0].1.id, two[1].1.id);
    // The player lies in the near bed: the townsfolk leave it.
    let mut o = Occupancy::default();
    o.claim((1, 0), Who::Player).unwrap();
    let left = assign(&people, &[bed(1, 1.0), bed(2, 3.0)], &mut o, 20.0);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].1.id, 2);
    assert_eq!(o.holder((1, 0)), Some(Who::Player));
    // Run again from where it ended, and it ends the same.
    let again = assign(&people, &[bed(1, 1.0), bed(2, 3.0)], &mut o, 20.0);
    assert_eq!(again, left);
    // Out of reach, nobody goes.
    let mut o = Occupancy::default();
    assert!(assign(&people, &[bed(1, 50.0)], &mut o, 20.0).is_empty());
}

#[test]
fn who_wants_what() {
    assert_eq!(want(true, false, false, false, 0), Want::Bed);
    assert_eq!(want(true, true, false, false, 0), Want::Nothing);
    assert_eq!(want(false, false, true, false, 1), Want::Seat);
    assert_eq!(want(false, false, false, true, 3), Want::Seat);
    assert_eq!(want(false, false, false, true, 4), Want::Nothing);
    assert!(keeps_watch("wall_guard") && keeps_watch("lookout"));
    assert!(!keeps_watch("farmer"));
}

/// Design 9: what a store holds survives its record, a fish included, and a
/// name this build lacks is left out without breaking the rest.
#[test]
fn a_store_round_trips_through_its_record() {
    let mut s = Store::empty(8);
    s.give(Stack::new(Item::Good(Good::PLANKS), 5));
    s.give(Stack::new(Item::Fish(3), 2));
    s.coins = 12;
    let mut records = Records::new();
    records.put(s.record(77));
    let back = Store::of(&records, 77, Kind::Crate, || Store::empty(8));
    assert_eq!(back, s);
    // A later build's good, unknown here, is not read into a slot.
    let body = StoreBody {
        slots: vec![Some(("dragon-egg".into(), 1)), Some(("planks".into(), 2))],
        coins: 0,
    };
    let s = Store::from_body(&body, 8);
    assert_eq!(s.slots[0], None);
    assert_eq!(s.slots[1], Some(Stack::new(Item::Good(Good::PLANKS), 2)));
    assert_eq!(item_of(&item_name(Item::Fish(9))), Some(Item::Fish(9)));
    assert_eq!(records.of_kind(STORE_KIND).count(), 1);
}

/// Design 11: a stack moves whole or by half each way, and what does not
/// fit stays where it was.
#[test]
fn stacks_move_both_ways() {
    let mut carried = Slots::new();
    carried.give(Item::Good(Good::PLANKS), 10);
    let index = (0..crate::inventory::CARRIED)
        .find(|&i| carried.get(i).is_some())
        .unwrap();
    let mut chest = Store::empty(Kind::Chest.slots().unwrap());
    assert_eq!(chest.put_from(&mut carried, index, true), 5);
    assert_eq!(carried.count(Item::Good(Good::PLANKS)), 5);
    assert_eq!(chest.put_from(&mut carried, index, false), 5);
    assert_eq!(carried.count(Item::Good(Good::PLANKS)), 0);
    assert_eq!(
        chest.slots[0],
        Some(Stack::new(Item::Good(Good::PLANKS), 10))
    );
    assert_eq!(chest.take_into(0, &mut carried, false), 10);
    assert!(chest.is_empty());
    // A full crate takes nothing more.
    let mut crate_ = Store::empty(1);
    crate_.give(Stack::new(Item::Good(Good::COAL), 99));
    assert_eq!(crate_.put_from(&mut carried, index, false), 0);
    assert_eq!(carried.count(Item::Good(Good::PLANKS)), 10);
}

/// Design 9: loot is fixed by its container's id, fits its trade, and fits
/// its container.
#[test]
fn loot_is_seeded_and_fits_its_trade() {
    let id = town_piece_id(4, 2, 1);
    assert_eq!(
        loot(id, Trade::Farmer, Kind::Chest),
        loot(id, Trade::Farmer, Kind::Chest)
    );
    let farm = ["wheat-seed", "turnip-seed", "wheat", "turnip", "fibre"];
    let common = ["planks", "rope", "fibre", "log", "stone"];
    let mut coins = 0;
    let mut differ = false;
    for n in 0..200u32 {
        let id = town_piece_id(4, n, 0);
        let l = loot(id, Trade::Farmer, Kind::Chest);
        assert!(l.slots.len() == 15);
        let stacks: Vec<_> = l.slots.iter().flatten().collect();
        assert!(!stacks.is_empty() && stacks.len() <= 4, "{stacks:?}");
        for s in &stacks {
            let name = item_name(s.item);
            assert!(
                farm.contains(&name.as_str()) || common.contains(&name.as_str()),
                "{name}"
            );
        }
        coins += usize::from(l.coins > 0);
        differ |= l != loot(town_piece_id(4, n + 1, 0), Trade::Farmer, Kind::Chest);
    }
    assert!(differ);
    assert!(
        (20..90).contains(&coins),
        "{coins} of 200 farmers' chests hold coins"
    );
    let traders = (0..50)
        .filter(|&n| loot(town_piece_id(1, n, 0), Trade::Trader, Kind::Crate).coins > 0)
        .count();
    assert_eq!(traders, 50, "a trader's chest always holds coins");
    assert_eq!(Trade::of_role("smith"), Trade::Smith);
    assert_eq!(Trade::of_role("net_mender"), Trade::Fisher);
    assert_eq!(Trade::of_role("townsman"), Trade::Common);
}

/// Every piece is a good, by its code, and a recipe in `recipes.ron` at the
/// carpenter's and nowhere else; the bench itself is made at the workbench.
#[test]
fn every_piece_is_made_at_the_carpenter_only() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/recipes.ron"
    ))
    .unwrap();
    let file: crate::station::RecipeFile = ron::from_str(&text).expect("recipes.ron");
    let book = file.resolve().expect("resolves");
    for kind in Kind::ALL {
        assert_eq!(Kind::of_good(kind.good()), Some(kind));
        let makers: Vec<_> = book
            .recipes
            .iter()
            .filter(|r| {
                r.outputs
                    .iter()
                    .any(|(item, _)| *item == Item::Good(kind.good()))
            })
            .map(|r| r.station.code)
            .collect();
        assert_eq!(makers, ["carpenter"], "{kind:?}");
    }
    let bench: Vec<_> = book
        .recipes
        .iter()
        .filter(|r| {
            r.outputs
                .iter()
                .any(|(item, _)| *item == Item::Block(crate::terrain::Material::Carpenter))
        })
        .map(|r| r.station.code)
        .collect();
    assert_eq!(bench, ["workbench"]);
    assert_eq!(
        crate::station::station_of(crate::terrain::Material::Carpenter).map(|s| s.code),
        Some("carpenter")
    );
}

/// Design 2: every model is closed faces in the town's textures, inside its
/// footprint and its height, with no face that has no area.
#[test]
fn every_model_is_shaped_inside_its_footprint() {
    let textures = [
        model::FRAME,
        model::TOP,
        model::BOARDS,
        model::SHEET,
        model::BLANKET,
        model::IRON,
    ];
    for kind in Kind::ALL {
        let polys = model::polys(kind);
        assert!(polys.len() > 40, "{kind:?}: {} faces", polys.len());
        let s = kind.size_m();
        for p in &polys {
            assert!(textures.contains(&p.texture));
            assert!(p.points.len() >= 3);
            assert!(p.normal().length() > 0.5, "{kind:?}: a face with no area");
            for q in &p.points {
                assert!(q.x.abs() <= s.x / 2.0 + 0.03, "{kind:?} {q}");
                assert!(q.z.abs() <= s.y / 2.0 + 0.03, "{kind:?} {q}");
                assert!(
                    q.y >= -0.01 && q.y <= kind.height_m() + 0.03,
                    "{kind:?} {q}"
                );
            }
        }
        // Bevelled: no two faces meet at a right angle along a box's edge
        // without one between them, so the top face's normal is never the
        // only one looking up among faces touching the top.
        let tops = polys.iter().filter(|p| p.normal().y > 0.99).count();
        let slopes = polys
            .iter()
            .filter(|p| (0.3..0.95).contains(&p.normal().y))
            .count();
        assert!(slopes >= tops, "{kind:?}: {tops} tops, {slopes} bevels");
    }
}

/// A measurement instrument for the mockup (`furniture` task 1.2): writes
/// every model's faces to `PBD_FURNITURE_EXPORT`, so M15's Furniture scene
/// draws the game's own pieces. Run with `--ignored`.
#[test]
#[ignore]
fn export_models_for_the_mockup() {
    let Ok(path) = std::env::var("PBD_FURNITURE_EXPORT") else {
        return;
    };
    let r = |v: f32| (v * 1000.0).round() / 1000.0;
    let polys_json = |polys: Vec<model::Poly>| -> String {
        let faces: Vec<String> = polys
            .iter()
            .map(|p| {
                let pts: Vec<String> = p
                    .points
                    .iter()
                    .map(|q| format!("{},{},{}", r(q.x), r(q.y), r(q.z)))
                    .collect();
                format!("[\"{}\",[{}]]", p.texture, pts.join(","))
            })
            .collect();
        format!("[{}]", faces.join(","))
    };
    let mut parts: Vec<String> = Kind::ALL
        .iter()
        .map(|&k| {
            let s = k.size_m();
            format!(
                "\"{}\":{{\"size\":[{},{}],\"height\":{},\"solid\":{},\"faces\":{}}}",
                k.code(),
                s.x,
                s.y,
                k.height_m(),
                k.solid(),
                polys_json(model::polys(k))
            )
        })
        .collect();
    parts.push(format!(
        "\"carpenter-top\":{{\"faces\":{}}}",
        polys_json(model::carpenter_top())
    ));
    std::fs::write(&path, format!("{{{}}}\n", parts.join(","))).unwrap();
}

/// Design 9 and the migration plan: a world saved before furniture holds no
/// `store` record. Opening a town chest in it shows its loot and writes
/// nothing; once a stack moves, the chest's record is what it holds, and the
/// loot never comes back (no refill).
#[test]
fn an_old_world_opens_a_town_chest_without_writing_and_taken_stays_taken() {
    let old = Records::new();
    let id = town_piece_id(7, 3, 1);
    let opened = Store::of(&old, id, Kind::Chest, || {
        loot(id, Trade::Farmer, Kind::Chest)
    });
    assert_eq!(opened.slots.len(), 15);
    assert!(!opened.is_empty());
    assert!(old.is_empty(), "opening writes nothing");
    // Take everything: the record holds an empty chest.
    let mut carried = Slots::new();
    let mut chest = opened.clone();
    for k in 0..chest.slots.len() {
        chest.take_into(k, &mut carried, false);
    }
    chest.take_coins();
    let mut records = Records::new();
    records.put(chest.record(id));
    let again = Store::of(&records, id, Kind::Chest, || {
        loot(id, Trade::Farmer, Kind::Chest)
    });
    assert!(again.is_empty(), "a town chest does not refill");
    assert_eq!(records.of_kind(STORE_KIND).count(), 1);
}
