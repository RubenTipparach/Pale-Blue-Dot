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
        .insert_resource(super::place::StarterLoon)
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
/// T weighs the anchor (`river-craft` design 4): the rode is hauled in to
/// straight up and down and the anchor lifted aboard, some seconds of
/// updates; then the boat is free.
fn weigh(app: &mut App, entity: Entity) {
    tap(app, KeyCode::KeyT);
    for _ in 0..3600 {
        let free = app
            .world()
            .get::<Vehicle>(entity)
            .is_some_and(|v| v.craft.anchor.is_none() && v.craft.mooring.is_none());
        if free {
            return;
        }
        app.update();
    }
    panic!("the anchor never came aboard");
}

/// The anchor let go falls to the bed and sets: a couple of seconds.
fn settle_anchor(app: &mut App, entity: Entity) {
    for _ in 0..1200 {
        if app
            .world()
            .get::<Vehicle>(entity)
            .is_some_and(|v| v.craft.anchored())
        {
            return;
        }
        app.update();
    }
    panic!("the anchor never set");
}

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

/// Deep water near the Loon, with its bow out along the way it was found:
/// where a test puts a craft that wants more water than the Loon's berth
/// (where the retired Tern lay, 14 m off the Loon in 3 m of water).
fn deep_berth(app: &mut App) -> (pbd_core::DVec3, pbd_core::DQuat) {
    let loon = crafts(app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .expect("the Loon")
        .1;
    let sea = app.world().resource::<Sea>();
    place::deep_berth_near(
        sea,
        loon.reference_position().normalize().as_vec3(),
        14.0,
        3.0,
    )
    .expect("deep water near the Loon")
}

/// The new world's Loon moved to [`deep_berth`] and anchored there: clear of
/// the shore its own berth lies off, as the retired Tern was, for a test that
/// sails it about or swings a chase camera round it.
fn loon_in_deep_water(app: &mut App) -> (Entity, Craft) {
    let (at, facing) = deep_berth(app);
    let (entity, _) = crafts(app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .expect("the Loon");
    let bed = f64::from(place::floor(at.normalize().as_vec3()));
    let mut vehicle = app.world_mut().get_mut::<Vehicle>(entity).unwrap();
    let craft = &mut vehicle.craft;
    craft.set_reference_pose(at, facing);
    craft.body.velocity = pbd_core::DVec3::ZERO;
    craft.body.angular_velocity = pbd_core::DVec3::ZERO;
    craft.mooring = Some(pbd_core::vehicle::Mooring {
        at: craft.bow().normalize() * bed,
        length: place::rode(at.length() - bed),
        anchored: true,
    });
    let craft = craft.clone();
    app.update();
    (entity, craft)
}

fn walker(app: &mut App) -> Vec3 {
    let mut query = app.world_mut().query_filtered::<&Position, With<Walker>>();
    query.single(app.world()).unwrap().0
}

/// A new world's fleet is the boats (`retire-kestrel`, `retire-tern`): the
/// Loon, afloat, at anchor, in water deep enough for it, and no Tern or
/// aircraft.
#[test]
fn a_new_world_places_each_craft_where_it_can_work() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    assert!(app.world().resource::<Fleet>().spawned);
    let all = crafts(&mut app);
    let kinds: Vec<Kind> = all.iter().map(|(_, c)| c.kind).collect();
    assert_eq!(kinds, [Kind::Loon], "the Loon, and no Tern");
    let sea = app.world().resource::<Sea>().clone();
    for (_, craft) in &all {
        let at = craft.reference_position().as_vec3();
        let depth = sea.depth_at(at.normalize());
        assert!(depth >= 1.2, "{} in {depth} m", craft.kind.name());
        assert!(craft.mooring.is_some_and(|m| m.anchored), "and at anchor");
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
        let lift = craft.reference_position().length() - sea.radius as f64;
        assert!(
            lift.abs() < 0.5,
            "{} floats at the sea: {lift:.2} m",
            craft.kind.name()
        );
    }
    // The new fleet is written, not left owed.
    assert!(!app.world().resource::<Fleet>().dirty);
    let saved = app
        .world()
        .resource::<crate::saves::WorldSave>()
        .vehicles
        .clone();
    assert_eq!(saved.map(|f| f.vehicles.len()), Some(1));
}

/// Board, get under way, step off: the craft changes occupancy and never
/// existence. It stays where the unattended policy puts it, keeps its ID, is
/// written to the save, and comes back from that save after the world is put
/// away. The Loon stands for any craft (it was the Kestrel, in the air,
/// until `retire-kestrel`, then the Tern until `retire-tern`).
#[test]
fn a_craft_left_behind_stays_boardable_and_comes_back_from_its_save() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (entity, loon) = loon_in_deep_water(&mut app);
    // Walk up to it and board.
    board_from(&mut app, loon.exit().as_vec3());
    assert_eq!(app.world().resource::<Aboard>().0, Some(entity), "F boards");
    assert!(!app.world().resource::<WalkingState>().active);
    let mut cameras = app
        .world_mut()
        .query_filtered::<&Camera, With<VehicleCamera>>();
    assert!(
        cameras.single(app.world()).unwrap().is_active,
        "its camera is the view"
    );
    // Weigh anchor and get under way: two metres a second along the bow,
    // for a couple of seconds.
    weigh(&mut app, entity);
    {
        let mut vehicle = app.world_mut().get_mut::<Vehicle>(entity).unwrap();
        let craft = &mut vehicle.craft;
        assert!(craft.mooring.is_none(), "T weighs the anchor");
        craft.body.velocity = craft.body.axis(pbd_core::vehicle::FORWARD) * 2.0;
    }
    for _ in 0..60 {
        app.update();
    }
    let sailed = app.world().get::<Vehicle>(entity).unwrap().craft.clone();
    let moved = sailed
        .reference_position()
        .distance(loon.reference_position());
    assert!(moved > 0.5, "under way: {moved:.2} m");
    assert!(sailed.body.velocity.length() > 0.1, "still making way");
    // Step off.
    tap(&mut app, KeyCode::KeyF);
    assert_eq!(app.world().resource::<Aboard>().0, None);
    assert!(
        app.world().resource::<WalkingState>().active,
        "back on foot"
    );
    let left = crafts(&mut app);
    assert_eq!(left.len(), 1, "nothing was despawned");
    let (still, craft) = left.iter().find(|(_, c)| c.id == loon.id).unwrap();
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
    let record = saved.vehicles.iter().find(|r| r.id == loon.id).unwrap();
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
    let restored = &back.iter().find(|(_, c)| c.id == loon.id).unwrap().1;
    let at = pbd_core::DVec3::from(record.position);
    assert!(
        restored.reference_position().distance(at) < 0.5,
        "back where it was left"
    );
    assert!(app.world().resource::<Fleet>().next_id > loon.id);
    // And F boards it where it was left.
    let (entity, craft) = back.into_iter().find(|(_, c)| c.id == loon.id).unwrap();
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
    let (entity, craft) = loon_in_deep_water(&mut app);
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
    // The starting fleet has no tube or raft (they are made), so one of
    // each is put in deep water near the Loon, for its keys.
    let (at, facing) = deep_berth(&mut app);
    for kind in [Kind::Tube, Kind::Raft] {
        let craft = {
            let mut fleet = app.world_mut().resource_mut::<Fleet>();
            let id = fleet.next_id;
            fleet.next_id += 1;
            pbd_core::vehicle::Craft::new(
                kind,
                id,
                fleet.specs.clone(),
                fleet.hulls.clone(),
                at,
                facing,
            )
        };
        super::place::spawn_craft(app.world_mut(), craft);
    }
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
            Kind::Tube => assert_eq!(
                (input.forward, input.steer, input.rudder, input.bail),
                (1.0, 1.0, 0.0, true)
            ),
            Kind::Loon | Kind::Raft => assert_eq!(
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
    weigh(&mut app, entity);
    assert!(
        saved(&app).mooring.is_none(),
        "weighed, and saved in the same update"
    );
    tap(&mut app, KeyCode::KeyT);
    settle_anchor(&mut app, entity);
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
    weigh(&mut app, entity);
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

/// `town-frame-time` decision 6: a resting craft far from the camera sleeps,
/// is stepped once every `SLEEP_CHECK_TICKS` to see whether it has moved, and
/// never sleeps while aboard, near the camera, or moving.
#[test]
fn a_resting_craft_far_off_sleeps_and_is_checked() {
    use pbd_core::vehicle::{Hulls, Kind, spec::VehicleSpecs};
    use pbd_core::{DQuat, DVec3};
    let specs = std::sync::Arc::new(VehicleSpecs::default());
    let mut v = Vehicle::new(Craft::new(
        Kind::Loon,
        1,
        specs.clone(),
        Hulls::new(&specs),
        DVec3::Y * 4800.0,
        DQuat::IDENTITY,
    ));
    let far = Some(DVec3::Y * 4800.0 + DVec3::X * 100.0);
    let near = Some(DVec3::Y * 4800.0 + DVec3::X * 10.0);
    // Moving: never sleeps.
    assert!(!sleeps(&mut v, false, far));
    v.rested = true;
    // Aboard or near: never sleeps, and the count starts again.
    assert!(!sleeps(&mut v, true, far));
    assert!(!sleeps(&mut v, false, near));
    // Far and at rest: sleeps, but is stepped on every SLEEP_CHECK_TICKS-th.
    let stepped = (0..SLEEP_CHECK_TICKS * 3)
        .filter(|_| !sleeps(&mut v, false, far))
        .count();
    assert_eq!(stepped, 3);
    // A check step that finds it moving ends the rest: stepped every tick.
    v.rested = false;
    assert!(!sleeps(&mut v, false, far));
    assert_eq!(v.dozing, 0);
}

/// The vehicle file of a world made before `retire-kestrel`, as that build
/// wrote it to the disk: a Kestrel, a Tern and a Loon, layout 1 (tasks.md
/// 1.1; how it was made is in `fixtures/README.md`).
const KESTREL_ERA: &str = include_str!("fixtures/kestrel-era-vehicles.ron");

/// One record's text in a vehicle file as written: from the `(` that opens
/// it to the `),` that closes it, found by its kind.
fn record_text<'a>(file: &'a str, kind: &str) -> &'a str {
    let at = file
        .find(&format!("kind: \"{kind}\""))
        .unwrap_or_else(|| panic!("no {kind} in the file"));
    let start = file[..at].rfind("(\n").expect("the record opens");
    let end = at + file[at..].find("\n        ),").expect("the record closes");
    &file[start..end]
}

