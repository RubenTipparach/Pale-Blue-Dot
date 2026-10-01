//! The app's side of the craft, driven through the real plugin: they are
//! placed where they can work, boarding hands the player over and back, and a
//! craft left behind is still there, in the save, with its ID.

use super::*;
use crate::flight_view::{FlightViewConfig, FlightViewPlugin};
use crate::planet::PlanetContact;
use crate::planet::terrain::PLANET_RADIUS;
use crate::walking::{EYE_HEIGHT, HALF_HEIGHT, WalkingPlugin};
use bevy::input::mouse::AccumulatedMouseMotion;
use pbd_core::vehicle::record::RECORD_VERSION;

fn app(save: crate::saves::WorldSave) -> App {
    let spawn = FlightViewConfig::default().spawn_direction;
    let water = crate::config::WaterSettings::default();
    let mut app = crate::headless_app();
    app.add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Image>()
        .insert_resource(PlanetContact::test_planet(5))
        .insert_resource(Sea::new(&water))
        .insert_resource(water)
        .insert_resource(WeatherSettings::default())
        .insert_resource(crate::config::VehiclesConfig::default())
        .insert_resource(crate::sky::Sun::default())
        .insert_resource(crate::planet::PlanetRenderFrame::default())
        .insert_resource(save)
        .insert_resource(crate::CelestialScene::planet_at_origin(
            PLANET_RADIUS as f64,
            1.0,
        ))
        .insert_resource(FlightViewConfig {
            minimum_clearance: EYE_HEIGHT,
            spawn_direction: spawn,
            ..default()
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseMotion>()
        .add_plugins((FlightViewPlugin, WalkingPlugin, VehiclePlugin));
    app.finish();
    app.cleanup();
    app.update();
    app
}

/// A tap: down for one update and up on the next.
fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.clear();
    keys.release(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
}

fn crafts(app: &mut App) -> Vec<(Entity, Craft)> {
    let mut query = app.world_mut().query::<(Entity, &Vehicle)>();
    let mut all: Vec<_> = query
        .iter(app.world())
        .map(|(e, v)| (e, v.craft.clone()))
        .collect();
    all.sort_by_key(|(_, c)| c.id);
    all
}

fn walker(app: &mut App) -> Vec3 {
    let mut query = app.world_mut().query_filtered::<&Position, With<Walker>>();
    query.single(app.world()).unwrap().0
}

/// A new world's three craft each stand where they can work: the Kestrel on
/// dry ground near the player, each boat afloat, at anchor, in water deep
/// enough for it.
#[test]
fn a_new_world_places_each_craft_where_it_can_work() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    assert!(app.world().resource::<Fleet>().spawned);
    let all = crafts(&mut app);
    let kinds: Vec<Kind> = all.iter().map(|(_, c)| c.kind).collect();
    assert_eq!(
        kinds,
        [Kind::Kestrel, Kind::Tern, Kind::Loon],
        "all three, in ID order"
    );
    let player = walker(&mut app);
    let sea = app.world().resource::<Sea>().clone();
    for (_, craft) in &all {
        let at = craft.reference_position().as_vec3();
        let depth = sea.depth_at(at.normalize());
        match craft.kind {
            Kind::Kestrel => {
                assert_eq!(depth, 0.0, "on dry ground");
                assert!(at.distance(player) < 50.0, "and near the player");
                assert!(craft.mooring.is_none());
            }
            Kind::Tern | Kind::Loon => {
                let need = if craft.kind == Kind::Tern { 3.0 } else { 1.2 };
                assert!(depth >= need, "{} in {depth} m", craft.kind.name());
                assert!(craft.mooring.is_some_and(|m| m.anchored), "and at anchor");
            }
            Kind::Cog => unreachable!("a harbour moors the cog, not the starting fleet"),
        }
    }
    // And they stay afloat and upright: a berth judged on a coarser seabed
    // than the one the hull meets runs the keel aground on the first tick.
    for _ in 0..180 {
        app.update();
    }
    for (_, craft) in crafts(&mut app) {
        let up = craft.body.position.normalize();
        let upright = (craft.body.orientation * pbd_core::DVec3::Y).dot(up);
        assert!(
            upright > 0.97,
            "{} is heeled: {upright:.3}",
            craft.kind.name()
        );
        if craft.kind != Kind::Kestrel {
            let lift = craft.reference_position().length() - sea.radius as f64;
            assert!(
                lift.abs() < 0.5,
                "{} floats at the sea: {lift:.2} m",
                craft.kind.name()
            );
        }
    }
    // The new fleet is written, not left owed.
    assert!(!app.world().resource::<Fleet>().dirty);
    let saved = app
        .world()
        .resource::<crate::saves::WorldSave>()
        .vehicles
        .clone();
    assert_eq!(saved.map(|f| f.vehicles.len()), Some(3));
}

/// Board, drive, step off: the craft changes occupancy and never existence.
/// It stays where it was left, keeps its ID, is written to the save, and
/// comes back from that save after the world is put away.
#[test]
fn a_craft_left_behind_stays_boardable_and_comes_back_from_its_save() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (entity, kestrel) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Kestrel)
        .unwrap();
    // Walk up to it and board.
    board_from(&mut app, kestrel.exit().as_vec3());
    assert_eq!(app.world().resource::<Aboard>().0, Some(entity), "F boards");
    assert!(!app.world().resource::<WalkingState>().active);
    let mut cameras = app
        .world_mut()
        .query_filtered::<&Camera, With<VehicleCamera>>();
    assert!(
        cameras.single(app.world()).unwrap().is_active,
        "its camera is the view"
    );
    // Power up and lift off for a couple of seconds.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    for _ in 0..150 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::Space);
    let flown = crafts(&mut app)[0].1.clone();
    let lifted = flown.reference_position().length() - kestrel.reference_position().length();
    assert!(lifted > 2.0, "power lifts it: {lifted:.2} m");
    // Step off.
    tap(&mut app, KeyCode::KeyF);
    assert_eq!(app.world().resource::<Aboard>().0, None);
    assert!(
        app.world().resource::<WalkingState>().active,
        "back on foot"
    );
    let left = crafts(&mut app);
    assert_eq!(left.len(), 3, "nothing was despawned");
    let (still, craft) = left.iter().find(|(_, c)| c.id == kestrel.id).unwrap();
    assert_eq!(*still, entity, "the same craft");
    assert!(!craft.occupied);
    assert!(
        walker(&mut app).distance(craft.exit().as_vec3()) < 3.0,
        "the player steps off beside it"
    );
    app.update();
    let saved = app
        .world()
        .resource::<crate::saves::WorldSave>()
        .vehicles
        .clone()
        .expect("the fleet is saved");
    assert_eq!(saved.version, RECORD_VERSION);
    let record = saved.vehicles.iter().find(|r| r.id == kestrel.id).unwrap();
    // Put the world away and bring it back: same IDs, same poses.
    let now = crafts(&mut app);
    let file = put_away(app.world_mut()).expect("a spawned fleet");
    assert!(crafts(&mut app).is_empty());
    app.world_mut()
        .resource_mut::<crate::saves::WorldSave>()
        .snapshot_vehicles(&file);
    app.update();
    let back = crafts(&mut app);
    assert_eq!(
        back.iter().map(|(_, c)| c.id).collect::<Vec<_>>(),
        now.iter().map(|(_, c)| c.id).collect::<Vec<_>>()
    );
    let restored = &back.iter().find(|(_, c)| c.id == kestrel.id).unwrap().1;
    let at = pbd_core::DVec3::from(record.position);
    assert!(
        restored.reference_position().distance(at) < 0.5,
        "back where it was left"
    );
    assert!(app.world().resource::<Fleet>().next_id > kestrel.id);
    // And F boards it where it was left.
    let (entity, craft) = back.into_iter().find(|(_, c)| c.id == kestrel.id).unwrap();
    board_from(&mut app, craft.exit().as_vec3());
    assert_eq!(
        app.world().resource::<Aboard>().0,
        Some(entity),
        "boardable again"
    );
}

