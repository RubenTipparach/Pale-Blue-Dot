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
    let laid = lay_out(&site, &template, &kits, &repeat, &config).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        laid.chart.cells.len(),
        template.ground.len(),
        "every layout cell charted"
    );
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
    let laid = lay_out(&site, &template, &load_kits(), &repeat, &config).unwrap();
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
