use super::*;

const R: f32 = 4800.0;

fn shipped() -> NightLightsConfig {
    let text = include_str!("../../../../assets/config/night_lights.ron");
    let cfg: NightLightsConfig = ron::from_str(text).expect("night_lights.ron parses");
    cfg.validate().expect("night_lights.ron is valid");
    cfg
}

fn site(kind: SiteKind, capital: bool) -> Site {
    Site {
        id: 77,
        kind,
        direction: Vec3::new(0.3, 0.8, 0.2).normalize(),
        name: "Test".into(),
        capital,
        home: false,
        pinned: false,
        river: false,
    }
}

/// `n` lights scattered within `spread_m` of `centre`, a few metres apart.
fn lamps_round(centre: Vec3, n: usize, spread_m: f32) -> Vec<Vec3> {
    let centre = centre.normalize();
    let a = centre.cross(Vec3::Z).normalize();
    let b = centre.cross(a);
    (0..n)
        .map(|i| {
            let t = i as f32 * 2.399_963;
            let r = spread_m * ((i as f32 + 0.5) / n as f32).sqrt();
            (centre + (a * t.cos() + b * t.sin()) * (r / R)).normalize()
        })
        .collect()
}

#[test]
fn the_shipped_config_is_valid_and_ranks_the_kinds() {
    let cfg = shipped();
    let walled = site_light(&site(SiteKind::Walled, false), &cfg).unwrap();
    let village = site_light(&site(SiteKind::Village, false), &cfg).unwrap();
    let capital = site_light(&site(SiteKind::Walled, true), &cfg).unwrap();
    assert!(capital.brightness > walled.brightness && capital.radius_m > walled.radius_m);
    assert!(walled.brightness > village.brightness && walled.radius_m > village.radius_m);
    // Under the rock: nothing.
    assert_eq!(site_light(&site(SiteKind::Cave, false), &cfg), None);
}

#[test]
fn a_config_missing_a_kind_or_with_the_fade_backwards_is_refused() {
    let mut cfg = shipped();
    cfg.kinds.remove(&SiteKind::Swamp);
    assert!(cfg.validate().is_err());
    let mut cfg = shipped();
    cfg.near_in_m = cfg.near_out_m;
    assert!(cfg.validate().is_err());
}

/// Dark by day, whole at night far off, gone close to.
#[test]
fn a_far_light_is_lit_at_night_far_off_and_not_by_day_or_near() {
    let cfg = shipped();
    let light = site_light(&site(SiteKind::Walled, false), &cfg).unwrap();
    let d = light.direction;
    let side = d.cross(Vec3::X).normalize();
    let far = d * (R + 12_000.0);
    // The sun straight overhead, then straight below.
    assert_eq!(strength(&light, d, far, R, &cfg), 0.0);
    let night = strength(&light, -d, far, R, &cfg);
    assert!((night - light.brightness).abs() < 1e-5, "{night}");
    // At sunset (the sun on the horizon) not yet; with the sun 4 degrees
    // down, partly; 8 degrees down, whole.
    let below = |deg: f32| (side * deg.to_radians().cos() - d * deg.to_radians().sin()).normalize();
    assert_eq!(
        strength(&light, side, far, R, &cfg),
        light.brightness * sky_light::night(0.0)
    );
    let dusk = strength(&light, below(4.0), far, R, &cfg);
    assert!(
        dusk > 0.1 * light.brightness && dusk < 0.9 * light.brightness,
        "{dusk}"
    );
    assert!((strength(&light, below(8.0), far, R, &cfg) - light.brightness).abs() < 1e-5);
    // Within the town's standing range: nothing, so the real lamps are alone.
    let over = d * (R + 600.0);
    assert_eq!(strength(&light, -d, over, R, &cfg), 0.0);
    let between = strength(&light, -d, d * (R + 1300.0), R, &cfg);
    assert!(between > 0.0 && between < light.brightness);
}

/// The player's lights: none below the threshold, one far light at it.
#[test]
fn the_players_lights_show_once_a_group_reaches_the_threshold() {
    let cfg = shipped();
    let n = cfg.player_threshold as usize;
    let at = Vec3::new(-0.2, 0.5, 0.84).normalize();
    let lamps = lamps_round(at, n, 25.0);
    assert!(player_clusters(&lamps[..n - 1], R, &cfg).is_empty());
    let clusters = player_clusters(&lamps, R, &cfg);
    assert_eq!(clusters.len(), 1);
    let c = clusters[0];
    assert!(c.direction.distance(at) * R < 5.0, "centred on the lights");
    assert!(c.radius_m >= cfg.player_min_radius_m);
    assert!(
        (c.brightness - cfg.player_brightness * n as f32 / cfg.player_full_count as f32).abs()
            < 1e-5
    );
    // More lights, brighter, up to full.
    let many = player_clusters(&lamps_round(at, 200, 60.0), R, &cfg);
    assert_eq!(many.len(), 1);
    assert_eq!(many[0].brightness, cfg.player_brightness);
}

/// The grouping is the same whatever order the edits came in, and two bases
/// far apart are two groups, each judged alone.
#[test]
fn the_grouping_owes_nothing_to_order_and_keeps_far_bases_apart() {
    let cfg = shipped();
    let a = Vec3::new(0.9, 0.1, 0.3).normalize();
    let b = Vec3::new(0.1, 0.9, -0.3).normalize();
    let mut lamps = lamps_round(a, 20, 30.0);
    lamps.extend(lamps_round(b, 30, 40.0));
    // A third, too small to show.
    lamps.extend(lamps_round(Vec3::new(-0.5, -0.5, 0.7), 5, 10.0));
    let forward = player_clusters(&lamps, R, &cfg);
    lamps.reverse();
    let backward = player_clusters(&lamps, R, &cfg);
    assert_eq!(forward.len(), 2);
    assert_eq!(forward, backward);
    // Two groups of eight 300 m apart do not add up to one of sixteen.
    let mut split = lamps_round(a, 8, 10.0);
    let a_far = (a + a.cross(Vec3::Y).normalize() * (300.0 / R)).normalize();
    split.extend(lamps_round(a_far, 8, 10.0));
    assert!(player_clusters(&split, R, &cfg).is_empty());
}
