use super::*;
use std::sync::OnceLock;

fn shipped() -> SitesConfig {
    let text = include_str!("../../../../assets/config/sites.ron");
    let cfg: SitesConfig = ron::from_str(text).expect("sites.ron parses");
    cfg.validate().expect("sites.ron is valid");
    cfg
}

/// The game's default spawn direction (`desktop.rs`).
fn spawn() -> Vec3 {
    Vec3::new(0.8776, 0.4794, 0.0).normalize()
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get())
}

/// The shipped seed's list, made once for every test that reads it.
fn planet() -> &'static SiteList {
    static LIST: OnceLock<SiteList> = OnceLock::new();
    LIST.get_or_init(|| generate(&shipped(), &TerrainConfig::TENEBRIS, spawn(), threads()))
}

#[test]
fn the_shipped_config_is_valid() {
    let cfg = shipped();
    cfg.validate_pins(&TerrainConfig::TENEBRIS)
        .expect("the shipped pins stand on land");
}

/// "Two machines": one thread and many give the same list. Run at level 5
/// (10,242 candidates) so the single-threaded run stays quick; the rules and
/// the joins are the same at every level.
#[test]
fn one_thread_and_many_give_the_same_list() {
    let cfg = SitesConfig {
        level: 5,
        ..shipped()
    };
    let one = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), 1);
    let many = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), threads().max(3));
    assert!(!one.sites.is_empty());
    assert_eq!(one, many);
}

/// "Fewer sites keeps the same ones": a site's id is its cell and its kind
/// is its cell's, so lowering a count renames nothing that stays.
#[test]
fn fewer_towns_keeps_the_same_sites() {
    let mut cfg = shipped();
    let before = planet();
    for rule in &mut cfg.kinds {
        if rule.kind == SiteKind::Walled {
            rule.count -= 1;
        }
    }
    let after = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), threads());
    let mut kept = 0;
    for site in &after.sites {
        if let Some(old) = before.sites.iter().find(|s| s.id == site.id) {
            assert_eq!(old.kind, site.kind, "site {} changed kind", site.id);
            assert_eq!(old.direction, site.direction, "site {} moved", site.id);
            kept += 1;
        }
    }
    assert!(
        kept + 2 >= before.sites.len(),
        "only {kept} of {} kept",
        before.sites.len()
    );
}

/// "Flat, dry ground", "A harbour is on the water", "A cliff village climbs",
/// "A cave town has rock over it": every site's footprint, sampled at the
/// terrain's own cell spacing, passes its kind's rule. Sampled here afresh,
/// not through the rule's own code, for the flat kinds.
#[test]
fn every_site_stands_where_its_settlement_can_be_built() {
    let cfg = shipped();
    let terrain = TerrainConfig::TENEBRIS;
    let sea = terrain.sea_level_m;
    let shallows = crate::fauna::WaterLimits::default().shallows_max_m;
    for site in &planet().sites {
        let rule = cfg.rule(site.kind);
        let alts: Vec<f32> = footprint(site.direction, rule.radius_m, terrain.radius_m)
            .into_iter()
            .map(|p| surface_altitude(&terrain, p))
            .collect();
        let dry: Vec<f32> = alts.iter().copied().filter(|&h| h >= sea).collect();
        let lo = dry.iter().copied().fold(f32::MAX, f32::min);
        let hi = dry.iter().copied().fold(f32::MIN, f32::max);
        let label = format!("{} ({}, {})", site.name, site.kind.name(), site.id);
        match site.kind {
            SiteKind::Harbour => {
                assert!(
                    hi - sea <= cfg.harbour.shore_max_m,
                    "{label}: shore too high"
                );
                assert!(
                    alts.iter().any(|&h| h < sea && sea - h <= shallows),
                    "{label}: no shallows"
                );
                assert!(
                    shelf_within(&terrain, site.direction, cfg.harbour.shelf_reach_m),
                    "{label}: no shelf within reach"
                );
            }
            SiteKind::Swamp => {
                assert!(
                    alts.iter().all(|&h| sea - h <= shallows),
                    "{label}: deep water"
                );
                assert!(
                    hi - lo <= rule.flatness_m,
                    "{label}: {:.1} m of dry range",
                    hi - lo
                );
            }
            SiteKind::Cliff => {
                assert_eq!(dry.len(), alts.len(), "{label}: wet");
                assert!(
                    cliff_passes(&cfg, &terrain, site.direction, rule.radius_m),
                    "{label}"
                );
            }
            SiteKind::Cave => {
                assert_eq!(dry.len(), alts.len(), "{label}: wet");
                assert!(cave_passes(&cfg, &terrain, site.direction, 5.0), "{label}");
            }
            _ => {
                assert_eq!(dry.len(), alts.len(), "{label}: below the sea");
                assert!(
                    hi - lo <= rule.flatness_m,
                    "{label}: {:.1} m of range",
                    hi - lo
                );
            }
        }
    }
}