/// Stand at `at` and press F, the interaction key: it acts on the press.
fn board_from(app: &mut App, at: Vec3) {
    stand_at(app, at);
    tap(app, KeyCode::KeyF);
}

fn stand_at(app: &mut App, at: Vec3) {
    let mut query = app.world_mut().query_filtered::<Entity, With<Walker>>();
    let body = query.single(app.world()).unwrap();
    let center = app
        .world()
        .resource::<crate::planet::PlanetRenderFrame>()
        .center;
    let up = (at.as_dvec3() - center).normalize().as_vec3();
    app.world_mut().entity_mut(body).insert((
        Position(at + up * crate::walking::HALF_HEIGHT),
        Transform::from_translation(at),
    ));
}

fn stop_clock(app: &mut App) {
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::ZERO,
    ));
}

fn camera_pose(app: &mut App) -> Transform {
    *app.world_mut()
        .query_filtered::<&Transform, With<VehicleCamera>>()
        .single(app.world())
        .unwrap()
}

#[test]
fn translated_boarding_and_camera_follow_use_the_current_frame() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    stop_clock(&mut app);
    app.add_systems(
        PostUpdate,
        crate::planet::update_planet_frame.before(bevy::transform::TransformSystems::Propagate),
    );
    let offset = DVec3::new(8192.0, -4096.0, 2048.0);
    app.world_mut()
        .resource_mut::<crate::PhysicsFrame>()
        .0
        .origin = -offset;
    app.world_mut()
        .resource_mut::<crate::planet::PlanetRenderFrame>()
        .center = offset;
    let (entity, craft) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    stand_at(&mut app, (offset + craft.exit()).as_vec3());
    app.world_mut().resource_mut::<view::VehicleView>().seat = true;
    tap(&mut app, KeyCode::KeyF);
    assert_eq!(app.world().resource::<Aboard>().0, Some(entity));
    assert!(
        camera_pose(&mut app)
            .translation
            .distance((offset + craft.eye()).as_vec3())
            < 0.002
    );
    // Change only the authoritative origin: follow must see the newly published centre.
    let next = offset + DVec3::new(512.0, -256.0, 128.0);
    app.world_mut()
        .resource_mut::<crate::PhysicsFrame>()
        .0
        .origin = -next;
    app.update();
    assert!(
        camera_pose(&mut app)
            .translation
            .distance((next + craft.eye()).as_vec3())
            < 0.002
    );
}

