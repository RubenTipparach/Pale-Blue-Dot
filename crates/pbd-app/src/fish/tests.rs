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

/// A full hotbar lets the fish go and says so, rather than losing it
/// silently or recording a catch the player does not have.
#[test]
fn a_full_hotbar_lets_the_fish_go() {
    let mut hotbar = Hotbar::default();
    for i in 0..pbd_core::inventory::SLOTS {
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

/// A world with no tool record is dealt the four tools with the rod in hand,
/// and one that recorded a change comes back with it.
#[test]
fn the_tool_slot_opens_on_the_rod_or_on_what_the_save_says() {
    let mut save = WorldSave::memory_only();
    let slot = ToolSlot::restore(&save);
    assert_eq!(slot.held(), Tool::Rod);
    assert!(Tool::ALL.iter().all(|t| slot.owns(*t)));
    let mut changed = slot;
    assert!(changed.hold(Tool::Axe));
    save.equipment = Some(changed.0);
    assert_eq!(ToolSlot::restore(&save).held(), Tool::Axe);
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