/// "No pentagons": no footprint reaches a pentagon or its neighbours.
#[test]
fn no_site_stands_on_a_pentagon() {
    let cfg = shipped();
    let terrain = TerrainConfig::TENEBRIS;
    for site in &planet().sites {
        for p in pentagons() {
            let gap = arc_m(p, site.direction, terrain.radius_m) - cfg.rule(site.kind).radius_m;
            assert!(
                gap > 2.0 * FINE_SPACING_M,
                "{} is {gap:.0} m from a pentagon",
                site.name
            );
        }
    }
}

/// "Every kind has its biome" and "Every kind occurs" on the shipped seed.
#[test]
fn every_kind_occurs_in_its_own_biome() {
    let terrain = TerrainConfig::TENEBRIS;
    for site in &planet().sites {
        let h = surface_altitude(&terrain, site.direction);
        assert_eq!(
            biome_at(&terrain, site.direction, h),
            site.kind.biome(),
            "{} is a {} out of its biome",
            site.name,
            site.kind.name()
        );
    }
    for kind in SiteKind::ALL {
        assert!(
            planet().sites.iter().any(|s| s.kind == kind),
            "no {} on the shipped seed",
            kind.name()
        );
    }
}

/// "Measuring every pair": no two sites stand closer than their spacing.
#[test]
fn every_pair_is_spaced_apart() {
    let cfg = shipped();
    let r = TerrainConfig::TENEBRIS.radius_m;
    let list = &planet().sites;
    for (i, a) in list.iter().enumerate() {
        for b in &list[i + 1..] {
            assert!(
                clear_of(&cfg, r, (a.kind, a.direction), (b.kind, b.direction)),
                "{} and {} are too close: {:.0} m",
                a.name,
                b.name,
                arc_m(a.direction, b.direction, r)
            );
        }
    }
}

/// "Unique names", and a name is the same from one run to the next.
#[test]
fn names_are_unique_and_stable() {
    let list = &planet().sites;
    let names: BTreeSet<&str> = list.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names.len(), list.len(), "two sites share a name");
    let cfg = SitesConfig {
        level: 5,
        ..shipped()
    };
    let a = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), threads());
    let b = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), threads());
    let names = |l: &SiteList| l.sites.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
    assert_eq!(names(&a), names(&b));
}

/// "Pinning a town" and "A pinned name": the town stands where it was put
/// with the name it was given, it is the capital, no generated site takes
/// its name, and none stands within its spacing.
#[test]
fn a_pinned_town_stands_where_it_was_put() {
    let terrain = TerrainConfig::TENEBRIS;
    let r = terrain.radius_m;
    // A generated walled town's place, moved a little: dry and off a
    // pentagon by construction.
    let donor = planet()
        .sites
        .iter()
        .find(|s| s.kind == SiteKind::Village && !s.home)
        .expect("a village");
    let (lat, lon) = geo::lat_lon(donor.direction).degrees();
    let name = planet().sites[0].name.clone();
    let mut cfg = shipped();
    cfg.pins.push(Pin {
        kind: SiteKind::Walled,
        lat,
        lon,
        name: Some(name.clone()),
        capital: true,
    });
    cfg.validate().expect("a valid pin");
    cfg.validate_pins(&terrain).expect("the pin is on land");
    let list = generate(&cfg, &terrain, spawn(), threads());
    let pinned: Vec<&Site> = list.sites.iter().filter(|s| s.pinned).collect();
    assert_eq!(pinned.len(), 1);
    let town = pinned[0];
    assert_eq!(town.name, name);
    assert!(town.capital);
    assert!(arc_m(town.direction, donor.direction, r) < 1.0);
    assert_eq!(list.sites.iter().filter(|s| s.name == name).count(), 1);
    assert_eq!(list.sites.iter().filter(|s| s.capital).count(), 1);
    for other in list.sites.iter().filter(|s| !s.pinned) {
        assert!(
            clear_of(
                &cfg,
                r,
                (town.kind, town.direction),
                (other.kind, other.direction)
            ),
            "{} stands within the pinned town's spacing",
            other.name
        );
    }
}