#[test]
fn mouse_look_is_immediate_without_a_physics_tick_and_v_switches_views() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    stop_clock(&mut app);
    let (entity, craft) = crafts(&mut app).remove(0);
    app.world_mut().resource_mut::<view::VehicleView>().seat = true;
    take_seat(app.world_mut(), entity, true);
    let delta = Vec2::new(37.0, -19.0);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = delta;
    app.update();
    let sensitivity = crate::flight_view::MOUSE_LOOK_SENSITIVITY;
    let want = craft.body.orientation.as_quat()
        * Quat::from_rotation_y(-delta.x * sensitivity)
        * Quat::from_rotation_x(-delta.y * sensitivity);
    assert!(camera_pose(&mut app).rotation.angle_between(want) < 0.001);
    assert_eq!(
        app.world()
            .get::<Vehicle>(entity)
            .unwrap()
            .craft
            .body
            .orientation,
        craft.body.orientation
    );
    let seat = camera_pose(&mut app).translation;
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::ZERO;
    tap(&mut app, KeyCode::KeyV);
    assert!(!app.world().resource::<view::VehicleView>().seat);
    assert!(camera_pose(&mut app).translation.distance(seat) > 5.0);
    tap(&mut app, KeyCode::KeyV);
    assert!(app.world().resource::<view::VehicleView>().seat);
    assert!(camera_pose(&mut app).translation.distance(seat) < 0.001);
}

#[test]
fn each_craft_maps_its_controls_and_menus_suppress_keys_and_look() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    stop_clock(&mut app);
    // The starting fleet has no cog (a harbour moors it), so one is put
    // where the Tern lies, for its keys.
    let tern = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Tern)
        .expect("the Tern")
        .1;
    let cog = {
        let mut fleet = app.world_mut().resource_mut::<Fleet>();
        let id = fleet.next_id;
        fleet.next_id += 1;
        pbd_core::vehicle::Craft::new(
            Kind::Cog,
            id,
            fleet.specs.clone(),
            fleet.hulls.clone(),
            tern.reference_position(),
            tern.body.orientation,
        )
    };
    super::place::spawn_craft(app.world_mut(), cog);
    for (entity, craft) in crafts(&mut app) {
        take_seat(app.world_mut(), entity, true);
        let held = [
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyQ,
            KeyCode::Space,
            KeyCode::KeyZ,
            KeyCode::KeyB,
        ];
        for key in held {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        app.update();
        let input = app.world().resource::<VehicleControls>().0;
        match craft.kind {
            Kind::Kestrel => {
                assert_eq!(
                    (
                        input.pitch,
                        input.roll,
                        input.yaw,
                        input.collective,
                        input.tilt
                    ),
                    (-1.0, -1.0, -1.0, 1.0, -1.0)
                );
                assert!(!input.bail);
            }
            Kind::Tern => assert_eq!(
                (input.steer, input.sheet, input.crew, input.bail),
                (1.0, 1.0, -1.0, true)
            ),
            Kind::Loon => assert_eq!(
                (input.forward, input.steer, input.rudder, input.bail),
                (1.0, 1.0, 1.0, true)
            ),
            Kind::Cog => assert_eq!((input.steer, input.sheet, input.bail), (1.0, 1.0, true)),
        }
        let before = camera_pose(&mut app);
        let seat = app.world().resource::<view::VehicleView>().seat;
        app.insert_resource(crate::controls::MenuOpen(true));
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::splat(100.0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyV);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        let input = app.world().resource::<VehicleControls>().0;
        assert_eq!(input, Input::default());
        assert_eq!(app.world().resource::<Aboard>().0, Some(entity));
        assert_eq!(app.world().resource::<view::VehicleView>().seat, seat);
        assert_eq!(camera_pose(&mut app), before);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
        app.insert_resource(crate::controls::MenuOpen(false));
    }
}

