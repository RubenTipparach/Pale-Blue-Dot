use super::*;
use pbd_app::sites::load_rules;

const SCREEN: Vec2 = Vec2::new(1440.0, 900.0);

fn site(id: u32, kind: SiteKind, lat: f32, lon: f32, name: &str) -> Site {
    Site {
        id,
        kind,
        direction: geo::direction(geo::LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        }),
        name: name.into(),
        capital: false,
        home: false,
        pinned: false,
        river: false,
    }
}

fn list() -> Vec<Site> {
    let mut capital = site(1, SiteKind::Walled, -53.4, 108.3, "Ashingstead");
    capital.capital = true;
    let mut home = site(2, SiteKind::Village, 29.7, 1.4, "Holbrook");
    home.home = true;
    vec![
        capital,
        home,
        site(3, SiteKind::Harbour, 10.0, -40.0, "Coringport"),
        site(4, SiteKind::Cave, 45.0, 170.0, "Stoagard"),
    ]
}

fn app(view: MapView, night: bool) -> App {
    let mut app = App::new();
    app.insert_resource(Screen::Map)
        .insert_resource(view)
        .insert_resource(MapChoice { night, ..default() })
        .insert_resource(Sun::default())
        .insert_resource(WorldSites::ready_with(list()))
        .insert_resource(SitesRules(load_rules()))
        .init_resource::<DrawnSites>()
        .add_systems(Update, draw_sites);
    app.world_mut().spawn((
        Window {
            resolution: (SCREEN.x as u32, SCREEN.y as u32).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    app.world_mut().spawn((SiteLayer, Node::default()));
    app.world_mut().spawn((SitesNote, Text::new("")));
    // The first frame makes the nodes, the second places them.
    app.update();
    app.update();
    app
}

fn parts(app: &mut App) -> Vec<(SitePart, Node)> {
    let mut query = app.world_mut().query::<(&SitePart, &Node)>();
    query
        .iter(app.world())
        .map(|(p, n)| (*p, n.clone()))
        .collect()
}

fn px_of(v: Val) -> f32 {
    match v {
        Val::Px(p) => p,
        other => panic!("placed in pixels: {other:?}"),
    }
}

/// Task 4.1: every site has a marker, and it stands at its anchor's place
/// on the map, in the copy of the planet nearest the view's centre; the
/// legend says how many there are.
#[test]
fn every_site_has_a_marker_at_its_anchor() {
    let view = MapView {
        centre: Vec2::new(0.5, 0.5),
        px_per_turn: SCREEN.x,
    };
    let mut app = app(view, false);
    let parts = parts(&mut app);
    for (index, s) in list().iter().enumerate() {
        let want = view
            .clamped(SCREEN)
            .to_screen(geo::project(s.direction), SCREEN);
        let (_, node) = parts
            .iter()
            .find(|(p, _)| p.site == index && p.copy == 0 && p.part == Part::Icon)
            .unwrap_or_else(|| panic!("{} has a marker", s.name));
        assert_eq!(node.display, Display::Flex, "{} is shown", s.name);
        let half = px_of(node.width) / 2.0;
        let at = Vec2::new(px_of(node.left) + half, px_of(node.top) + half);
        assert!(at.distance(want) < 0.01, "{}: {at} against {want}", s.name);
        // A square for a town, a circle for a village.
        let round = node.border_radius == BorderRadius::MAX;
        assert_eq!(round, !is_big(s), "{}'s shape", s.name);
    }
    let crowns = parts.iter().filter(|(p, _)| p.part == Part::Crown).count();
    assert_eq!(crowns, 3, "one ring, the capital's, in each copy");
    let note = app
        .world_mut()
        .query_filtered::<&Text, With<SitesNote>>()
        .single(app.world())
        .unwrap()
        .0
        .clone();
    assert_eq!(note, "4 settlements on the map.");
}

/// Names: at the world's zoom only the capital's and the home town's; close
/// in, every name on screen. The footprint only once it is big enough to
/// see.
#[test]
fn names_and_footprints_come_in_as_the_map_zooms() {
    let shown = |app: &mut App, part: Part| -> Vec<usize> {
        let mut v: Vec<usize> = parts(app)
            .into_iter()
            .filter(|(p, n)| p.part == part && n.display == Display::Flex)
            .map(|(p, _)| p.site)
            .collect();
        v.sort();
        v.dedup();
        v
    };
    let world = MapView {
        centre: Vec2::new(0.5, 0.5),
        px_per_turn: SCREEN.x,
    };
    let mut far = app(world, false);
    assert_eq!(
        shown(&mut far, Part::Label),
        vec![0, 1],
        "the capital and home"
    );
    assert!(
        shown(&mut far, Part::Footprint).is_empty(),
        "too small to see"
    );
    let home = list()[1].clone();
    let close = MapView::at_metres_per_pixel(geo::project(home.direction), 2.0);
    let mut near = app(close, false);
    assert!(shown(&mut near, Part::Label).contains(&1));
    assert_eq!(
        shown(&mut near, Part::Footprint),
        vec![1],
        "the home town's, in view"
    );
}

/// At night, with the map's night on, a town in the dark lights a pool
/// round it; with it off, none does.
#[test]
fn a_town_in_the_dark_lights_the_ground_round_it() {
    let view = MapView {
        centre: Vec2::new(0.5, 0.5),
        px_per_turn: SCREEN.x,
    };
    let dark: Vec<usize> = list()
        .iter()
        .enumerate()
        .filter(|(_, s)| Sun::default().clock.daylight(s.direction) < 0.5)
        .map(|(i, _)| i)
        .collect();
    assert!(
        !dark.is_empty(),
        "some site is in the dark at the clock's start"
    );
    for night in [false, true] {
        let mut app = app(view, night);
        let lit: Vec<usize> = parts(&mut app)
            .into_iter()
            .filter(|(p, n)| p.part == Part::Glow && n.display == Display::Flex)
            .map(|(p, _)| p.site)
            .collect();
        if night {
            for i in &dark {
                assert!(lit.contains(i), "site {i} is lit");
            }
        } else {
            assert!(lit.is_empty(), "no lamplight with the night off");
        }
    }
}

/// Built towns on the map (`cities-in-the-world`, 2026-10-01): with the
/// world's towns known, a built town is ringed and drawn cream, a site still
/// to come is dimmed and an unsettled one red-edged; only a built town
/// lights the ground at night; the legend counts them.
#[test]
fn built_towns_are_ringed_and_counted() {
    use pbd_app::towns::{Held, Towns};
    use pbd_core::settlement::record::Town;
    let view = MapView {
        centre: Vec2::new(0.5, 0.5),
        px_per_turn: SCREEN.x,
    };
    let home = list()[1].clone();
    let towns = || {
        let town = Town {
            site: home.id,
            template: "village".into(),
            layout: 1,
            terrace: 0,
            anchor: (0, 0),
            cells: Vec::new(),
            levels: Vec::new(),
            buildings: Vec::new(),
        };
        Towns::holding(
            vec![Held {
                site: home.clone(),
                town,
            }],
            vec![4],
        )
    };
    for night in [false, true] {
        let mut app = app(view, night);
        app.insert_resource(towns());
        app.update();
        let shown = |app: &mut App, part: Part| -> Vec<usize> {
            let mut v: Vec<usize> = parts(app)
                .into_iter()
                .filter(|(p, n)| p.part == part && n.display == Display::Flex)
                .map(|(p, _)| p.site)
                .collect();
            v.sort();
            v.dedup();
            v
        };
        assert_eq!(
            shown(&mut app, Part::Built),
            vec![1],
            "only Holbrook is built"
        );
        if night {
            assert!(
                shown(&mut app, Part::Glow).iter().all(|&i| i == 1),
                "only a built town lights the ground"
            );
            continue;
        }
        let mut icons = app
            .world_mut()
            .query::<(&SitePart, &BackgroundColor, &BorderColor)>();
        let colours: Vec<(usize, Color, Color)> = icons
            .iter(app.world())
            .filter(|(p, ..)| p.part == Part::Icon && p.copy == 0)
            .map(|(p, f, b)| (p.site, f.0, b.top))
            .collect();
        // By place in the list: the capital and Coringport are still to
        // come, Holbrook is built, Stoagard (id 4) is unsettled.
        for (index, fill, border) in colours {
            match index {
                1 => assert_eq!(fill, Color::srgb_u8(0xf4, 0xef, 0xe4), "built is cream"),
                0 | 2 => assert!(fill.alpha() < 0.9, "to come is dimmed"),
                3 => assert_eq!(border, Color::srgba_u8(0xe0, 0x5a, 0x48, 0xb3), "unsettled"),
                _ => unreachable!("four sites"),
            }
        }
        let note = app
            .world_mut()
            .query_filtered::<&Text, With<SitesNote>>()
            .single(app.world())
            .unwrap()
            .0
            .clone();
        assert_eq!(
            note,
            "4 settlements on the map: 1 built, 2 still to come, 1 unsettled."
        );
    }
}
