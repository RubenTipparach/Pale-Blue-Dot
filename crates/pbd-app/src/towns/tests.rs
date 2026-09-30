use super::*;
use crate::sites::{list_spawn, load_rules};

/// Holbrook, as a new world's list places it.
fn holbrook() -> Site {
    let list = pbd_core::sites::generate(
        &load_rules(),
        crate::planet::terrain_config(),
        list_spawn(),
        4,
    );
    list.sites
        .into_iter()
        .find(|s| s.home)
        .expect("a home town")
}

/// Slice 1: the village template is laid at Holbrook on the game's own
/// cells: every layout cell charted, the ground terraced under what it
/// builds and eased round it, and every building cut, standing on the
/// terrace.
#[test]
fn holbrook_is_laid_out_on_its_own_ground() {
    let site = holbrook();
    assert_eq!(site.kind, SiteKind::Village);
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let kits = load_kits();
    let repeats = load_repeats();
    let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
    let town = lay_out(&site, &template, &config).unwrap_or_else(|e| panic!("{e}"));
    let laid = build(&site, &town, &kits, &repeat, &config).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        laid.chart.cells.len(),
        town.cells.len(),
        "every footprint cell charted from the record"
    );
    for b in &town.buildings {
        for [c, r] in &b.cells {
            assert!(laid.chart.cell(*c, *r).is_some(), "{}: ({c}, {r})", b.name);
        }
    }
    let (footprint, margin) = laid.ground.counts();
    assert!(footprint > 300, "{footprint} footprint cells");
    let anchor = laid.patch.cells[laid
        .chart
        .cell(laid.chart.anchor.0, laid.chart.anchor.1)
        .unwrap()]
    .direction;
    let at = laid
        .ground
        .at(anchor)
        .expect("the anchor is on the town's ground");
    assert_eq!(at.ring, 0);
    assert_eq!(at.height(laid.terrace_m + 3.0), laid.terrace_m);
    assert!(laid.terrace_m > config.sea_level_m, "dry");
    let _ = margin;
    let triangles: usize = laid.meshes.values().map(|m| m.positions.len() / 3).sum();
    assert!(triangles > 5_000, "{triangles}");
    for (name, m) in &laid.meshes {
        assert!(repeats.contains_key(name), "{name} has no texture");
        for p in &m.positions {
            let h = Vec3::from_array(*p).length() - config.radius_m - laid.terrace_m;
            assert!(
                (-0.5..16.0).contains(&h),
                "{name}: {h:.2} m over the terrace"
            );
        }
    }
}