/// Casting off and anchoring are written the frame they happen, not on a
/// timer: the save holds the new state after one update.
#[test]
fn anchoring_and_casting_off_are_saved_at_once() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (entity, loon) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    board_from(&mut app, loon.exit().as_vec3());
    assert_eq!(
        app.world().resource::<Aboard>().0,
        Some(entity),
        "F boards the Loon"
    );
    let saved = |app: &App| {
        app.world()
            .resource::<crate::saves::WorldSave>()
            .vehicles
            .as_ref()
            .and_then(|f| f.vehicles.iter().find(|r| r.id == loon.id).cloned())
            .expect("the Loon is in the save")
    };
    assert!(saved(&app).mooring.is_some(), "it was placed at anchor");
    tap(&mut app, KeyCode::KeyT);
    assert!(
        saved(&app).mooring.is_none(),
        "cast off, and saved in the same update"
    );
    tap(&mut app, KeyCode::KeyT);
    let anchor = saved(&app).mooring.expect("anchored again, and saved");
    assert!(anchor.2, "an anchor, not a line to a bollard");
}

/// G beside a craft is the tool picker, never a boarding: only F boards.
#[test]
fn holding_g_beside_a_craft_boards_nothing() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (_, loon) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    let at = loon.exit().as_vec3();
    stand_at(&mut app, at);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyG);
    for _ in 0..4 {
        stand_at(&mut app, at);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    assert!(
        app.world().resource::<crate::controls::PickerKey>().holding,
        "the picker is open"
    );
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::KeyG);
    stand_at(&mut app, at);
    app.update();
    assert_eq!(app.world().resource::<Aboard>().0, None, "G boards nothing");
}

/// A capture's hold keeps the Kestrel where it was put, 20 m up where it could
/// not stay by itself, at rest; let go, it falls (`lamps-and-lanterns` task
/// 3.3's cave rig).
#[test]
fn a_held_kestrel_stays_where_the_capture_put_it() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let kestrel = |app: &mut App| {
        crafts(app)
            .into_iter()
            .find(|(_, c)| c.kind == Kind::Kestrel)
            .map(|(_, c)| c)
            .expect("the fleet has a Kestrel")
    };
    let parked = kestrel(&mut app);
    let at = parked.body.position + parked.body.position.normalize() * 20.0;
    let facing = parked.body.orientation;
    app.insert_resource(CraftHold(Some((at, facing))));
    for _ in 0..120 {
        app.update();
    }
    let held = kestrel(&mut app);
    assert!(
        held.body.position.distance(at) < 1e-6,
        "held at {at}, found at {}",
        held.body.position
    );
    assert_eq!(held.body.velocity, DVec3::ZERO);
    assert!(held.body.orientation.abs_diff_eq(facing, 1e-9));
    app.insert_resource(CraftHold(None));
    for _ in 0..120 {
        app.update();
    }
    let fell = at.length() - kestrel(&mut app).body.position.length();
    assert!(fell > 1.0, "let go, it falls: {fell:.2} m");
}

/// Task 4.2b: a craft left far away is stowed as its record, still in the
/// save, and comes back where it was left once the player is near again.
#[test]
fn a_craft_far_away_is_stowed_and_comes_back_where_it_was_left() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (entity, loon) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    // Carry it 3 km round the planet away from the player, past the stowing
    // range.
    let far = {
        let at = loon.reference_position();
        let up = at.normalize();
        let away = at - walker(&mut app).as_dvec3();
        let side = (away - up * away.dot(up)).normalize();
        let angle = 3000.0 / at.length();
        (up * angle.cos() + side * angle.sin()) * at.length()
    };
    {
        let mut vehicle = app.world_mut().get_mut::<Vehicle>(entity).unwrap();
        let orientation = vehicle.craft.body.orientation;
        vehicle.craft.set_reference_pose(far, orientation);
        vehicle.craft.mooring = None;
    }
    for _ in 0..120 {
        app.update();
    }
    assert!(
        crafts(&mut app).iter().all(|(_, c)| c.id != loon.id),
        "stowed out of the world"
    );
    let stowed = app.world().resource::<Fleet>().stowed.clone();
    let record = stowed
        .iter()
        .find(|r| r.id == loon.id)
        .expect("its record kept");
    let file = app.world().resource::<Fleet>().file(std::iter::empty());
    assert!(file.vehicles.iter().any(|r| r.id == loon.id), "in the save");
    // Its record is where it was left.
    assert!(
        pbd_core::DVec3::from(record.position).distance(far) < 1.0,
        "stowed where it was left"
    );
    // Bring the record within range of the player, as coming near it does,
    // and it is back in the world at that pose, the same craft.
    let near = {
        let w = walker(&mut app).as_dvec3();
        let up = w.normalize();
        let side = up.any_orthonormal_vector();
        let angle = 60.0 / w.length();
        (up * angle.cos() + side * angle.sin()) * far.length()
    };
    {
        let mut fleet = app.world_mut().resource_mut::<Fleet>();
        let r = fleet.stowed.iter_mut().find(|r| r.id == loon.id).unwrap();
        r.position = near.to_array();
    }
    // On the frame it is back, before it has stepped far.
    let mut back = None;
    for _ in 0..120 {
        app.update();
        back = crafts(&mut app).into_iter().find(|(_, c)| c.id == loon.id);
        if back.is_some() {
            break;
        }
    }
    let (_, craft) = back.expect("back");
    assert!(
        craft.reference_position().distance(near) < 1.0,
        "brought back at its record's pose"
    );
    assert!(app.world().resource::<Fleet>().stowed.is_empty());
}

