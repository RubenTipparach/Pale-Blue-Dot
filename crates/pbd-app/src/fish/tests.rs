use super::*;

/// The planet-centred origin: these tests are written in the planet's frame
/// (`world-frames`).
const PLANET: pbd_core::world_frame::SimOrigin =
    pbd_core::world_frame::SimOrigin::new(crate::sim_origin::HOME, bevy::math::DVec3::ZERO);

/// A landed fish is in the hotbar and in the save on the same frame, and the
/// field guide's record counts it.
#[test]
fn a_catch_lands_in_the_hotbar_and_the_save_at_once() {
    let mut hotbar = Hotbar::default();
    let mut save = WorldSave::memory_only();
    let mut status = FishingStatus::default();
    assert!(land(5, 44, Some(&mut hotbar), Some(&mut save), &mut status));
    assert!(land(5, 51, Some(&mut hotbar), Some(&mut save), &mut status));
    let fish: u16 = hotbar
        .iter()
        .flatten()
        .filter(|s| s.item == Item::Fish(5))
        .map(|s| s.count)
        .sum();
    assert_eq!(fish, 2);
    let record = save.catches.get(&5).copied().unwrap_or_default();
    assert_eq!((record.count, record.best_cm), (2, 51));
    assert_eq!(
        save.carried.as_ref(),
        Some(&hotbar.0),
        "the hotbar rode the line"
    );
}

/// A full hotbar and pack let the fish go and say so, rather than losing it
/// silently or recording a catch the player does not have.
#[test]
fn a_full_pack_lets_the_fish_go() {
    let mut hotbar = Hotbar::default();
    for i in 0..pbd_core::inventory::CARRIED {
        hotbar.give(Item::Fish(100 + i as u16), 16);
    }
    let mut save = WorldSave::memory_only();
    let mut status = FishingStatus::default();
    assert!(!land(
        1,
        20,
        Some(&mut hotbar),
        Some(&mut save),
        &mut status
    ));
    assert!(save.catches.is_empty(), "no catch recorded");
    assert!(status.text.contains("let go"), "{}", status.text);
}

/// A bite struck inside its window and reeled patiently ends with the fish
/// in the slots and the save, through the same `report` the game runs; and a
/// fish wound in on a nibble is said to have got away rather than vanishing
/// (owner, 2026-11-16: "I seem to have pulled it in at the right moment, but
/// no fish is added to my inventory").
#[test]
fn a_bite_struck_in_its_window_puts_the_fish_in_the_slots() {
    const R: f32 = 4800.0;
    struct Shore;
    impl Water for Shore {
        fn surface(&self, _point: Vec3) -> f32 {
            R
        }
        fn bed(&self, point: Vec3) -> f32 {
            let z = (point.normalize_or(Vec3::Y) * R).z;
            R - (0.4 * z).clamp(-2.0, 8.0)
        }
    }
    let at = |x: f32, depth: f32, z: f32| Vec3::new(x, R, z).normalize() * (R - depth);
    let feet = at(0.0, -0.4, -1.0);
    let up = feet.normalize();
    let angler = Angler {
        tip: feet + up * 1.9,
        look: (Vec3::Z - up * Vec3::Z.dot(up) + up * 0.1).normalize(),
        feet,
    };
    let fauna = pbd_core::fauna::FaunaSettings::default();
    let roster = fauna.roster(HOME_BODY).to_vec();
    let minnow = roster.iter().position(|s| s.id == "minnow").unwrap();

    // Only the left button hooks; a right click at the bite cancels (owner,
    // 2026-11-18), pinned in pbd_core's `a_right_click_at_the_bite_cancels`.
    for (strike, seed) in [("left", 70), ("left", 71)] {
        let mut line = Line::new(seed);
        let mut schools = vec![School::spawn(
            &PLANET,
            minnow as u16,
            &roster[minnow],
            at(0.0, 1.0, 8.0),
            seed,
            &Shore,
        )];
        let mut hotbar = Hotbar::default();
        let mut save = WorldSave::memory_only();
        let mut status = FishingStatus::default();
        let mut landed = false;
        let mut seen = Vec::new();
        let dt = 1.0 / 30.0;
        for frame in 0..30 * 150 {
            // A player: hold to cast for 0.6 s, watch, strike when the float
            // goes under, then reel and ease off as the tension climbs.
            let controls = match line.phase {
                Phase::Ready if frame == 0 => Controls {
                    pressed: true,
                    held: true,
                    ..Default::default()
                },
                Phase::Charging if line.power < 0.5 => Controls {
                    held: true,
                    ..Default::default()
                },
                Phase::Charging => Controls {
                    released: true,
                    ..Default::default()
                },
                Phase::Bite => Controls {
                    pressed: true,
                    held: true,
                    ..Default::default()
                },
                Phase::Hooked => Controls {
                    held: line.tension < 0.6,
                    ..Default::default()
                },
                _ => Controls::default(),
            };
            let events = line.step(
                &PLANET,
                dt,
                controls,
                &angler,
                &mut schools,
                &roster,
                &fauna.fishing,
                &fauna.flock,
                &Shore,
                25.0,
                1.0,
            );
            let splash = line.splash();
            for school in &mut schools {
                school.step(&PLANET, dt, &roster[minnow], &fauna.flock, &Shore, splash);
            }
            for event in events {
                seen.push(event);
                let did = report(
                    &mut status,
                    event,
                    &roster,
                    Some(&mut hotbar),
                    Some(&mut save),
                    line.float,
                    false,
                );
                if matches!(event, Event::Caught { .. }) {
                    assert!(!did.is_empty(), "a landed fish counts for the missions");
                    landed = true;
                }
            }
            if landed {
                break;
            }
        }
        assert!(landed, "{strike}: never landed: {seen:?}");
        assert!(
            seen.iter().any(|e| matches!(e, Event::Bite { .. })),
            "{seen:?}"
        );
        let fish: u16 = hotbar
            .iter()
            .flatten()
            .filter(|s| s.item == Item::Fish(minnow as u16))
            .map(|s| s.count)
            .sum();
        assert_eq!(fish, 1, "{strike}: the minnow is in the slots");
        assert_eq!(save.catches.get(&(minnow as u16)).map(|r| r.count), Some(1));
    }

    // Wound in on a nibble: the line was not empty, and the player is told.
    let mut status = FishingStatus::default();
    report(
        &mut status,
        Event::Lost {
            species: minnow as u16,
        },
        &roster,
        None,
        None,
        Vec3::ZERO,
        false,
    );
    assert!(status.text.contains("got away"), "{}", status.text);
}

