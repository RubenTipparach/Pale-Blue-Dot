//! The app's side of the craft, driven through the real plugin: they are
//! placed where they can work, boarding hands the player over and back, and a
//! craft left behind is still there, in the save, with its ID.

use super::*;
use crate::flight_view::{FlightViewConfig, FlightViewPlugin};
use crate::planet::PlanetContact;
use crate::planet::terrain::PLANET_RADIUS;
use crate::walking::{EYE_HEIGHT, WalkingPlugin};
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

/// A tap: down for one update and up on the next, which is when G acts, since
/// a hold of it is the tool picker (`controls::InteractKey`).
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
    assert_eq!(app.world().resource::<Aboard>().0, Some(entity), "G boards");
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
    tap(&mut app, KeyCode::KeyG);
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
    // And G boards it where it was left.
    let (entity, craft) = back.into_iter().find(|(_, c)| c.id == kestrel.id).unwrap();
    board_from(&mut app, craft.exit().as_vec3());
    assert_eq!(
        app.world().resource::<Aboard>().0,
        Some(entity),
        "boardable again"
    );
}

/// Stand at `at` and tap G. Boarding happens on the RELEASE, since a hold of
/// G is the tool picker, so the walker is put back on the spot for both
/// frames: a walker set down beside a rocking boat has had a frame to slide
/// off it by the time the key comes up.
fn board_from(app: &mut App, at: Vec3) {
    stand_at(app, at);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyG);
    app.update();
    stand_at(app, at);
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.clear();
    keys.release(KeyCode::KeyG);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
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
    tap(&mut app, KeyCode::KeyG);
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
            .press(KeyCode::KeyG);
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
        "G boards the Loon"
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

/// Holding G beside a craft is the tool picker, not a boarding: the craft
/// reads only a tap.
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
    let frames = (crate::controls::PICKER_HOLD_S * crate::FIXED_HZ as f32).ceil() as usize + 2;
    for _ in 0..frames {
        stand_at(&mut app, at);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    assert!(
        app.world()
            .resource::<crate::controls::InteractKey>()
            .holding,
        "the picker is open"
    );
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::KeyG);
    stand_at(&mut app, at);
    app.update();
    assert_eq!(
        app.world().resource::<Aboard>().0,
        None,
        "a hold boards nothing"
    );
}
