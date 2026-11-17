use super::*;

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

    for (strike, seed) in [("left", 70), ("right", 71)] {
        let mut line = Line::new(seed);
        let mut schools = vec![School::spawn(
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
                Phase::Bite if strike == "left" => Controls {
                    pressed: true,
                    held: true,
                    ..Default::default()
                },
                Phase::Bite => Controls {
                    reel_in: true,
                    ..Default::default()
                },
                Phase::Hooked => Controls {
                    held: line.tension < 0.6,
                    ..Default::default()
                },
                _ => Controls::default(),
            };
            let events = line.step(
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
                school.step(dt, &roster[minnow], &fauna.flock, &Shore, splash);
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
            species.name
        );
        assert!(
            section.starts_with(&heading),
            "{}: the section heading is not {heading:?}",
            species.id
        );
        assert!(
            section.contains(&species.guide.entry),
            "{}: entry",
            species.id
        );
        assert!(
            section.contains(&format!("*Tip:* {}", species.guide.tip)),
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

/// The drawn line starts at the rod tip and ends at the drawn float, at low
/// tide, in a wave trough and pulled under on a bite, and lies on the water
/// between rather than through it (owner, 2026-11-16: "end of fishing rod
/// rope is not connected to bobber"; `fishing-and-equipment` design 4). The
/// line was held over the mean sea, so at low water its end hung over the
/// float.
#[test]
fn the_line_runs_from_the_rod_tip_to_the_float() {
    let water = crate::config::WaterSettings::default();
    let mut sea = Sea::new(&water);
    let beds = Mutex::new(HashMap::new());
    let direction = Vec3::new(0.3, 0.9, 0.2).normalize();
    // Low water here: the moon a quarter turn round, the tide's full range.
    sea.set_moon(direction.any_orthonormal_vector());
    assert!(sea.tide_at(direction) < -0.9, "low water");
    let side = direction.any_orthonormal_vector();
    let feet = direction * (sea.radius + 1.5);
    let tip = feet + direction * 1.6 + side * 0.8;
    for (seconds, dip, slack) in [(0.0, 0.0, 0.6), (2.3, 0.14, 0.6), (4.1, 0.0, 0.2)] {
        let out = (direction + side * 0.004).normalize();
        let here = PlanetWater::at(&sea, None, None, &beds, out * sea.radius, seconds);
        let surface = |p: Vec3| here.surface(p);
        // The float as `Line` places it: on the sea, less its dip.
        let float = out * (surface(out * sea.radius) - dip);
        assert!(
            float.length() < sea.radius - 0.5,
            "the float rides under the mean"
        );
        let tied = float + float.normalize() * draw::FLOAT_TOP_M;
        let floor = |p: Vec3| draw::line_floor(&here, tied, p);
        let points = draw::line_points(tip, tied, slack, floor);
        assert_eq!(points[0], Vec3::ZERO, "the line leaves the rod tip");
        let end = tip + *points.last().unwrap();
        assert!(
            end.distance(tied) < 1e-3,
            "the line ends at the float: {} m off",
            end.distance(tied)
        );
        for p in &points[1..points.len() - 1] {
            let at = tip + *p;
            assert!(
                at.length() >= floor(at) + draw::LINE_CLEARANCE_M - 1e-3,
                "a point of the line is not held clear of the water"
            );
            // Clear of the drawn sea and of the composite's tided waterline,
            // so nothing of it is drawn as under water.
            assert!(
                at.length() > draw::drawn_surface(&here, at),
                "a point of the line is under the drawn sea"
            );
            assert!(
                at.length() > sea.radius + sea.tide_at(at),
                "a point of the line is under the tided waterline"
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
        .push(School::spawn(0, &roster[0], anchor, 7, &Deep));
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