/// A world with no roster for its body has nothing in the water, and the rod
/// says so.
#[test]
fn a_body_with_no_roster_says_nothing_lives_here() {
    let line = Line::new(1);
    assert_eq!(idle_text(&line, true, &[]), "Nothing lives in this water");
    assert!(idle_text(&line, false, &[]).is_empty(), "no rod, no line");
}

fn assets() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"))
}

/// Every species on every body and every tool has an icon on disk: an item
/// with no thumbnail is a blank slot and a blank field-guide entry.
#[test]
fn every_species_and_every_tool_has_an_icon() {
    let fauna = pbd_core::fauna::FaunaSettings::default();
    for (body, roster) in &fauna.bodies {
        for species in roster {
            let path = assets().join(species_icon(species));
            assert!(
                path.exists(),
                "{body}/{}: no icon at {}",
                species.id,
                path.display()
            );
        }
    }
    for tool in Tool::ALL {
        assert!(
            assets().join(tool_icon(tool)).exists(),
            "{tool:?} has no icon"
        );
    }
}

/// The five fish icons copied from Tenebris are the bytes the provenance
/// record says they are: an edit in place fails here until the record moves
/// with it.
#[test]
fn the_copied_tenebris_icons_are_unchanged() {
    let record = std::fs::read_to_string(assets().join("items/fish/PROVENANCE.md")).unwrap();
    let fnv = |bytes: &[u8]| {
        bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    };
    let mut checked = 0;
    for line in record
        .lines()
        .filter(|l| l.starts_with("| `") && l.contains(".png`"))
    {
        let cells: Vec<&str> = line
            .split('|')
            .map(|c| c.trim().trim_matches('`'))
            .collect();
        let (file, bytes, hash) = (cells[1], cells[4], cells[5]);
        let data = std::fs::read(assets().join("items/fish").join(file)).unwrap();
        assert_eq!(data.len().to_string(), bytes, "{file}: length");
        assert_eq!(format!("{:016x}", fnv(&data)), hash, "{file}: bytes");
        checked += 1;
    }
    assert_eq!(checked, 5, "the five Tenebris fish");
}

/// The guide's numbers are the record's: a strength-5 fish shows the 0.43 s
/// window the bite uses, and a bed dweller says it lives on the bed.
#[test]
fn the_guide_reads_its_numbers_off_the_species() {
    let fauna = pbd_core::fauna::FaunaSettings::default();
    let roster = fauna.roster(pbd_core::fauna::HOME_BODY);
    let strongest = roster
        .iter()
        .find(|s| s.strength == 5)
        .expect("a strength-5 species");
    let facts = guide_facts(strongest, &fauna.fishing);
    let window = facts.iter().find(|(k, _)| *k == "HOOK WINDOW").unwrap();
    assert_eq!(window.1, "0.43 s");
    let ray = roster.iter().find(|s| s.id == "ray").unwrap();
    let facts = guide_facts(ray, &fauna.fishing);
    assert!(
        facts
            .iter()
            .any(|(k, v)| *k == "DEPTH" && v.starts_with("on the bed"))
    );
    for (key, value) in &facts {
        assert!(value.is_ascii() && !value.is_empty(), "{key}: {value}");
    }
}