/// `cities-in-the-world` task 4.2b ("you can use any boat you find"): a
/// harbour's boat, at anchor at its berth, is boarded, cast off and paddled
/// away, and after the world is put away and opened again it is where it
/// was left, the same craft, still tagged with its berth so its harbour
/// never makes it again. The walker boards from beside it; the test planet
/// has no pier.
#[test]
fn a_harbour_boat_is_paddled_away_and_kept_where_it_was_left() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (entity, loon) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    app.world_mut()
        .get_mut::<Vehicle>(entity)
        .unwrap()
        .craft
        .berth = Some((7, 3));
    board_from(&mut app, loon.exit().as_vec3());
    assert_eq!(
        app.world().resource::<Aboard>().0,
        Some(entity),
        "F boards it"
    );
    tap(&mut app, KeyCode::KeyT);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    for _ in 0..240 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyW);
    for _ in 0..60 {
        app.update();
    }
    let moved = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.id == loon.id)
        .unwrap()
        .1;
    let paddled = moved
        .reference_position()
        .distance(loon.reference_position());
    assert!(paddled > 3.0, "paddled {paddled:.1} m away from its berth");
    tap(&mut app, KeyCode::KeyF);
    assert_eq!(app.world().resource::<Aboard>().0, None, "stepped off");
    app.update();
    let left = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.id == loon.id)
        .unwrap()
        .1;
    let file = put_away(app.world_mut()).expect("a spawned fleet");
    app.world_mut()
        .resource_mut::<crate::saves::WorldSave>()
        .snapshot_vehicles(&file);
    app.update();
    let (_, back) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.id == loon.id)
        .expect("the same craft is back");
    assert!(
        back.reference_position()
            .distance(left.reference_position())
            < 0.5,
        "back where it was left"
    );
    assert_eq!(back.berth, Some((7, 3)), "still its harbour's boat");
}

/// A cog put where the Tern lies, under way at `speed` m/s with its tiller
/// hard over and nobody at its helm: sailed by its own physics, coasting
/// round a turn (from 4 m/s, through 90° in about 45 s), its deck standing
/// once a tick has passed.
fn launch_cog(app: &mut App, speed: f64) -> Entity {
    let tern = crafts(app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Tern)
        .expect("the Tern")
        .1;
    let mut cog = {
        let mut fleet = app.world_mut().resource_mut::<Fleet>();
        let id = fleet.next_id;
        fleet.next_id += 1;
        Craft::new(
            Kind::Cog,
            id,
            fleet.specs.clone(),
            fleet.hulls.clone(),
            tern.reference_position(),
            tern.body.orientation,
        )
    };
    cog.body.velocity = cog.body.axis(pbd_core::vehicle::FORWARD) * speed;
    let hard_over = cog.specs().cog.rudder_max_rad as f64;
    if let pbd_core::vehicle::CraftState::Cog(s) = &mut cog.state {
        s.tiller = hard_over;
    }
    let entity = place::spawn_craft(app.world_mut(), cog);
    app.update();
    entity
}

/// The cog's deck frame now.
fn cog_frame(app: &App, cog: Entity) -> pbd_core::settlement::pieces::Frame {
    app.world()
        .resource::<crate::decks::CraftDecks>()
        .at(cog)
        .expect("the cog holds its deck")
        .motion
        .now
}

/// The walker's feet in the cog's frame.
fn feet_on_cog(app: &mut App, cog: Entity) -> Vec3 {
    let p = walker(app);
    cog_frame(app, cog).local(p - p.normalize() * (HALF_HEIGHT + crate::walking::CONTACT_SKIN))
}

/// Stand the walker on the cog's deck with its feet at `at` in the cog's
/// frame, held by it.
fn stand_on_cog(app: &mut App, cog: Entity, at: Vec3) {
    use crate::walking::{GroundState, Piece};
    let f = cog_frame(app, cog);
    let feet = f.world(at);
    let p = feet + feet.normalize() * (HALF_HEIGHT + crate::walking::CONTACT_SKIN);
    let body = app
        .world_mut()
        .query_filtered::<Entity, With<Walker>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(body).insert((
        Position(p),
        Transform::from_translation(p),
        LinearVelocity::ZERO,
        GroundState {
            previous: p,
            grounded: true,
            on: Some(Piece::Craft(cog)),
            drift: Vec3::ZERO,
            on_deck: Some(at),
        },
    ));
}

/// Whether the walker's feet are down, what holds them, and the way it
/// keeps.
fn ground(app: &mut App) -> (bool, Option<crate::walking::Piece>, Vec3) {
    let g = app
        .world_mut()
        .query_filtered::<&crate::walking::GroundState, With<Walker>>()
        .single(app.world())
        .unwrap();
    (g.grounded, g.on, g.drift)
}