/// "Striking a site out": it is gone, and the rest keep their ids and names.
#[test]
fn a_struck_site_is_gone() {
    let before = planet();
    let victim = before
        .sites
        .iter()
        .rev()
        .find(|s| s.kind == SiteKind::Jungle)
        .expect("a jungle village");
    let mut cfg = shipped();
    cfg.strikes.push(victim.id);
    let after = generate(&cfg, &TerrainConfig::TENEBRIS, spawn(), threads());
    assert!(!after.sites.iter().any(|s| s.id == victim.id));
    let mut same = 0;
    for site in &after.sites {
        if let Some(old) = before.sites.iter().find(|s| s.id == site.id) {
            assert_eq!(old.kind, site.kind);
            if old.name == site.name {
                same += 1;
            }
        }
    }
    assert!(same + 3 >= before.sites.len() - 1, "{same} names kept");
}

/// Survey C2 and C3: a small town near the spawn, and the capital on
/// another land mass.
#[test]
fn a_small_town_is_near_the_spawn_and_the_capital_is_elsewhere() {
    let cfg = shipped();
    let terrain = TerrainConfig::TENEBRIS;
    let list = &planet().sites;
    let home: Vec<&Site> = list.iter().filter(|s| s.home).collect();
    assert_eq!(home.len(), 1, "one small town near the spawn");
    assert!(matches!(home[0].kind, SiteKind::Village | SiteKind::Walled));
    assert!(arc_m(home[0].direction, spawn(), terrain.radius_m) <= cfg.home_within_m);
    let capitals: Vec<&Site> = list.iter().filter(|s| s.capital).collect();
    assert_eq!(capitals.len(), 1, "one capital");
    assert_eq!(capitals[0].kind, SiteKind::Walled);
    let cells = Cells::new(cfg.level);
    let masses = land_masses(&terrain, &cells);
    assert_ne!(
        masses[cells.nearest(capitals[0].direction) as usize],
        masses[cells.nearest(spawn()) as usize],
        "the capital is on the spawn's land mass"
    );
}

/// The config refuses what the rules cannot run on, naming the field.
#[test]
fn a_bad_config_is_refused_by_name() {
    let mut zero = shipped();
    zero.kinds[1].spacing_m = 0.0;
    let error = zero.validate().unwrap_err();
    assert!(error.contains("kinds[1] (Village).spacing_m"), "{error}");

    let text = include_str!("../../../../assets/config/sites.ron").replacen(
        "kind: Walled",
        "kind: Castle",
        1,
    );
    let error = ron::from_str::<SitesConfig>(&text).unwrap_err().to_string();
    assert!(error.contains("Castle"), "{error}");

    let mut wet = shipped();
    wet.pins.push(Pin {
        kind: SiteKind::Village,
        lat: 0.0,
        lon: 0.0,
        name: Some("Atlantis".into()),
        capital: false,
    });
    // Find a direction at sea for the pin.
    let terrain = TerrainConfig::TENEBRIS;
    let sea = (0..10_000)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / 10_000.0;
            let a = i as f32 * 2.399_963;
            let r = (1.0 - y * y).sqrt();
            Vec3::new(a.cos() * r, y, a.sin() * r)
        })
        .find(|&d| surface_altitude(&terrain, d) < terrain.sea_level_m - 20.0)
        .expect("the planet has sea");
    let (lat, lon) = geo::lat_lon(sea).degrees();
    wet.pins[0].lat = lat;
    wet.pins[0].lon = lon;
    let error = wet.validate_pins(&terrain).unwrap_err();
    assert!(
        error.contains("pins[0] (Atlantis) stands at sea"),
        "{error}"
    );
}