/// The wiki page (`docs/wiki/fish.md`, written by `tools/gen_fish_wiki.py`)
/// holds to the roster the game loads: one section per species in roster
/// order under its icon, the species' own entry and tip, and every number the
/// in-game guide shows for it, as `guide_facts` writes it. A roster change
/// not regenerated into the page fails here.
#[test]
fn the_wiki_page_holds_to_the_roster() {
    let page = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/wiki/fish.md"
    ))
    .expect("docs/wiki/fish.md: run tools/gen_fish_wiki.py");
    let fauna = pbd_core::fauna::FaunaSettings::default();
    let sections: Vec<&str> = page.split("\n### ").skip(1).collect();
    let species: Vec<_> = fauna.bodies.values().flatten().collect();
    assert_eq!(
        sections.len(),
        species.len(),
        "one section per species: run tools/gen_fish_wiki.py"
    );
    for (section, species) in sections.iter().zip(species) {
        let heading = format!(
            "![](../../assets/{}) {}\n",
            species_icon(species),
            species.name()
        );
        assert!(
            section.starts_with(&heading),
            "{}: the section heading is not {heading:?}",
            species.id
        );
        assert!(
            section.contains(species.entry().as_str()),
            "{}: entry",
            species.id
        );
        assert!(
            section.contains(&format!("*Tip:* {}", species.tip())),
            "{}: tip",
            species.id
        );
        for (label, value) in guide_facts(species, &fauna.fishing) {
            assert!(
                section.contains(&format!("| {label} | {value} |")),
                "{}: {label} is not {value:?} on the page",
                species.id
            );
        }
    }
}

/// A world from before tool crafting with no tool record is dealt the four
/// tools with nothing in hand (`inventory-grid` decision 8); a world made
/// since owns none (`tool-crafting` decision 8); and one that recorded a
/// change comes back with it.
#[test]
fn the_tool_slot_opens_bare_handed_or_on_what_the_save_says() {
    let mut save = WorldSave::memory_only();
    let fresh = ToolSlot::restore(&save);
    assert_eq!(fresh.held(), None);
    assert!(fresh.tools().is_empty(), "a new world owns no tool");
    save.identity.tools = 0;
    let slot = ToolSlot::restore(&save);
    assert_eq!(slot.held(), None);
    assert!(Tool::ALL.iter().all(|t| slot.owns(*t)));
    assert!(
        Tool::ALL
            .iter()
            .all(|t| slot.tier(*t) == Some(pbd_core::inventory::Tier::Stone)),
        "at the stone tier"
    );
    let mut changed = slot;
    assert!(changed.hold(Tool::Axe));
    save.equipment = Some(changed.0);
    assert_eq!(ToolSlot::restore(&save).held(), Some(Tool::Axe));
}

/// The float's water is the sea table the hulls float on: its surface at a
/// point is the table's height there, and it moves with the swell.
#[test]
fn the_float_rides_the_sea_the_hulls_ride() {
    let water = crate::config::WaterSettings::default();
    let sea = Sea::new(&water);
    let beds = Mutex::new(HashMap::new());
    // A point well out to sea on the reference world.
    let direction = (0..4000)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / 4000.0;
            let r = (1.0 - y * y).sqrt();
            let phi = i as f32 * 2.399_963;
            Vec3::new(r * phi.cos(), y, r * phi.sin())
        })
        .find(|d| sea.depth_at(*d) > 30.0)
        .expect("open sea");
    let point = direction * sea.radius;
    let heights: Vec<f32> = [0.0, 1.3, 2.9, 4.4]
        .iter()
        .map(|seconds| {
            let here = PlanetWater::at(&sea, None, None, &beds, point, *seconds);
            let state = sea.state_at(None, direction);
            let table = sea
                .table
                .local(&state, direction, sea.depth_at(direction), *seconds);
            let surface = here.surface(point);
            assert_eq!(surface, sea.radius + table.height(point, sea.radius));
            surface - sea.radius
        })
        .collect();
    let spread = heights.iter().cloned().fold(f32::MIN, f32::max)
        - heights.iter().cloned().fold(f32::MAX, f32::min);
    assert!(spread > 0.01, "the float rises and falls: {heights:?}");
}