/// `sail-the-cog` step 3 (`player/walking`, standing still through a turn):
/// a walker set on the aftcastle of a cog under way, 6 m from its mast,
/// stands where it was on the deck to a centimetre, held all the way, while
/// the cog turns through 90° with nobody at its helm.
#[test]
fn a_walker_rides_a_cog_under_way_through_a_turn() {
    use crate::walking::Piece;
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = launch_cog(&mut app, 4.0);
    let at = Vec3::new(1.0, 3.5, 4.6);
    let mast = pbd_core::settlement::pieces::cog::MAST_STEP;
    assert!(Vec2::new(at.x - mast.x, at.z - mast.z).length() > 6.0);
    stand_on_cog(&mut app, cog, at);
    let f0 = cog_frame(&app, cog);
    let mut ticks = 0;
    while cog_frame(&app, cog).z.angle_between(f0.z) < std::f32::consts::FRAC_PI_2 {
        assert!(ticks < 4000, "the cog never turned a quarter round");
        app.update();
        ticks += 1;
        let (grounded, on, _) = ground(&mut app);
        assert!(
            grounded && on == Some(Piece::Craft(cog)),
            "let go at tick {ticks}"
        );
    }
    let feet = feet_on_cog(&mut app, cog);
    assert!(
        feet.distance(at) < 0.01,
        "on the deck at {feet}, set at {at}"
    );
}

/// `sail-the-cog` step 3 (`player/walking`, up the stair under way): a
/// walker climbs the cog's stair onto the aftcastle while it coasts round
/// its turn, never a tick in the air.
#[test]
fn a_walker_climbs_a_cogs_stair_under_way() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = launch_cog(&mut app, 2.5);
    // The stair rises aft from amidships on the middle line.
    stand_on_cog(&mut app, cog, Vec3::new(0.0, 1.9, -0.8));
    let body = app
        .world_mut()
        .query_filtered::<Entity, With<Walker>>()
        .single(app.world())
        .unwrap();
    app.world_mut().resource_mut::<WalkingState>().captured = true;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    let mut airborne = 0;
    let mut feet = feet_on_cog(&mut app, cog);
    let mut jump: f32 = 0.0;
    for _ in 0..400 {
        let was = feet;
        feet = feet_on_cog(&mut app, cog);
        jump = jump.max((feet.y - was.y).abs());
        if feet.z > 5.2 {
            break;
        }
        let f = cog_frame(&app, cog);
        // Aft along the middle line.
        let dir = f.z - f.x * (feet.x * 2.0);
        let p = walker(&mut app);
        app.world_mut()
            .resource_mut::<WalkingState>()
            .face(p.normalize(), dir);
        app.update();
        if !app
            .world()
            .get::<crate::walking::GroundState>(body)
            .unwrap()
            .grounded
        {
            airborne += 1;
        }
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyW);
    assert!(feet.z > 5.2, "got aft to {feet}");
    assert!(
        (feet.y - 3.5).abs() < 0.05,
        "on the aftcastle, the feet {} m over the waterline",
        feet.y
    );
    assert_eq!(airborne, 0, "ticks in the air going up");
    assert!(jump < 0.1, "the eye jumped {jump} m in a tick");
}

/// `sail-the-cog` step 3 (`player/vehicles`, letting go of the helm): F at
/// the helm of a cog under way puts the walker at the tiller on the
/// aftcastle, riding it, not left behind by its way; the cog sails on with
/// its yard and tiller as they were.
#[test]
fn letting_go_of_a_cogs_helm_leaves_the_walker_riding_its_aftcastle() {
    use crate::walking::Piece;
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = launch_cog(&mut app, 2.5);
    take_seat(app.world_mut(), cog, true);
    app.update();
    let rig = |app: &mut App| match crafts(app).into_iter().find(|(e, _)| *e == cog) {
        Some((
            _,
            Craft {
                state: pbd_core::vehicle::CraftState::Cog(s),
                ..
            },
        )) => (s.yard, s.tiller),
        _ => panic!("the cog"),
    };
    let held = rig(&mut app);
    tap(&mut app, KeyCode::KeyF);
    assert_eq!(app.world().resource::<Aboard>().0, None, "let go");
    let exit = crafts(&mut app)
        .into_iter()
        .find(|(e, _)| *e == cog)
        .unwrap()
        .1
        .specs()
        .cog
        .seat
        .exit;
    let exit = Vec3::from(exit);
    for _ in 0..180 {
        app.update();
    }
    let (grounded, on, _) = ground(&mut app);
    assert!(grounded && on == Some(Piece::Craft(cog)), "riding it");
    let feet = feet_on_cog(&mut app, cog);
    assert!(
        (feet.y - 3.5).abs() < 0.05 && Vec2::new(feet.x - exit.x, feet.z - exit.z).length() < 0.3,
        "on the aftcastle at {feet}, the tiller's exit {exit}"
    );
    assert_eq!(rig(&mut app), held, "its yard and tiller as they were left");
    let sails = crafts(&mut app)
        .into_iter()
        .find(|(e, _)| *e == cog)
        .unwrap()
        .1
        .body
        .velocity
        .length();
    assert!(sails > 1.0, "and it sails on at {sails} m/s");
}