fn temporary(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-towns-{name}-{}-{}",
        std::process::id(),
        crate::saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Slice 3a: a world's first open stores its town, buildings first and
/// the settlement last, as the world's creation. Every later open builds
/// the stored town and writes nothing, even with the template changed; the
/// changed template is what a new world would get.
#[test]
fn a_world_stores_its_town_once_and_keeps_it_when_the_template_changes() {
    use crate::saves::{self, LOG, WorldSave};
    let root = temporary("keep");
    let slot = saves::create(&root, "Holbrook", 41).unwrap();
    let config = *crate::planet::terrain_config();
    let site = holbrook();
    let template = load_template("village");
    let made = {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        assert_eq!(stored(&save, site.id), Stored::None, "no town yet");
        let (town, seq) = ensure(&mut save, &site, &template, &config).expect("laid");
        assert!(seq > 0, "queued to the disk");
        save.drain();
        assert!(save.committed() >= seq, "and on it");
        town
    };
    let path = root.join(&slot.id).join(LOG);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rec @c building "))
            .count(),
        template.buildings.len(),
        "a line per building, the creation's"
    );
    let last = text.trim_end().lines().last().unwrap();
    assert!(
        last.starts_with(&format!("rec @c settlement {} 1 ", site.id)),
        "the settlement last"
    );
    let identity = saves::read_identity(&root.join(&slot.id)).expect("an identity");
    assert_eq!(identity.records.get("settlement"), Some(&1));
    assert_eq!(identity.records.get("building"), Some(&1));

    let mut revised = template.clone();
    revised.buildings.remove(0);
    revised.buildings[0].storeys += 1;
    let mut reopened = WorldSave::open(root.clone(), saves::list(&root)[0].clone());
    let (kept, seq) = ensure(&mut reopened, &site, &revised, &config).expect("kept");
    assert_eq!(kept, made, "the world keeps its town");
    assert_eq!(seq, 0, "and nothing is written");
    reopened.drain();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        text,
        "byte for byte"
    );
    assert_ne!(
        lay_out(&site, &revised, &config).unwrap(),
        made,
        "a new world would get the revision"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A settlement record this build cannot read is never taken for no town:
/// no town is built and nothing is written over it.
#[test]
fn a_damaged_settlement_is_neither_built_nor_written_over() {
    use crate::saves::WorldSave;
    use pbd_core::records::Record;
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let mut save = WorldSave::memory_only();
    save.store(
        &Author::Creation,
        vec![Record {
            kind: record::SETTLEMENT_RECORD.into(),
            id: u64::from(site.id),
            schema: record::RECORD_SCHEMA + 1,
            body: "(later: true)".into(),
        }],
    )
    .expect("stored");
    let before = save.records.len();
    let made = ensure(&mut save, &site, &load_template("village"), &config);
    assert!(made.is_err(), "{made:?}");
    assert_eq!(save.records.len(), before, "nothing written");
}

/// The town's ground is generation (the design's slice 3a): the terrace
/// over its footprint and the margin eased to the natural ground. It is
/// pinned here as a generator version's ground is, so a change to the
/// easing is a new rule for new towns and never reshapes a made one.
#[test]
fn holbrooks_ground_is_pinned() {
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let town = lay_out(&site, &load_template("village"), &config).unwrap();
    let laid = build(&site, &town, &load_kits(), &|_: &str| 2.0, &config).unwrap();
    let digest = Ground::new(config, vec![laid.ground.clone()]).digest();
    let (footprint, margin) = laid.ground.counts();
    println!(
        "Holbrook's ground: terrace {} m, {footprint} + {margin} cells, digest {digest:#018x}",
        town.terrace
    );
    assert_eq!(digest, HOLBROOK_GROUND);
}

/// Measured 2026-09-30 on generator 6: a terrace at 74 m over 799 cells, eased
/// over 2051.
const HOLBROOK_GROUND: u64 = 0xc331_13d0_7b63_82f3;

/// A measurement instrument, run by hand: where to stand in Holbrook for
/// the shots set beside the mockup's, as `--at` and `--yaw` for a capture.
/// Outside the Fieldstone house's door looking at it, inside looking out of
/// it (slice 2a's shot), and on the lane looking along it.
#[test]
#[ignore]
fn print_where_to_stand_for_the_mockup_shots() {
    let site = holbrook();
    let config = *crate::planet::terrain_config();
    let template = load_template("village");
    let repeats = load_repeats();
    let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
    let town = lay_out(&site, &template, &config).unwrap();
    let laid = build(&site, &town, &load_kits(), &repeat, &config).unwrap();
    let radius = config.radius_m;
    // `--at` and `--yaw` for standing at `stand` looking along `toward`.
    let spot = |name: &str, stand: Vec3, toward: Vec3| {
        let up = stand.normalize();
        let heading = Vec3::Y.cross(up).normalize();
        let flat = (toward - up * toward.dot(up)).normalize();
        let angle = heading.cross(flat).dot(up).atan2(heading.dot(flat));
        let (lat, lon) = pbd_core::geo::lat_lon(up).degrees();
        println!(
            "{name}: --at {lat:.5} {lon:.5} --yaw {:.1}",
            (-angle).to_degrees()
        );
    };
    let house = &template.buildings[0];
    let [c, r, d, _] = house.doors[0];
    let side = laid.chart.side(c, r, d as usize).unwrap();
    let cell = &laid.patch.cells[laid.chart.cell(c, r).unwrap()];
    let (a, e) = (cell.corners[side], cell.corners[(side + 1) % 6]);
    let door = ((a + e) * 0.5).normalize() * radius;
    let out = ((a + e) * 0.5 - cell.direction).normalize();
    spot(
        &format!("outside the {}", house.name),
        door + out * 3.5,
        -out,
    );
    spot(&format!("inside the {}", house.name), door - out * 1.6, out);
    // The lane at layout cell (20, 17), looking along the mockup's +x.
    let lane = &laid.patch.cells[laid.chart.cell(20, 17).unwrap()];
    let (lc, lr) = pbd_core::settlement::neighbour(20, 17, 0);
    let ahead = laid.patch.cells[laid.chart.cell(lc, lr).unwrap()].direction - lane.direction;
    spot("the lane at (20, 17)", lane.direction * radius, ahead);
}

/// A measurement instrument, run by hand: the rain over Holbrook at 11:00,
/// on a new world's first days and after each hour of `--weather-at`, along
/// the capture's own path (`Air::open`, then `run`). Picks a dry sky for the
/// town's shots: `--rain 0` forces no storm, but the weather still rains.
#[test]
#[ignore]
fn print_the_rain_over_holbrook() {
    use pbd_core::atmosphere::AtmosphereSettings;
    use pbd_core::daylight::Clock;
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/atmosphere.ron"
    );
    let settings: AtmosphereSettings =
        ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let here = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: 29.70f32.to_radians(),
        lon: 1.36f32.to_radians(),
    });
    let hour = (3600.0 / settings.dt_s).ceil() as u32;
    for day in 0..3 {
        let clock = Clock::at(day, 11.0);
        let mut air = crate::atmosphere::Air::open(
            settings,
            crate::planet::terrain_config().seed,
            None,
            clock.seconds,
        );
        let mut line = format!("--day {day} --time 11, --weather-at 0, 3600, ...:");
        for k in 0..6 {
            if k > 0 {
                air.run(hour, &[]);
            }
            line += &format!(" {:.2}", pbd_core::weather::rain_at(&air.now, here));
        }
        println!("{line}");
    }
}