/// The drawn line starts at the rod tip and ends at the drawn float, in a
/// wave trough and pulled under on a bite, and lies on the water between
/// rather than through it (owner, 2026-11-16: "end of fishing rod rope is
/// not connected to bobber"; `fishing-and-equipment` design 4).
#[test]
fn the_line_runs_from_the_rod_tip_to_the_float() {
    let water = crate::config::WaterSettings::default();
    let sea = Sea::new(&water);
    let beds = Mutex::new(HashMap::new());
    let direction = Vec3::new(0.3, 0.9, 0.2).normalize();
    let side = direction.any_orthonormal_vector();
    let feet = direction * (sea.radius + 1.5);
    let tip = feet + direction * 1.6 + side * 0.8;
    for (seconds, dip, slack) in [(0.0, 0.0, 0.6), (2.3, 0.14, 0.6), (4.1, 0.0, 0.2)] {
        let out = (direction + side * 0.004).normalize();
        let here = PlanetWater::at(&sea, None, None, &beds, out * sea.radius, seconds);
        let surface = |p: Vec3| here.surface(p);
        // The float as `Line` places it: on the sea, less its dip.
        let float = out * (surface(out * sea.radius) - dip);
        let tied = float + float.normalize() * draw::FLOAT_TOP_M;
        let floor = |p: Vec3| draw::line_floor(&here, tied, p);
        let centre =
            pbd_core::world_frame::SimOrigin::new(crate::sim_origin::HOME, bevy::math::DVec3::ZERO);
        let points = draw::line_points(&centre, tip, tied, slack, None, floor);
        assert_eq!(points[0], Vec3::ZERO, "the line leaves the rod tip");
        let end = tip + *points.last().unwrap();
        assert!(
            end.distance(tied) < 1e-3,
            "the line ends at the float: {} m off",
            end.distance(tied)
        );
        // The same line laid in local space (`world-frames` 10.1), the origin
        // a cell from the angler: the same points, and its end exactly the
        // float's.
        let near = pbd_core::world_frame::SimOrigin::new(
            crate::sim_origin::HOME,
            (feet.as_dvec3() / 128.0).round() * 128.0,
        );
        let (tip_l, tied_l) = (near.from_query_point(tip), near.from_query_point(tied));
        let local = draw::line_points(&near, tip_l, tied_l, slack, None, |p| {
            floor(near.query_point(p))
        });
        assert!((tip_l + *local.last().unwrap()).distance(tied_l) < 1e-5);
        for (a, b) in points.iter().zip(&local) {
            assert!(a.distance(*b) < 2e-3, "{a} {b}");
        }
        for p in &points[1..points.len() - 1] {
            let at = tip + *p;
            assert!(
                at.length() >= floor(at) + draw::LINE_CLEARANCE_M - 1e-3,
                "a point of the line is not held clear of the water"
            );
            // Clear of the drawn sea and of the composite's waterline, the
            // sea's mean, so nothing of it is drawn as under water.
            assert!(
                at.length() > draw::drawn_surface(&here, at),
                "a point of the line is under the drawn sea"
            );
            assert!(
                at.length() > sea.radius,
                "a point of the line is under the waterline"
            );
        }
    }
}

/// A school fades in over its first frames and, once too far, fades out and
/// only then is dropped: never there or gone in one frame
/// (`ocean-environments` B18).
#[test]
fn a_school_fades_in_and_fades_out_before_it_is_dropped() {
    struct Deep;
    impl Water for Deep {
        fn surface(&self, point: Vec3) -> f32 {
            let _ = point;
            1000.0
        }
        fn bed(&self, point: Vec3) -> f32 {
            let _ = point;
            990.0
        }
    }
    let roster = FaunaConfig::default().0.roster(HOME_BODY).to_vec();
    let mut fishery = Fishery::default();
    let anchor = Vec3::Y * 1000.0;
    fishery
        .schools
        .push(School::spawn(&PLANET, 0, &roster[0], anchor, 7, &Deep));
    let dt = 1.0 / 60.0;
    let school = fishery.schools[0].clone();
    assert_eq!(
        fishery.fade_of(&school),
        1.0,
        "unseen before its first frame"
    );
    let mut last = 1.0;
    for _ in 0..60 {
        fishery.fade(dt);
        let now = fishery.fade_of(&school);
        assert!((last - now).abs() <= dt / SCHOOL_FADE_S + 1e-5);
        last = now;
    }
    assert_eq!(last, 0.0, "whole after its fade");
    // The player 200 m off: it starts leaving, and is kept until faded.
    let feet = (Vec3::Y * 1000.0 + Vec3::X * 200.0).normalize() * 1000.0;
    let mut frames = 0;
    while !fishery.schools.is_empty() {
        drop_far(&mut fishery, feet, 90.0);
        if fishery.schools.is_empty() {
            break;
        }
        let now = fishery.fade_of(&fishery.schools[0]);
        assert!((now - last).abs() <= dt / SCHOOL_FADE_S + 1e-5);
        last = now;
        fishery.fade(dt);
        frames += 1;
        assert!(frames < 200, "it was dropped");
    }
    assert!(last >= 0.99, "dropped only once faded: {last}");
    assert!(frames as f32 * dt >= SCHOOL_FADE_S - 2.0 * dt);
}