/// `sail-the-cog` step 3 (`player/walking`, over the side): a walker goes
/// out through the gangway of a cog under way, leaves the deck with the
/// deck's way there as drift, and the water takes it off; the cog sails on.
#[test]
fn a_walker_goes_over_a_cogs_side_with_its_way() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = launch_cog(&mut app, 2.5);
    stand_on_cog(&mut app, cog, Vec3::new(-0.8, 1.9, 0.0));
    app.world_mut().resource_mut::<WalkingState>().captured = true;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    let mut left: Option<f32> = None;
    let mut way = 0.0;
    for _ in 0..600 {
        let f = cog_frame(&app, cog);
        let p = walker(&mut app);
        // To port, through the gangway, and on away from the ship.
        app.world_mut()
            .resource_mut::<WalkingState>()
            .face(p.normalize(), -f.x);
        let before = f.origin;
        app.update();
        let (_, on, drift) = ground(&mut app);
        if left.is_none() && on.is_none() {
            let speed = cog_frame(&app, cog).origin.distance(before) * 60.0;
            left = Some(drift.length());
            way = speed;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyW);
        }
        if left.is_some() && drift.length() < 0.05 {
            break;
        }
    }
    let left = left.expect("went over the side");
    assert!(
        (left - way).abs() < 0.25 * way,
        "left with {left} m/s of the ship's {way} m/s"
    );
    assert!(ground(&mut app).2.length() < 0.05, "the water took it off");
    let sails = crafts(&mut app)
        .into_iter()
        .find(|(e, _)| *e == cog)
        .unwrap()
        .1
        .body
        .velocity
        .length();
    assert!(sails > 1.0, "and the cog sails on at {sails} m/s");
}

/// A harbour's cog made fast at its berth where the Tern lies, its berth
/// worked out as if from its town.
fn moor_cog(app: &mut App) -> Entity {
    let tern = crafts(app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Tern)
        .expect("the Tern")
        .1;
    let mut cog = {
        let mut fleet = app.world_mut().resource_mut::<Fleet>();
        let id = fleet.next_id;
        fleet.next_id += 1;
        Craft::new(
            Kind::Cog,
            id,
            fleet.specs.clone(),
            fleet.hulls.clone(),
            tern.reference_position(),
            tern.body.orientation,
        )
    };
    cog.berth = Some((7, harbour::COG));
    cog.mooring = Some(pbd_core::vehicle::Mooring {
        at: cog.bow(),
        length: 2.0,
        anchored: false,
    });
    let origin = cog.reference_position().as_vec3();
    let bow = cog.body.axis(pbd_core::vehicle::FORWARD).as_vec3();
    let bow = (bow - origin.normalize() * bow.dot(origin.normalize())).normalize();
    let entity = place::spawn_craft(app.world_mut(), cog);
    app.world_mut()
        .entity_mut(entity)
        .insert(harbour::Berthed::new(origin, bow));
    entity
}

fn the_craft(app: &mut App, entity: Entity) -> Craft {
    crafts(app)
        .into_iter()
        .find(|(e, _)| *e == entity)
        .expect("the craft")
        .1
}

/// How far a cog lies from its swing at its berth now, m.
fn off_swing(app: &mut App, cog: Entity) -> f32 {
    let t = app.world().resource::<Time<Fixed>>().elapsed_secs();
    let berthed = *app.world().get::<harbour::Berthed>(cog).unwrap();
    let want = crate::decks::Swing::MOORED.at(&berthed.rest, berthed.bow, t);
    let craft = the_craft(app, cog);
    let f = crate::decks::craft_frame(&craft, bevy::math::DVec3::ZERO);
    f.origin.distance(want.origin) + f.x.distance(want.x) * 10.0
}