/// Tasks.md 1.2 (`retire-kestrel` decision 1, `retire-tern` 1.1): a world
/// saved with a Kestrel and a Tern opens with its Loon and neither retired
/// craft in it; their records are held aside, and after the Loon is moved
/// and the fleet written again they are in the file byte for byte as they
/// were read, and no new craft can take their IDs.
#[test]
fn a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record() {
    let old: VehicleFile = ron::from_str(KESTREL_ERA).expect("layout 1 still reads");
    assert_eq!(old.version, 1);
    assert_eq!(RECORD_VERSION, 1, "no layout change");
    let held: Vec<_> = old
        .vehicles
        .iter()
        .filter(|r| r.kind == "kestrel" || r.kind == "tern")
        .cloned()
        .collect();
    assert_eq!(held.len(), 2, "the fixture has a Kestrel and a Tern");
    let root = std::env::temp_dir().join(format!("pbd-retire-kestrel-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let slot = crate::saves::create(&root, "Kestrel era", 1).unwrap();
    let path = root.join(&slot.id).join(crate::saves::VEHICLES);
    std::fs::write(&path, KESTREL_ERA).unwrap();
    let mut app = app(crate::saves::WorldSave::open(root.clone(), slot));
    app.update();
    // Its Loon, stowed where it lies far from this test's walker and in the
    // world when the walker is near, and no Kestrel or Tern in either.
    let mut ids: Vec<u64> = crafts(&mut app).iter().map(|(_, c)| c.id).collect();
    ids.extend(app.world().resource::<Fleet>().stowed.iter().map(|r| r.id));
    ids.sort();
    let loon_id = old.vehicles.iter().find(|r| r.kind == "loon").unwrap().id;
    assert_eq!(ids, [loon_id], "the world opens with one craft, the Loon");
    // This test's planet starts its walker elsewhere than the world the file
    // came from, so the Loon is stowed. Bring its record near the walker, as
    // coming near it does, and it is in the world.
    let near = {
        let w = walker(&mut app).as_dvec3();
        let up = w.normalize();
        let angle = 60.0 / w.length();
        (up * angle.cos() + up.any_orthonormal_vector() * angle.sin()) * w.length()
    };
    {
        let mut fleet = app.world_mut().resource_mut::<Fleet>();
        if let Some(r) = fleet.stowed.iter_mut().find(|r| r.id == loon_id) {
            r.position = near.to_array();
        }
    }
    let mut all = Vec::new();
    for _ in 0..120 {
        app.update();
        all = crafts(&mut app);
        if all.iter().any(|(_, c)| c.id == loon_id) {
            break;
        }
    }
    assert!(
        all.iter().any(|(_, c)| c.id == loon_id),
        "the Loon is in the world near the walker"
    );
    let mut names = app.world_mut().query::<&Name>();
    assert!(
        names
            .iter(app.world())
            .all(|n| n.as_str() != "Kestrel" && n.as_str() != "Tern"),
        "no Kestrel or Tern is drawn"
    );
    let fleet = app.world().resource::<Fleet>();
    assert_eq!(fleet.retired, held, "held aside as read");
    assert!(
        held.iter().all(|r| fleet.next_id > r.id),
        "their IDs are never given again"
    );
    // Move the Loon and save.
    let (entity, loon) = all.into_iter().find(|(_, c)| c.id == loon_id).unwrap();
    {
        let mut vehicle = app.world_mut().get_mut::<Vehicle>(entity).unwrap();
        let moved = loon.reference_position() + loon.body.axis(pbd_core::vehicle::FORWARD) * 3.0;
        vehicle
            .craft
            .set_reference_pose(moved, loon.body.orientation);
    }
    app.world_mut().resource_mut::<Fleet>().dirty = true;
    app.update();
    app.world().resource::<crate::saves::WorldSave>().drain();
    let written = std::fs::read_to_string(&path).unwrap();
    let file: VehicleFile = ron::from_str(&written).unwrap();
    assert_eq!(file.version, 1);
    let loon_now = file.vehicles.iter().find(|r| r.id == loon_id).unwrap();
    let was = old.vehicles.iter().find(|r| r.id == loon_id).unwrap();
    assert_ne!(loon_now.position, was.position, "the Loon's move is saved");
    for kind in ["kestrel", "tern"] {
        assert_eq!(
            record_text(&written, kind),
            record_text(KESTREL_ERA, kind),
            "the {kind}'s record is byte-identical"
        );
    }
    assert!(
        file.vehicles.ends_with(&held),
        "after the live ones, in the order read"
    );
    assert!(
        held.iter().all(|r| file.next_id > r.id),
        "next_id never falls below them"
    );
    assert!(file.vehicles.iter().all(|r| r.id <= file.next_id));
    // And a world put away and opened again keeps them the same way.
    let put = put_away(app.world_mut()).expect("a spawned fleet");
    assert!(put.vehicles.ends_with(&held));
    let _ = std::fs::remove_dir_all(&root);
}

/// The vehicle file of a world made by the build before `retire-tern`, as
/// that build wrote it to the disk: the Tern and the Loon it placed, and a
/// harbour's Tern and rowboat, each with its berth (`fixtures/README.md`).
const TERN_ERA: &str = include_str!("fixtures/tern-era-vehicles.ron");

/// `retire-tern` 1.2: every Tern in a world saved before the Tern was
/// retired, the one placed by the new world and the harbour's, is held aside
/// and written back byte for byte; the harbour's rowboat is held too
/// (`retire-rowboat-and-cog`); a held
/// harbour Tern's berth counts as made, so it is never filled again; and
/// `next_id` stays above them.
#[test]
fn a_world_saved_with_harbour_terns_keeps_every_record() {
    let old: VehicleFile = ron::from_str(TERN_ERA).expect("layout 1 still reads");
    let terns: Vec<_> = old
        .vehicles
        .iter()
        .filter(|r| r.kind == "tern")
        .cloned()
        .collect();
    assert_eq!(terns.len(), 2, "the placed Tern and a harbour's");
    let berth = terns
        .iter()
        .find_map(|r| r.berth)
        .expect("a harbour's Tern has its berth");
    let root = std::env::temp_dir().join(format!("pbd-retire-tern-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let slot = crate::saves::create(&root, "Tern era", 1).unwrap();
    let path = root.join(&slot.id).join(crate::saves::VEHICLES);
    std::fs::write(&path, TERN_ERA).unwrap();
    let mut app = app(crate::saves::WorldSave::open(root.clone(), slot));
    app.update();
    let mut kinds: Vec<&str> = crafts(&mut app).iter().map(|(_, c)| c.kind.key()).collect();
    let fleet = app.world().resource::<Fleet>();
    kinds.extend(fleet.stowed.iter().map(|r| r.kind.as_str()));
    kinds.sort();
    assert_eq!(kinds, ["loon"], "the live craft, and no Tern or rowboat");
    // The harbour's rowboat is retired too (`retire-rowboat-and-cog`).
    let held: Vec<_> = old
        .vehicles
        .iter()
        .filter(|r| r.kind == "tern" || r.kind == "rowboat")
        .cloned()
        .collect();
    assert_eq!(
        fleet.retired, held,
        "every Tern and rowboat held aside as read"
    );
    assert!(fleet.next_id >= old.next_id);
    assert!(
        harbour::made(app.world_mut()).contains(&berth),
        "a held Tern's berth is not filled again"
    );
    // Written again, every Tern is in the file as it was read.
    app.world_mut().resource_mut::<Fleet>().dirty = true;
    app.update();
    app.world().resource::<crate::saves::WorldSave>().drain();
    let written = std::fs::read_to_string(&path).unwrap();
    let file: VehicleFile = ron::from_str(&written).unwrap();
    assert!(file.vehicles.ends_with(&held), "{written}");
    assert_eq!(file.vehicles.len(), old.vehicles.len(), "nothing dropped");
    assert!(file.next_id >= old.next_id);
    let _ = std::fs::remove_dir_all(&root);
}

/// `retire-tern` decision 2: a walker loaded over the sea within reach of a
/// held retired craft was aboard it, and is put ashore; on land, or far
/// from one, it is left where the save put it.
#[test]
fn a_walker_at_a_retired_craft_over_the_sea_goes_ashore() {
    let old: VehicleFile = ron::from_str(TERN_ERA).unwrap();
    let held: Vec<_> = old
        .vehicles
        .into_iter()
        .filter(|r| r.kind == "tern")
        .collect();
    let seat = pbd_core::DVec3::from(held[0].position) + pbd_core::DVec3::X * 1.5;
    let found = place::at_a_retired_craft(seat, &held, |_| true);
    assert_eq!(found.map(|r| r.id), Some(held[0].id), "aboard: ashore");
    assert!(
        place::at_a_retired_craft(seat, &held, |_| false).is_none(),
        "on land: left"
    );
    // `retire-rowboat-and-cog` decision 4: on a held cog's deck, 7.5 m from
    // its middle, is aboard it; as far from a held rowboat is not.
    let mut cog = held[0].clone();
    cog.kind = "cog".into();
    let bow = pbd_core::DVec3::from(cog.position) + pbd_core::DVec3::Z * 7.5;
    assert_eq!(
        place::at_a_retired_craft(bow, std::slice::from_ref(&cog), |_| true).map(|r| r.id),
        Some(cog.id),
        "on a cog's deck: ashore"
    );
    let mut rowboat = cog.clone();
    rowboat.kind = "rowboat".into();
    assert!(
        place::at_a_retired_craft(bow, std::slice::from_ref(&rowboat), |_| true).is_none(),
        "7.5 m off a rowboat: left"
    );
    let far = pbd_core::DVec3::from(held[0].position) + pbd_core::DVec3::X * 40.0;
    assert!(
        place::at_a_retired_craft(far, &held, |_| true).is_none(),
        "swimming elsewhere: left"
    );
    // In the game: a world whose walker was saved at its Tern, on this
    // test's planet, loads on dry land.
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let (at, facing) = deep_berth(&mut app);
    let record = {
        let fleet = app.world().resource::<Fleet>();
        let mut tern = Craft::new(
            Kind::Loon,
            50,
            fleet.specs.clone(),
            fleet.hulls.clone(),
            at,
            facing,
        )
        .record();
        tern.kind = "tern".into();
        tern
    };
    let mut file = put_away(app.world_mut()).expect("a spawned fleet");
    file.vehicles.push(record);
    file.next_id = 51;
    app.world_mut()
        .resource_mut::<crate::saves::WorldSave>()
        .snapshot_vehicles(&file);
    let frame = app
        .world()
        .resource::<crate::planet::PlanetRenderFrame>()
        .physics_center();
    stand_at(&mut app, (frame + at).as_vec3());
    let before = walker(&mut app).as_dvec3() - frame;
    let sea_radius = app.world().resource::<Sea>().radius;
    assert!(
        place::floor(before.normalize().as_vec3()) < sea_radius,
        "the walker starts over the sea"
    );
    app.update();
    let frame = app
        .world()
        .resource::<crate::planet::PlanetRenderFrame>()
        .physics_center();
    let after = walker(&mut app).as_dvec3() - frame;
    assert!(
        place::floor(after.normalize().as_vec3()) >= sea_radius,
        "put on dry land: {:.1} m from where it was",
        after.distance(before)
    );
    assert!(
        crafts(&mut app).iter().all(|(_, c)| c.id != 50),
        "no Tern drawn"
    );
}

/// `retire-kestrel` decision 1: a held record's ID is never given again,
/// even where the file's own `next_id` has fallen behind it.
#[test]
fn a_held_record_keeps_its_id_from_every_new_craft() {
    let mut old: VehicleFile = ron::from_str(KESTREL_ERA).unwrap();
    let kestrel = old
        .vehicles
        .iter_mut()
        .find(|r| r.kind == "kestrel")
        .unwrap();
    kestrel.id = 9;
    old.next_id = 4;
    let mut save = crate::saves::WorldSave::memory_only();
    assert!(save.snapshot_vehicles(&old));
    let mut app = app(save);
    app.update();
    let fleet = app.world().resource::<Fleet>();
    assert_eq!(fleet.retired.len(), 2, "the Kestrel and the Tern");
    assert!(
        fleet.next_id >= 10,
        "next_id {} is not past 9",
        fleet.next_id
    );
    let file = fleet.file(std::iter::empty());
    assert!(file.next_id >= 10);
}

/// The moving things' light (`lamps-and-lanterns` decision 10), pinned on
/// the boats now that the Kestrel and its cave rig are gone
/// (`retire-kestrel` decision 5): every craft placed is lit by the field,
/// and as the terrain is, by its own sun and shadows (`craft-lighting`).
#[test]
fn every_craft_is_lit_by_the_field() {
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.update();
    let mut lit = app.world_mut().query_filtered::<&Vehicle, (
        With<crate::field_light::LitByField>,
        With<crate::field_light::LitLikeTerrain>,
    )>();
    let kinds: Vec<Kind> = lit.iter(app.world()).map(|v| v.craft.kind).collect();
    assert_eq!(kinds.len(), crafts(&mut app).len());
    assert!(kinds.contains(&Kind::Loon));
}

/// Interact on F (owner, 2026-11-16): beside a craft with nothing usable
/// aimed, the prompt names the craft and F boards it; with something usable
/// aimed (a ripe crop here, offered as the farm offers it), F does that and
/// the craft is left where it is.
#[test]
fn f_boards_a_craft_only_when_nothing_usable_is_aimed() {
    use crate::interact::{Act, Interact, InteractSet, Offer};
    #[derive(Resource, Default)]
    struct CropAimed(bool);
    /// Where the walker is held for the offers (the walker settles back
    /// between a test's teleport and the offers otherwise).
    #[derive(Resource, Default)]
    struct Pin(Option<Vec3>);
    let mut app = app(crate::saves::WorldSave::memory_only());
    app.init_resource::<CropAimed>()
        .init_resource::<Pin>()
        .add_systems(
            PostUpdate,
            (
                (|pin: Res<Pin>,
                  frame: Res<crate::planet::PlanetRenderFrame>,
                  mut walkers: Query<&mut Position, With<Walker>>| {
                    if let Some(at) = pin.0 {
                        let up = (at.as_dvec3() - frame.center).normalize().as_vec3();
                        for mut p in &mut walkers {
                            p.0 = at + up * crate::walking::HALF_HEIGHT;
                        }
                    }
                })
                .before(InteractSet::Offer),
                (|aimed: Res<CropAimed>, mut interact: ResMut<Interact>| {
                    if aimed.0 {
                        interact.offer(Offer::new(Act::Harvest, "wheat"));
                    }
                })
                .in_set(InteractSet::Offer),
            ),
        );
    app.update();
    let (entity, loon) = crafts(&mut app)
        .into_iter()
        .find(|(_, c)| c.kind == Kind::Loon)
        .unwrap();
    let at = loon.exit().as_vec3();
    // Stand by the craft for a frame, so the offers are seen, and press F.
    let press_beside = |app: &mut App| {
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        // The readout the prompt reads (a test world is never "open", so
        // the walker does not publish it).
        let mut readout = app
            .world_mut()
            .resource_mut::<crate::walking::WalkingReadout>();
        (readout.active, readout.captured) = (true, true);
        app.world_mut().resource_mut::<Pin>().0 = Some(at);
        stand_at(app, at);
        app.update();
        stand_at(app, at);
        let now = app.world().resource::<Interact>().now.clone();
        tap(app, KeyCode::KeyF);
        app.world_mut().resource_mut::<Pin>().0 = None;
        now
    };
    // A ripe crop aimed beside the craft: the prompt names the crop, and F
    // leaves the craft alone.
    app.world_mut().resource_mut::<CropAimed>().0 = true;
    assert_eq!(
        press_beside(&mut app),
        Some(Offer::new(Act::Harvest, "wheat"))
    );
    assert_eq!(app.world().resource::<Aboard>().0, None, "F harvested");
    // Nothing usable aimed: the prompt names the craft, and F boards it.
    app.world_mut().resource_mut::<CropAimed>().0 = false;
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(press_beside(&mut app), Some(Offer::new(Act::Board, "loon")));
    assert_eq!(app.world().resource::<Aboard>().0, Some(entity), "F boards");
}

/// The vehicle file of a world made by the build before
/// `retire-rowboat-and-cog`, as that build wrote it to the disk: its Loon,
/// Marenstrand's two rowboats, canoe and cog, and a raft the player made
/// whose kind was edited to `rowboat`, the made rowboat's record
/// (`fixtures/README.md`).
const ROWBOAT_ERA: &str = include_str!("fixtures/rowboat-era-vehicles.ron");

/// The test world over `ROWBOAT_ERA`, on disk at `root`, with the player's
/// slots `slots` written as a pack line, so the save holds them.
fn rowboat_era(
    root: &std::path::Path,
    slots: &pbd_core::inventory::Slots,
) -> (App, std::path::PathBuf) {
    let _ = std::fs::remove_dir_all(root);
    let slot = crate::saves::create(root, "Rowboat era", 1).unwrap();
    let path = root.join(&slot.id).join(crate::saves::VEHICLES);
    std::fs::write(&path, ROWBOAT_ERA).unwrap();
    let mut save = crate::saves::WorldSave::open(root.to_path_buf(), slot);
    assert!(save.record_pack(slots));
    save.drain();
    let mut app = app(save);
    app.insert_resource(crate::hotbar::Hotbar(slots.clone()));
    (app, path)
}

fn planks_and_rope(app: &App) -> (u32, u32) {
    use pbd_core::goods::Good;
    use pbd_core::inventory::Item;
    let hotbar = app.world().resource::<crate::hotbar::Hotbar>();
    (
        hotbar.count(Item::Good(Good::PLANKS)),
        hotbar.count(Item::Good(Good::ROPE)),
    )
}

/// `retire-rowboat-and-cog` 1.1 and 1.2: a world saved with harbour
/// rowboats, a cog and a rowboat the player made opens with its Loon and its
/// canoe only; every rowboat and cog record is held and written back byte
/// for byte, their berths count as made, and `next_id` stays above them.
/// The made rowboat is refunded its 16 planks and 2 rope once, with a
/// `refund` record, and not again when the world is opened again.
#[test]
fn a_world_saved_with_rowboats_and_a_cog_keeps_every_record() {
    let old: VehicleFile = ron::from_str(ROWBOAT_ERA).expect("layout 1 still reads");
    let held: Vec<_> = old
        .vehicles
        .iter()
        .filter(|r| r.kind == "rowboat" || r.kind == "cog")
        .cloned()
        .collect();
    assert_eq!(
        held.len(),
        4,
        "two harbour rowboats, the cog, the made rowboat"
    );
    let made = held.iter().find(|r| r.made).expect("a made rowboat").id;
    let root = std::env::temp_dir().join(format!("pbd-retire-rowboat-{}", std::process::id()));
    let (mut app, path) = rowboat_era(&root, &pbd_core::inventory::Slots::new());
    for _ in 0..3 {
        app.update();
    }
    let mut kinds: Vec<&str> = crafts(&mut app).iter().map(|(_, c)| c.kind.key()).collect();
    let fleet = app.world().resource::<Fleet>();
    kinds.extend(fleet.stowed.iter().map(|r| r.kind.as_str()));
    kinds.sort();
    assert_eq!(
        kinds,
        ["loon", "loon"],
        "the Loon and the canoe, nothing else"
    );
    assert_eq!(
        fleet.retired, held,
        "every rowboat and cog held aside as read"
    );
    assert!(held.iter().all(|r| fleet.next_id > r.id));
    let berths: Vec<_> = held.iter().filter_map(|r| r.berth).collect();
    assert_eq!(berths.len(), 3, "the rowboats' and the cog's berths");
    let taken = harbour::made(app.world_mut());
    assert!(
        berths.iter().all(|b| taken.contains(b)),
        "never filled again"
    );
    // Refunded, into the slots and the save, with its mark.
    assert_eq!(planks_and_rope(&app), (16, 2), "the rowboat's recipe back");
    {
        let save = app.world().resource::<crate::saves::WorldSave>();
        assert!(save.records.get(refund::REFUND_KIND, made).is_some());
        let carried = save.carried.as_ref().unwrap();
        assert_eq!(carried, &app.world().resource::<crate::hotbar::Hotbar>().0);
    }
    // Written again, every held record is in the file as it was read.
    app.world_mut().resource_mut::<Fleet>().dirty = true;
    app.update();
    app.world().resource::<crate::saves::WorldSave>().drain();
    let written = std::fs::read_to_string(&path).unwrap();
    let file: VehicleFile = ron::from_str(&written).unwrap();
    assert!(file.vehicles.ends_with(&held), "{written}");
    assert_eq!(file.vehicles.len(), old.vehicles.len(), "nothing dropped");
    assert_eq!(
        record_text(&written, "cog"),
        record_text(ROWBOAT_ERA, "cog"),
        "the cog's record is byte-identical"
    );
    let made_text = |file: &str| {
        let at = file.find(&format!("id: {made},")).unwrap();
        let start = file[..at].rfind("(\n").unwrap();
        let end = at + file[at..].find("\n        ),").unwrap();
        file[start..end].to_string()
    };
    assert_eq!(
        made_text(&written),
        made_text(ROWBOAT_ERA),
        "the made rowboat's too"
    );
    drop(app);
    // Opened again: the refund is not given twice.
    let listed = crate::saves::list(&root);
    let save = crate::saves::WorldSave::open(root.clone(), listed[0].clone());
    let carried = save.carried.clone().expect("the slots were saved");
    let mut again = app_with(save, carried);
    for _ in 0..3 {
        again.update();
    }
    assert_eq!(planks_and_rope(&again), (16, 2), "refunded once only");
    let _ = std::fs::remove_dir_all(&root);
}

fn app_with(save: crate::saves::WorldSave, slots: pbd_core::inventory::Slots) -> App {
    let mut app = app(save);
    app.insert_resource(crate::hotbar::Hotbar(slots));
    app
}

/// `retire-rowboat-and-cog` decision 2: a refund that does not all fit
/// waits, nothing given and nothing marked, and is given whole once there is
/// room; a hold's stacks come with it.
#[test]
fn a_refund_waits_for_room_in_the_pack() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots, Stack};
    let old: VehicleFile = ron::from_str(ROWBOAT_ERA).unwrap();
    let record = old.vehicles.iter().find(|r| r.made).unwrap().clone();
    let mut save = crate::saves::WorldSave::memory_only();
    // Something in its hold, as only a raft's could have.
    let mut held = pbd_core::vehicle::hold::Hold::default();
    held.slots[0] = Some(Stack::new(Item::Good(Good::LOG), 12));
    save.store(
        &pbd_core::records::Author::Player(0),
        vec![hold::record(record.id, &held)],
    );
    let mut full = Slots::new();
    while full.give(Item::Good(Good::COAL), 99) == 0 {}
    let before = full.clone();
    assert!(
        !refund::refund_one(&mut save, &mut full, &record),
        "no room"
    );
    assert_eq!(full, before, "nothing given");
    assert!(save.records.get(refund::REFUND_KIND, record.id).is_none());
    // Room for it: three slots emptied.
    for i in 0..3 {
        full.take(i, 99);
    }
    assert!(refund::refund_one(&mut save, &mut full, &record));
    assert_eq!(full.count(Item::Good(Good::PLANKS)), 16);
    assert_eq!(full.count(Item::Good(Good::ROPE)), 2);
    assert_eq!(full.count(Item::Good(Good::LOG)), 12, "its hold came too");
    assert!(save.records.get(refund::REFUND_KIND, record.id).is_some());
    assert_eq!(
        save.carried.as_ref(),
        Some(&full),
        "in one line with the slots"
    );
    assert!(
        !refund::refund_one(&mut save, &mut full, &record),
        "and never twice"
    );
    // A harbour rowboat was not made and holds nothing: owed nothing.
    let harbour = old
        .vehicles
        .iter()
        .find(|r| r.kind == "rowboat" && !r.made)
        .unwrap();
    assert!(refund::owed(&save, harbour).is_empty());
}

/// `retire-rowboat-and-cog` decision 3: a rowboat good in the pack is
/// swapped for its recipe's planks and rope, and written; a full pack keeps
/// the good until there is room.
#[test]
fn a_rowboat_good_becomes_its_materials() {
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Slots};
    let mut slots = Slots::new();
    slots.give(Item::Good(Good::ROWBOAT), 1);
    assert!(refund::swap_goods(&mut slots));
    assert_eq!(slots.count(Item::Good(Good::ROWBOAT)), 0);
    assert_eq!(slots.count(Item::Good(Good::PLANKS)), 16);
    assert_eq!(slots.count(Item::Good(Good::ROPE)), 2);
    assert!(!refund::swap_goods(&mut slots), "once");
    // In the game: opened with one in the pack, and written.
    let mut pack = Slots::new();
    pack.give(Item::Good(Good::ROWBOAT), 1);
    let mut app = app(crate::saves::WorldSave::memory_only());
    {
        let mut save = app.world_mut().resource_mut::<crate::saves::WorldSave>();
        assert!(save.record_pack(&pack));
    }
    app.insert_resource(crate::hotbar::Hotbar(pack));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(planks_and_rope(&app), (16, 2));
    let save = app.world().resource::<crate::saves::WorldSave>();
    assert_eq!(
        save.carried
            .as_ref()
            .map(|c| c.count(Item::Good(Good::ROWBOAT))),
        Some(0),
        "the swap is saved"
    );
    // A full pack keeps it.
    let mut full = Slots::new();
    full.give(Item::Good(Good::ROWBOAT), 1);
    while full.give(Item::Good(Good::COAL), 99) == 0 {}
    let before = full.clone();
    assert!(!refund::swap_goods(&mut full));
    assert_eq!(full, before);
}