/// A puddle's fish are a third of their size; a wide, deep river's are full
/// size; a narrow brook sits between (`fishing-anywhere` design 3).
#[test]
fn smaller_water_makes_smaller_fish() {
    use crate::fish::body_scale;
    use bevy::prelude::Vec3;
    let up = Vec3::Y;
    let r = crate::planet::PLANET_RADIUS;
    let within = |metres: f32| move |p: Vec3| p.normalize().angle_between(up) * r <= metres;
    let puddle = body_scale(up, 0.4, within(2.0));
    let brook = body_scale(up, 1.0, within(7.0));
    let river = body_scale(up, 3.5, within(40.0));
    assert!((puddle - 0.35).abs() < 1e-3, "puddle {puddle}");
    assert!(brook > puddle && brook < river, "brook {brook}");
    assert!((river - 1.0).abs() < 1e-3, "river {river}");
}

/// Both buttons show in every phase with what each does now (owner,
/// 2026-11-18): only the left hooks, lit only at the bite; the right's
/// CANCEL is never lit.
#[test]
fn both_buttons_say_what_they_do_in_every_phase() {
    use pbd_core::fishing::Phase;
    let table = [
        (Phase::Ready, ("fishing.key.hold_to_cast", true), false),
        (Phase::Charging, ("fishing.key.let_go_to_cast", true), false),
        (Phase::Flying, ("fishing.key.hook", false), true),
        (Phase::Floating, ("fishing.key.hook", false), true),
        (Phase::Nibble, ("fishing.key.hook", false), true),
        (Phase::Bite, ("fishing.key.hook", true), true),
        (Phase::Hooked, ("fishing.key.hold_to_reel", true), false),
    ];
    for (phase, left, _) in table {
        let [l, r] = super::keys_for(phase);
        assert_eq!(l, left, "{phase:?}");
        let cancel = if matches!(phase, Phase::Ready | Phase::Charging | Phase::Hooked) {
            ""
        } else {
            "fishing.key.cancel"
        };
        assert_eq!(r, (cancel, false), "{phase:?}");
    }
}

/// The hooked float's one-frame jump when a run starts or stops is drawn
/// eased out, along the water, and gone in a quarter second; a plain reel and
/// every other phase are drawn where the line has them
/// (`movement-smoothness` 7.1).
#[test]
fn a_hooked_float_jump_is_eased_and_nothing_else_moves() {
    use draw::DrawnFloat;
    let r = 1000.0;
    let at = |x: f32| Vec3::new(x, r, 0.0).normalize() * r;
    let dt = 1.0 / 60.0;
    let mut drawn = DrawnFloat::default();
    // Floating and reeling: drawn as the line has it.
    drawn.update(&PLANET, dt, at(10.0), Phase::Floating, 0.0);
    assert_eq!(drawn.drawn(&PLANET, at(10.0)), at(10.0));
    drawn.update(&PLANET, dt, at(10.0), Phase::Bite, 0.0);
    // The strike, and on the next step the swing goes on with a jump of a
    // metre: drawn where it was.
    drawn.update(&PLANET, dt, at(10.0), Phase::Hooked, 0.8);
    drawn.update(&PLANET, dt, at(11.0), Phase::Hooked, 0.8);
    assert!(drawn.drawn(&PLANET, at(11.0)).distance(at(10.0)) < 1e-3);
    assert!(
        drawn.drawn(&PLANET, at(11.0)).length() - r < 1e-3,
        "on the water"
    );
    // Eased: each frame closer, no step larger than a sixth of the jump.
    let mut was = drawn.drawn(&PLANET, at(11.0));
    for frame in 0..25 {
        drawn.update(&PLANET, dt, at(11.0), Phase::Hooked, 0.8);
        let now = drawn.drawn(&PLANET, at(11.0));
        assert!(
            now.distance(was) < 0.17,
            "frame {frame}: {}",
            now.distance(was)
        );
        was = now;
    }
    assert!(drawn.drawn(&PLANET, at(11.0)).distance(at(11.0)) < 0.05);
    // The run stops: the swing's jump is taken up again.
    drawn.update(&PLANET, dt, at(10.4), Phase::Hooked, 0.0);
    assert!(drawn.drawn(&PLANET, at(10.4)).distance(was) < 0.05);
    // Caught: the line's float again.
    drawn.update(&PLANET, dt, at(10.4), Phase::Ready, 0.0);
    assert_eq!(drawn.drawn(&PLANET, at(10.4)), at(10.4));
}