/// `sail-the-cog` step 3, part 3: a harbour's cog made fast at its berth
/// rides the mooring swing there, to a millimetre, and is not stepped; T at
/// its helm casts it off to its own physics; brought back slow within a few
/// metres of its berth, T makes it fast and it eases back onto the swing;
/// far from its berth, T anchors it as a boat.
#[test]
fn a_harbours_cog_rides_its_swing_casts_off_and_makes_fast_again() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = moor_cog(&mut app);
    for _ in 0..120 {
        app.update();
        assert!(off_swing(&mut app, cog) < 1e-3, "off its swing");
    }
    assert!(harbour::on_swing(&the_craft(&mut app, cog)));
    // Cast off, and give it way: it goes where its physics takes it.
    take_seat(app.world_mut(), cog, true);
    tap(&mut app, KeyCode::KeyT);
    let cast = the_craft(&mut app, cog);
    assert!(cast.mooring.is_none(), "cast off");
    assert!(cast.body.velocity.length() < 0.3, "at the swing's way");
    let rest = app.world().get::<harbour::Berthed>(cog).unwrap().rest;
    let bow = app.world().get::<harbour::Berthed>(cog).unwrap().bow;
    app.world_mut()
        .get_mut::<Vehicle>(cog)
        .unwrap()
        .craft
        .body
        .velocity = bow.as_dvec3();
    for _ in 0..120 {
        app.update();
    }
    let gone = the_craft(&mut app, cog)
        .reference_position()
        .distance(rest.origin.as_dvec3());
    assert!(gone > 1.5, "sailed {gone:.2} m off its berth");
    // Back 2 m off its berth and still: T makes it fast, and it eases on.
    let put = |app: &mut App, along: f32| {
        let mut v = app.world_mut().get_mut::<Vehicle>(cog).unwrap();
        let q = v.craft.body.orientation;
        v.craft
            .set_reference_pose((rest.origin + bow * along).as_dvec3(), q);
        v.craft.body.velocity = bevy::math::DVec3::ZERO;
        v.craft.body.angular_velocity = bevy::math::DVec3::ZERO;
    };
    put(&mut app, 2.0);
    tap(&mut app, KeyCode::KeyT);
    let fast = the_craft(&mut app, cog);
    assert!(fast.mooring.is_some_and(|m| !m.anchored), "made fast");
    assert!(off_swing(&mut app, cog) > 1.0, "not snapped onto the swing");
    for _ in 0..(harbour::EASE_S * 60.0) as usize + 30 {
        app.update();
    }
    assert!(off_swing(&mut app, cog) < 1e-3, "eased onto its swing");
    // Far from its berth, T anchors it.
    tap(&mut app, KeyCode::KeyT);
    put(&mut app, 25.0);
    tap(&mut app, KeyCode::KeyT);
    assert!(
        the_craft(&mut app, cog).mooring.is_some_and(|m| m.anchored),
        "anchored, 25 m off its berth"
    );
}

/// `sail-the-cog` step 3, part 3: a harbour's cog made fast at its berth is
/// saved on its bollard with its berth, and comes back on its swing there.
#[test]
fn a_moored_cog_is_saved_on_its_bollard_at_its_berth() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = moor_cog(&mut app);
    for _ in 0..30 {
        app.update();
    }
    let was = the_craft(&mut app, cog);
    let rest = app.world().get::<harbour::Berthed>(cog).unwrap().rest;
    let file = put_away(app.world_mut()).expect("a spawned fleet");
    app.world_mut()
        .resource_mut::<crate::saves::WorldSave>()
        .snapshot_vehicles(&file);
    app.update();
    let (_, back) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.id == was.id)
        .expect("the cog is back");
    assert_eq!(back.kind, Kind::Cog);
    assert_eq!(back.berth, Some((7, harbour::COG)));
    assert!(harbour::on_swing(&back), "on its bollard, on its swing");
    let off = back.reference_position().distance(rest.origin.as_dvec3());
    assert!(off < 0.2, "at its berth, {off:.3} m off");
}

/// `sail-the-cog` step 3, part 4: a cog's stern lantern burns while the
/// dusk lamps do and is dark by day, and its light on the ship's faces goes
/// where the ship does.
#[test]
fn a_cogs_stern_lantern_burns_from_dusk_and_its_light_goes_with_it() {
    use crate::planet::lod::DuskLamps;
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let cog = launch_cog(&mut app, 2.5);
    let flame = |app: &mut App| {
        let mut q = app
            .world_mut()
            .query_filtered::<(&Visibility, &ChildOf), With<draw::LanternFlame>>();
        let found: Vec<Visibility> = q
            .iter(app.world())
            .filter(|(_, p)| p.parent() == cog)
            .map(|(v, _)| *v)
            .collect();
        assert_eq!(found.len(), 1, "one flame in its lantern");
        found[0]
    };
    app.insert_resource(DuskLamps { lit: true });
    app.update();
    assert_eq!(flame(&mut app), Visibility::Inherited, "lit at dusk");
    app.insert_resource(DuskLamps { lit: false });
    app.update();
    assert_eq!(flame(&mut app), Visibility::Hidden, "out by day");
    for _ in 0..120 {
        app.update();
    }
    let craft = the_craft(&mut app, cog);
    let want = (craft.reference_position()
        + craft.body.orientation * pbd_core::settlement::pieces::cog::LANTERN.as_dvec3())
    .as_vec3();
    let mut q = app
        .world_mut()
        .query::<(&draw::CarriedLight, &crate::field_light::RoomLights)>();
    let lights: Vec<Vec3> = q
        .iter(app.world())
        .filter(|(c, _)| c.owner == cog)
        .map(|(_, l)| l.0[0].at)
        .collect();
    assert!(!lights.is_empty(), "its faces carry its light");
    for at in lights {
        assert!(
            at.distance(want) < 0.01,
            "the light at {at}, the lantern at {want}"
        );
    }
}