/// A fish at the lure keeps the level heading it took the lure with: the
/// lure's bob (straight down and up, or not moving at all) never turns it
/// (`movement-smoothness` design 8).
#[test]
fn a_fish_at_the_lure_is_not_turned_by_the_bob() {
    let mut headings = draw::DrawnHeadings::default();
    let up = Vec3::Y;
    let key = (7, 3);
    // Swimming in, nose a little up.
    let mut forward = Vec3::ZERO;
    for _ in 0..30 {
        headings.begin();
        forward = headings.ease(key, Vec3::new(1.0, 0.2, 0.0), up, false, 1.0 / 60.0);
        headings.end();
    }
    let took = (forward - up * forward.dot(up)).normalize();
    // Held: the lure bobs down, up and stands still, frame by frame.
    for (i, bob) in [-0.6, 0.6, 0.0, -0.3, 0.0, 0.5]
        .iter()
        .cycle()
        .take(240)
        .enumerate()
    {
        headings.begin();
        let f = headings.ease(key, Vec3::new(0.0, *bob, 0.0), up, true, 1.0 / 60.0);
        headings.end();
        if i > 30 {
            assert!(f.dot(up).abs() < 1e-3, "level at the lure, frame {i}: {f}");
            assert!(
                f.dot(took) > 0.9999,
                "the bearing it took the lure with: {f}"
            );
        }
        if i > 0 {
            assert!(f.angle_between(forward).to_degrees() < 3.0, "no snap: {f}");
        }
        forward = f;
    }
}

/// A step's turn is spread over the frames between steps; a step with no
/// velocity keeps the heading; a reversal turns about the local up.
#[test]
fn a_fish_turns_over_frames_not_in_one() {
    let mut headings = draw::DrawnHeadings::default();
    let up = Vec3::Y;
    let key = (1, 1);
    headings.begin();
    let mut last = headings.ease(key, Vec3::X, up, false, 1.0 / 60.0);
    headings.end();
    assert!(last.dot(Vec3::X) > 0.9999);
    // Its step turns it right round in one step.
    let mut frames = 0;
    while last.dot(Vec3::NEG_X) < 0.99 {
        headings.begin();
        let f = headings.ease(key, Vec3::NEG_X, up, false, 1.0 / 60.0);
        headings.end();
        assert!(
            f.angle_between(last).to_degrees() < 30.0,
            "one frame turned too far"
        );
        assert!(f.dot(up).abs() < 1e-3, "the reversal turns about up: {f}");
        last = f;
        frames += 1;
    }
    assert!(frames >= 5, "spread over {frames} frames");
    // No velocity: it keeps its heading.
    headings.begin();
    let f = headings.ease(key, Vec3::ZERO, up, false, 1.0 / 60.0);
    headings.end();
    assert!(f.dot(last) > 0.9999);
    // Not drawn for a frame: forgotten.
    headings.begin();
    headings.end();
    headings.begin();
    let f = headings.ease(key, Vec3::Z, up, false, 1.0 / 60.0);
    headings.end();
    assert!(f.dot(Vec3::Z) > 0.9999);
}

/// The cast's line (`rod-motion` 8): while the float flies the line runs
/// from the rod tip back to where the float was let go and out along the
/// float's path, ending at the float; the trail keeps a place every
/// 0.15 m; once the float is down the line eases to its hanging curve over
/// 0.6 s, and a wound-in line forgets the trail.
#[test]
fn the_cast_line_follows_the_float_from_the_tip() {
    let up = Vec3::Y;
    let feet = up * 100.0;
    let tip = feet + Vec3::new(0.1, 1.9, 0.3);
    let mut trail = draw::CastTrail::default();
    // Let go at the tip, the float thrown up and away.
    let (mut float, mut v) = (tip, Vec3::new(0.0, 4.0, -9.0));
    trail.update(Phase::Flying, float, 1.0 / 60.0);
    assert_eq!(
        trail.points,
        vec![tip],
        "the trail starts where it was let go"
    );
    for _ in 0..30 {
        v -= up * 9.81 / 60.0;
        float += v / 60.0;
        trail.update(Phase::Flying, float, 1.0 / 60.0);
    }
    assert_eq!(trail.blend(), 1.0);
    for w in trail.points.windows(2) {
        assert!(w[0].distance(w[1]) > draw::TRAIL_STEP_M && w[0].distance(w[1]) < 0.4);
    }
    // The rod has come forward since: the drawn tip is ahead and lower.
    let tip_now = tip + Vec3::new(0.0, -0.3, -0.4);
    let tied = float + up * draw::FLOAT_TOP_M;
    let path = draw::trail_points(tip_now, &trail.points, tied, 24);
    assert_eq!(path.len(), 24);
    assert_eq!(path[0], tip_now, "starts at the tip");
    assert!(path[23].distance(tied) < 1e-4, "ends at the float");
    // It runs back up to the release point before it goes out.
    assert!(
        path.iter().any(|p| p.distance(tip) < 0.2),
        "through the release"
    );
    // Evenly along the path: no chord longer than the path's length over
    // the 23 steps (a chord across a corner is shorter).
    let length: f32 = std::iter::once(tip_now)
        .chain(trail.points.iter().copied())
        .chain(std::iter::once(tied))
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| w[0].distance(w[1]))
        .sum();
    assert!(
        path.windows(2)
            .all(|w| w[0].distance(w[1]) <= length / 23.0 + 1e-4),
        "evenly along"
    );
    // Drawn: the line's points lie on the trail while flying.
    let points = draw::line_points(
        &PLANET,
        tip_now,
        tied,
        0.6,
        Some((&trail.points, trail.blend())),
        |_| 0.0,
    );
    for (k, p) in points.iter().enumerate() {
        assert!((tip_now + *p).distance(path[k]) < 1e-4, "{k}");
    }
    // Down on the water: eased out over 0.6 s.
    trail.update(Phase::Floating, float, 1.0 / 60.0);
    assert_eq!(trail.blend(), 1.0);
    trail.update(Phase::Floating, float, 0.3);
    assert!(
        trail.blend() > 0.3 && trail.blend() < 0.7,
        "{}",
        trail.blend()
    );
    trail.update(Phase::Floating, float, 0.31);
    assert_eq!(trail.blend(), 0.0);
    let hanging = draw::line_points(&PLANET, tip_now, tied, 0.6, None, |_| 0.0);
    let eased = draw::line_points(
        &PLANET,
        tip_now,
        tied,
        0.6,
        Some((&trail.points, 0.0)),
        |_| 0.0,
    );
    assert_eq!(hanging, eased);
    trail.update(Phase::Ready, float, 1.0 / 60.0);
    assert!(trail.points.is_empty() && trail.blend() == 0.0);
    // A rebase moves the trail with the origin.
    trail.update(Phase::Flying, tip, 1.0 / 60.0);
    trail.rebase(Vec3::X);
    assert_eq!(trail.points, vec![tip - Vec3::X]);
}

/// A school laid every frame lays into the buffers its mesh already holds
/// (`memory-leak`): once it has reached its size a frame moves no attribute's
/// buffer, so nothing is allocated for it, and what it lays is what laying
/// it into a new mesh would.
mod laid_in_place {
    use super::*;
    use bevy::mesh::{MeshVertexAttribute, VertexAttributeValues};

    struct Deep;
    impl Water for Deep {
        fn surface(&self, point: Vec3) -> f32 {
            let _ = point;
            1000.0
        }
        fn bed(&self, point: Vec3) -> f32 {
            let _ = point;
            990.0
        }
    }

    const ATTRIBUTES: [MeshVertexAttribute; 7] = [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_COLOR,
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_NORMAL,
        bodies::ATTRIBUTE_SWIM_AXES,
        bodies::ATTRIBUTE_SWIM_WAVE,
        bodies::ATTRIBUTE_SWIM_UP,
    ];

    fn school() -> (School, Species) {
        let roster = FaunaConfig::default().0.roster(HOME_BODY).to_vec();
        let species = roster[0].clone();
        let school = School::spawn(&PLANET, 0, &species, Vec3::Y * 995.0, 11, &Deep);
        assert!(!school.is_empty(), "a school with fish in it");
        (school, species)
    }

    fn lay(
        mesh: &mut Mesh,
        school: &School,
        species: &Species,
        headings: &mut draw::DrawnHeadings,
    ) {
        let swim = draw::DrawnSwim::default();
        draw::build(
            &PLANET,
            mesh,
            school,
            species,
            None,
            None,
            0.5,
            Vec3::ZERO,
            (headings, &swim, 1.0 / 60.0),
        );
    }

    fn new_mesh() -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
    }

    /// Each attribute's buffer: where it is and how long.
    fn buffers(mesh: &Mesh) -> Vec<(usize, usize)> {
        ATTRIBUTES
            .iter()
            .map(|a| match mesh.attribute(a.id).expect("laid") {
                VertexAttributeValues::Float32x2(v) => (v.as_ptr() as usize, v.len()),
                VertexAttributeValues::Float32x3(v) => (v.as_ptr() as usize, v.len()),
                VertexAttributeValues::Float32x4(v) => (v.as_ptr() as usize, v.len()),
                _ => panic!("an attribute of another type"),
            })
            .collect()
    }

    #[test]
    fn a_school_is_laid_into_the_buffers_it_holds() {
        let (school, species) = school();
        let mut headings = draw::DrawnHeadings::default();
        let mut mesh = new_mesh();
        lay(&mut mesh, &school, &species, &mut headings);
        let first = buffers(&mesh);
        for frame in 0..30 {
            lay(&mut mesh, &school, &species, &mut headings);
            assert_eq!(buffers(&mesh), first, "frame {frame} moved a buffer");
        }
    }

    #[test]
    fn a_school_relaid_is_the_school_laid_fresh() {
        let (school, species) = school();
        // The same history of eased headings for both, so only the mesh
        // each is laid into differs.
        let (mut a, mut b) = (
            draw::DrawnHeadings::default(),
            draw::DrawnHeadings::default(),
        );
        let mut relaid = new_mesh();
        lay(&mut relaid, &school, &species, &mut a);
        lay(&mut relaid, &school, &species, &mut a);
        lay(&mut new_mesh(), &school, &species, &mut b);
        let mut fresh = new_mesh();
        lay(&mut fresh, &school, &species, &mut b);
        for a in ATTRIBUTES {
            let (x, y) = (relaid.attribute(a.id), fresh.attribute(a.id));
            assert_eq!(
                x.map(VertexAttributeValues::get_bytes),
                y.map(VertexAttributeValues::get_bytes),
                "{} differs",
                a.name
            );
        }
        // And the normals are what `compute_flat_normals` makes of the
        // positions.
        let mut bevy_normals = fresh.clone();
        bevy_normals.compute_flat_normals();
        assert_eq!(
            bevy_normals
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .map(VertexAttributeValues::get_bytes),
            fresh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .map(VertexAttributeValues::get_bytes)
        );
    }
}

/// `ocean-environments` 18.2 (design B23, cause 4): a school in a river over
/// the sea's level reads its depth from the RIVER's surface, which is what
/// `draw::schools` dims it by. Its anchor's surface is the river's top, not
/// the sea's sphere (from which the depth was negative and the school never
/// dimmed), and a fish half a metre under the river's top is half a metre
/// deep.
#[test]
fn a_school_in_a_river_reads_its_depth_from_the_river() {
    use crate::planet::terrain;
    let river = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: 28.25_f32.to_radians(),
        lon: 0.75_f32.to_radians(),
    });
    assert!(pbd_core::rivers::floods(terrain::terrain_config(), river));
    let set = crate::planet::lod::generate_fine(
        river,
        &crate::config::ColumnSettings::default(),
        &pbd_core::edits::Edits::new(),
    );
    let mut contact = crate::planet::PlanetContact::test_planet(5);
    contact.set_fine(&std::sync::Arc::new(set));
    let water = crate::config::WaterSettings::default();
    let sea = Sea::new(&water);
    let beds = Mutex::new(HashMap::new());
    let r = terrain::PLANET_RADIUS;
    let bed = r + terrain::surface_height(river);
    let top = contact
        .column_water(river * (bed + 0.5))
        .flatten()
        .map(|top| r + top)
        .expect("the river's water on its bed");
    assert!(top > sea.radius + 1.0, "the river stands over the sea");
    // The anchor as the spawner puts it: at the fresh water's top.
    let anchor = river * top;
    let here = PlanetWater::at(&sea, None, Some(&contact), &beds, anchor, 0.0);
    let surface = here.surface(anchor);
    assert!(
        (surface - top).abs() < 1e-3,
        "the school's surface is the river's: {surface} against {top}"
    );
    let fish = river * (top - 0.5);
    let depth = surface - fish.length();
    assert!((depth - 0.5).abs() < 1e-2, "half a metre down: {depth}");
    assert!(sea.radius - fish.length() < 0.0, "the sea's rule gave none");
}

/// The beaver is offered by the spawner only within 40 m of a colony that
/// is not gone (`ocean-environments` B29, task 8.4); the river otter, which
/// asks for none, anywhere its water allows.
#[test]
fn a_beaver_is_offered_only_by_a_colony() {
    use pbd_core::beaver::{Colony, Site, State};
    let fauna = crate::config::FaunaConfig::default();
    let roster = fauna.0.roster(HOME_BODY);
    let beaver = roster.iter().find(|s| s.id == "beaver").unwrap();
    let otter = roster.iter().find(|s| s.id == "river_otter").unwrap();
    assert_eq!(beaver.near_colony_m, 40.0);
    let at = Vec3::new(0.3, 0.9, 0.3).normalize();
    let site = Site {
        at: at.to_array(),
        lip: vec![1, 2, 3],
        pond: vec![4],
        bed: 10,
        bank: 12,
        trees: Vec::new(),
    };
    let mut colony = Colony::new(vec![site], 4, 0);
    let r = crate::planet::PLANET_RADIUS;
    let off = |m: f32| (at + at.any_orthonormal_vector() * (m / r)).normalize();
    let colonies = [colony.clone()];
    assert!(colony_allows(beaver, off(20.0), &colonies), "20 m from it");
    assert!(!colony_allows(beaver, off(55.0), &colonies), "55 m from it");
    assert!(!colony_allows(beaver, at, &[]), "no colony");
    assert!(
        colony_allows(otter, off(500.0), &[]),
        "the otter needs none"
    );
    colony.state = State::Gone;
    assert!(!colony_allows(beaver, at, &[colony]), "a gone colony");
}
