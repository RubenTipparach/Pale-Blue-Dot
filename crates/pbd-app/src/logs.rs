//! Logs in the water (`log-drives`, design "As built"): thrown in with Q,
//! carried by the same current a craft feels (`vehicles::fresh_water`),
//! resting where the water stops, taken back with F, and loaded onto a raft
//! (Y) and off it (U).
//!
//! The rule is `pbd_core::log_drive`; a log is a `log` record in the save,
//! written when it is made, when it stops or starts, every few seconds while
//! it moves, and when it is taken.

use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetRenderFrame};
use crate::saves::WorldSave;
use crate::vehicles::{Aboard, Vehicle, fresh_water};
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::log_drive::{self, LOG_SCHEMA, Log, LogState, Stepped, Water};
use pbd_core::records::Author;
use pbd_core::vehicle::Kind;

/// How far from the walker's chest F takes a log, metres.
pub const TAKE_M: f64 = 2.6;
/// How far from a raft Y loads a log, metres.
pub const LOAD_M: f64 = 4.5;
/// Logs further than this from the camera are not stepped, metres: there is
/// no column tier to read their water from.
pub const STEP_M: f64 = 400.0;

/// How far a log must float before it comes to rest for the town tree to
/// count it, metres.
pub const FLOAT_COUNTS_M: f64 = 20.0;

/// Every log in the world that has not been taken.
#[derive(Resource, Default)]
pub struct Logs {
    pub live: Vec<Log>,
    serial: Option<u64>,
}

/// A drawn log.
#[derive(Component)]
pub struct LogView(pub u64);

/// What the water is at a body-frame point: a river or lake's surface and
/// current, the sea's still surface, or nothing over dry ground.
pub fn water_at(contact: &PlanetContact, point: DVec3) -> Option<Water> {
    let bed = crate::vehicles::ground_under(Some(contact), point);
    if let Some(fresh) = fresh_water(contact, point) {
        return Some(Water {
            surface: fresh.surface,
            current: fresh.current,
            depth: (fresh.surface - bed) as f32,
        });
    }
    let up = point.normalize_or(DVec3::Y);
    let top = [point, point - up * 0.5, point - up * 1.0]
        .into_iter()
        .find_map(|q| contact.drawn_water(q.as_vec3()).flatten())?;
    let surface = f64::from(PLANET_RADIUS + top);
    Some(Water {
        surface,
        current: DVec3::ZERO,
        depth: (surface - bed) as f32,
    })
}

impl Logs {
    /// A new floating log at `position` (body frame), its axis across
    /// `heading`, with the world's next log id. Not stored here: the caller
    /// writes its record, with whatever else moves in the same line.
    pub fn make(&self, save: &WorldSave, position: DVec3, heading: DVec3) -> Log {
        let id = log_drive::next_id(&save.records)
            .max(self.live.iter().map(|l| l.id + 1).max().unwrap_or(1));
        Log::new(id, log_drive::made(position, heading))
    }
}

/// Read the logs from the save when a world is opened; step every log near
/// the camera in its water, and write what the save must hear.
pub fn step(
    time: Res<Time>,
    mut logs: ResMut<Logs>,
    mut save: ResMut<WorldSave>,
    contact: Option<Res<PlanetContact>>,
    frame: Option<Res<PlanetRenderFrame>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut did: MessageWriter<crate::missions::Did>,
    mut set_off: Local<std::collections::HashMap<u64, DVec3>>,
) {
    if logs.serial != Some(save.serial()) {
        logs.live = log_drive::from_records(&save.records);
        logs.serial = Some(save.serial());
        info!("LOGS {} in the world", logs.live.len());
    }
    let Some(contact) = contact else {
        return;
    };
    let dt = f64::from(time.delta_secs());
    let camera = cameras
        .iter()
        .find(|(_, c)| c.is_active)
        .zip(frame.as_deref())
        .map(|((at, _), f)| at.translation().as_dvec3() - f.center);
    let mut writes = Vec::new();
    for log in &mut logs.live {
        if camera.is_some_and(|c| c.distance(log.position()) > STEP_M) {
            continue;
        }
        let water = water_at(&contact, log.position());
        let was = log.body.state;
        match log_drive::step(log, water, dt) {
            Stepped::Quiet => {}
            Stepped::Changed => {
                // Where a float began, and a float's end far enough from it:
                // a log driven down a river (`town-missions-tree` design 4).
                match (was, log.body.state) {
                    (_, LogState::Floating) => {
                        set_off.insert(log.id, log.position());
                    }
                    (LogState::Floating, LogState::Resting) => {
                        let far = set_off
                            .remove(&log.id)
                            .is_some_and(|from| from.distance(log.position()) >= FLOAT_COUNTS_M);
                        if far {
                            did.write(crate::missions::Did::new(
                                pbd_core::missions::Kind::Float,
                                "log",
                                1,
                            ));
                        }
                    }
                    _ => {
                        set_off.remove(&log.id);
                    }
                }
                info!(
                    "LOG {} {:?} -> {:?} at {:.1} m",
                    log.id,
                    was,
                    log.body.state,
                    log.position().length() - f64::from(PLANET_RADIUS)
                );
                writes.push(log_drive::record(log));
            }
            Stepped::Due => writes.push(log_drive::record(log)),
        }
    }
    if !writes.is_empty() {
        // The water moved it: the water's line (`world-persistence`).
        let _ = save.store(&crate::saves::water_author(), writes);
    }
}

/// Q with logs selected, over water: one log thrown in, written with the
/// slots after it in one line. Anything else Q drops as it always did.
pub fn throw_in(
    save: &mut WorldSave,
    logs: &mut Logs,
    hotbar: &mut crate::hotbar::Hotbar,
    contact: &PlanetContact,
    slot: usize,
    at: DVec3,
    heading: DVec3,
) -> bool {
    if hotbar.0.get(slot).map(|s| s.item) != Some(Item::Good(Good::LOG)) {
        return false;
    }
    let Some(water) = water_at(contact, at) else {
        return false;
    };
    if water.depth < log_drive::FLOAT_DEPTH_M {
        return false;
    }
    let up = at.normalize_or(DVec3::Y);
    let on = up * (water.surface - log_drive::RIDE_M);
    let log = logs.make(save, on, heading);
    let mut slots = hotbar.0.clone();
    if slots.take(slot, 1) != 1 {
        return false;
    }
    if !save.store_with_hands(log_drive::record(&log), LOG_SCHEMA, &slots) {
        error!("log BLOCKED: the save did not accept it");
        return false;
    }
    hotbar.0 = slots;
    info!("LOG {} thrown in", log.id);
    logs.live.push(log);
    true
}

/// F on foot, near a log: offer to take it.
pub fn offer(
    logs: Res<Logs>,
    walkers: Query<&Transform, With<crate::walking::Walker>>,
    space: crate::SimSpace,
    mut interact: ResMut<crate::interact::Interact>,
) {
    let Some(walker) = walkers.iter().next() else {
        return;
    };
    let chest = crate::drops::chest(&space.origin(), walker.translation);
    if logs
        .live
        .iter()
        .any(|l| l.position().distance(chest) < TAKE_M)
    {
        interact.offer(crate::interact::Offer::new(
            crate::interact::Act::TakeLog,
            pbd_core::locale::tr("item.log.name"),
        ));
    }
}

/// Take the nearest log in reach into the slots, in one line with them.
#[allow(clippy::too_many_arguments)]
pub fn take(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    interact: Option<Res<crate::interact::Interact>>,
    aboard: Option<Res<Aboard>>,
    walkers: Query<&Transform, With<crate::walking::Walker>>,
    space: crate::SimSpace,
    mut logs: ResMut<Logs>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<crate::hotbar::Hotbar>,
    mut gained: MessageWriter<crate::hotbar::Gained>,
) {
    if !keys.just_pressed(KeyCode::KeyF)
        || menu.is_some_and(|m| m.0)
        || aboard.is_some_and(|a| a.0.is_some())
        || !interact
            .as_deref()
            .and_then(|i| i.now.as_ref())
            .is_some_and(|o| o.act == crate::interact::Act::TakeLog)
    {
        return;
    }
    let Some(walker) = walkers.iter().next() else {
        return;
    };
    let chest = crate::drops::chest(&space.origin(), walker.translation);
    let Some(k) = nearest(&logs, chest, TAKE_M) else {
        return;
    };
    let mut slots = hotbar.0.clone();
    if slots.give(Item::Good(Good::LOG), 1) > 0 {
        return;
    }
    let mut log = logs.live[k];
    log.body.state = LogState::Taken;
    if !save.store_with_hands(log_drive::record(&log), LOG_SCHEMA, &slots) {
        error!("log BLOCKED: the save did not take it");
        return;
    }
    hotbar.0 = slots;
    logs.live.remove(k);
    gained.write(crate::hotbar::Gained {
        item: Item::Good(Good::LOG),
        count: 1,
        how: crate::hotbar::How::Found,
    });
    info!("LOG {} taken", log.id);
}

fn nearest(logs: &Logs, at: DVec3, reach: f64) -> Option<usize> {
    logs.live
        .iter()
        .enumerate()
        .map(|(k, l)| (k, l.position().distance(at)))
        .filter(|(_, d)| *d < reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k)
}

/// Aboard a raft: Y loads the nearest log in reach into its hold, U puts one
/// from the hold over the bow. The hold's line goes first: a crash between
/// the two lines leaves a log too many, never one lost (design, Risks).
#[allow(clippy::too_many_arguments)]
pub fn raft(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    aboard: Option<Res<Aboard>>,
    vehicles: Query<&Vehicle>,
    contact: Option<Res<PlanetContact>>,
    mut logs: ResMut<Logs>,
    mut save: ResMut<WorldSave>,
    hotbar: Res<crate::hotbar::Hotbar>,
) {
    let (load, unload) = (
        keys.just_pressed(KeyCode::KeyY),
        keys.just_pressed(KeyCode::KeyU),
    );
    if !(load || unload) || menu.is_some_and(|m| m.0) {
        return;
    }
    let Some(vehicle) = aboard.and_then(|a| a.0).and_then(|e| vehicles.get(e).ok()) else {
        return;
    };
    if vehicle.craft.kind != Kind::Raft {
        return;
    }
    let id = vehicle.craft.id;
    let at = vehicle.craft.body.position;
    let mut hold = crate::vehicles::hold::read(&save, id);
    let log_item = Item::Good(Good::LOG);
    if load {
        let Some(k) = nearest(&logs, at, LOAD_M) else {
            return;
        };
        let Some(slot) = (0..hold.slots.len()).find(|&i| {
            hold.slots[i].is_none_or(|s| s.item == log_item && s.count < log_item.stack_limit())
        }) else {
            return;
        };
        let count = hold.slots[slot].map_or(0, |s| s.count) + 1;
        hold.slots[slot] = Some(pbd_core::inventory::Stack::new(log_item, count));
        if !crate::vehicles::hold::store(&mut save, id, &hold, &hotbar.0) {
            return;
        }
        let mut log = logs.live.remove(k);
        log.body.state = LogState::Taken;
        let _ = save.store(&Author::Player(0), vec![log_drive::record(&log)]);
        info!("LOG {} loaded onto raft {id}", log.id);
    } else {
        let Some(slot) =
            (0..hold.slots.len()).find(|&i| hold.slots[i].is_some_and(|s| s.item == log_item))
        else {
            return;
        };
        let stack = hold.slots[slot].expect("found");
        hold.slots[slot] =
            (stack.count > 1).then(|| pbd_core::inventory::Stack::new(log_item, stack.count - 1));
        let up = at.normalize_or(DVec3::Y);
        // Over the bow (the body's forward is its -z), lying across it.
        let heading = vehicle.craft.body.orientation * DVec3::X;
        let bow = vehicle.craft.body.orientation * DVec3::NEG_Z;
        let bow = (bow - up * bow.dot(up)).normalize_or(up.any_orthonormal_vector());
        let Some(contact) = contact else {
            return;
        };
        let over = at + bow * 3.0;
        let Some(water) = water_at(&contact, over) else {
            return;
        };
        let flat = over - up * over.dot(up);
        let log = logs.make(
            &save,
            up * (water.surface - log_drive::RIDE_M) + flat,
            heading,
        );
        if !crate::vehicles::hold::store(&mut save, id, &hold, &hotbar.0) {
            return;
        }
        let _ = save.store(&Author::Player(0), vec![log_drive::record(&log)]);
        save.note_record_kinds(&[(log_drive::LOG_RECORD, LOG_SCHEMA)]);
        info!(
            "LOG {} unloaded off raft {id}, {:.1} m from it",
            log.id,
            log.position().distance(at)
        );
        logs.live.push(log);
    }
}

/// A triangle soup: positions, normals and texture coordinates.
type Soup = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>);

/// The log's shape: a bark cylinder with cut timber ends, along X.
fn log_meshes() -> (Mesh, Mesh) {
    let (r, half) = (log_drive::RADIUS_M, log_drive::LENGTH_M * 0.5);
    let sides = 10;
    let mut side = (Vec::new(), Vec::new(), Vec::new());
    let mut ends = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..sides {
        let a0 = k as f32 / sides as f32 * std::f32::consts::TAU;
        let a1 = (k + 1) as f32 / sides as f32 * std::f32::consts::TAU;
        let p = |a: f32, x: f32| Vec3::new(x, a.cos() * r, a.sin() * r);
        let n = |a: f32| Vec3::new(0.0, a.cos(), a.sin());
        let u0 = k as f32 / sides as f32 * 2.0;
        let u1 = (k + 1) as f32 / sides as f32 * 2.0;
        for (pa, pb, pc, na, nb, nc, ua, ub, uc) in [
            (
                p(a0, -half),
                p(a1, half),
                p(a1, -half),
                n(a0),
                n(a1),
                n(a1),
                [u0, 0.0],
                [u1, 2.0],
                [u1, 0.0],
            ),
            (
                p(a0, -half),
                p(a0, half),
                p(a1, half),
                n(a0),
                n(a0),
                n(a1),
                [u0, 0.0],
                [u0, 2.0],
                [u1, 2.0],
            ),
        ] {
            // Wound counter-clockwise seen from outside.
            side.0.extend([pa, pc, pb].map(|v| v.to_array()));
            side.1.extend([na, nc, nb].map(|v| v.to_array()));
            side.2.extend([ua, uc, ub]);
        }
        for x in [-half, half] {
            let out = Vec3::X * x.signum();
            let (b, c) = if x > 0.0 {
                (p(a0, x), p(a1, x))
            } else {
                (p(a1, x), p(a0, x))
            };
            ends.0.extend([Vec3::X * x, b, c].map(|v| v.to_array()));
            ends.1.extend([out; 3].map(|v| v.to_array()));
            let uv = |v: Vec3| [0.5 + v.y / (2.0 * r) * 0.4, 0.5 + v.z / (2.0 * r) * 0.4];
            ends.2.extend([uv(Vec3::X * x), uv(b), uv(c)]);
        }
    }
    let mesh = |(p, n, u): Soup| {
        Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, u)
    };
    (mesh(side), mesh(ends))
}

/// The log's look, made once.
#[derive(Resource)]
pub struct LogLook {
    bark: (Handle<Mesh>, Handle<StandardMaterial>),
    ends: (Handle<Mesh>, Handle<StandardMaterial>),
}

fn make_look(world: &mut World) {
    if world.contains_resource::<LogLook>() {
        return;
    }
    let (side, ends) = log_meshes();
    let (bark_image, end_image) = {
        let assets = world.resource::<AssetServer>();
        (
            crate::towns::texture(assets, "bark"),
            crate::towns::texture(assets, "timber"),
        )
    };
    let bark = crate::towns::town_material(world, Some(bark_image), false);
    let end = crate::towns::town_material(world, Some(end_image), false);
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let look = LogLook {
        bark: (meshes.add(side), bark),
        ends: (meshes.add(ends), end),
    };
    world.insert_resource(look);
}

/// A log's place in the render frame: its middle, its long axis along its
/// heading and its up away from the planet's centre, in f64 before the cast.
fn pose(log: &Log, centre: DVec3) -> Transform {
    let p = log.position();
    let up = p.normalize_or(DVec3::Y).as_vec3();
    let along = log.heading().as_vec3();
    let across = along.cross(up).normalize_or(Vec3::Z);
    let rotation = Quat::from_mat3(&Mat3::from_cols(along, up, across));
    Transform::from_translation((centre + p).as_vec3()).with_rotation(rotation)
}

/// Keep one drawn log per live log.
fn draw(
    mut commands: Commands,
    logs: Res<Logs>,
    look: Option<Res<LogLook>>,
    frame: Option<Res<PlanetRenderFrame>>,
    mut views: Query<(Entity, &LogView, &mut Transform)>,
) {
    let (Some(look), Some(frame)) = (look, frame) else {
        return;
    };
    let mut drawn = std::collections::HashSet::new();
    for (entity, view, mut transform) in &mut views {
        match logs.live.iter().find(|l| l.id == view.0) {
            Some(log) => {
                *transform = pose(log, frame.center);
                drawn.insert(view.0);
            }
            None => commands.entity(entity).despawn(),
        }
    }
    for log in &logs.live {
        if drawn.contains(&log.id) {
            continue;
        }
        info!(
            "LOG {} drawn at {:.2} m over the radius",
            log.id,
            log.position().length() - f64::from(PLANET_RADIUS)
        );
        commands
            .spawn((
                Name::new(format!("log {}", log.id)),
                LogView(log.id),
                crate::field_light::LitByField,
                crate::field_light::LitLikeTerrain,
                pose(log, frame.center),
                Visibility::default(),
            ))
            .with_children(|parts| {
                for (mesh, material) in [&look.bark, &look.ends] {
                    parts.spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                    ));
                }
            });
    }
}

/// The logs: stepped, offered, taken, loaded and drawn.
pub struct LogsPlugin;

impl Plugin for LogsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<crate::missions::Did>()
            .init_resource::<Logs>()
            .add_systems(Startup, make_look)
            .add_systems(
                Update,
                (step, take, raft, draw)
                    .chain()
                    .run_if(resource_exists::<WorldSave>),
            )
            .add_systems(
                PostUpdate,
                offer
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<crate::interact::Interact>),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every face of the log's shape faces out, so the back-face cull keeps
    /// what is seen.
    #[test]
    fn a_log_is_closed_and_faces_out() {
        let (side, ends) = log_meshes();
        for mesh in [side, ends] {
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions");
            };
            for t in p.chunks(3) {
                let [a, b, c] = [t[0], t[1], t[2]].map(Vec3::from);
                let n = (b - a).cross(c - a);
                let middle = (a + b + c) / 3.0;
                let out = Vec3::new(
                    if middle.x.abs() > log_drive::LENGTH_M * 0.49 {
                        middle.x
                    } else {
                        0.0
                    },
                    middle.y,
                    middle.z,
                );
                assert!(n.dot(out) > 0.0, "a face turned in at {middle}");
            }
        }
    }

    /// Task 2.2: a log written as it drifts comes back from the disk where
    /// its last line left it, and a taken one does not come back.
    #[test]
    fn a_log_comes_back_where_it_was_and_a_taken_one_does_not() {
        let root = std::env::temp_dir().join(format!("pbd-logs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Logs", 32).unwrap();
        let at = DVec3::new(0.0, f64::from(PLANET_RADIUS) + 3.0, 0.0);
        let (kept, taken) = {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let logs = Logs::default();
            let mut kept = logs.make(&save, at, DVec3::X);
            assert!(save.store_with_hands(
                log_drive::record(&kept),
                LOG_SCHEMA,
                &pbd_core::inventory::Slots::new()
            ));
            // It drifts and is written again, by the water.
            kept.body.position[0] += 7.5;
            save.store(
                &crate::saves::water_author(),
                vec![log_drive::record(&kept)],
            );
            let mut taken = Log::new(kept.id + 1, log_drive::made(at, DVec3::Z));
            save.store(&Author::Player(0), vec![log_drive::record(&taken)]);
            taken.body.state = LogState::Taken;
            save.store(&Author::Player(0), vec![log_drive::record(&taken)]);
            save.drain();
            (kept, taken)
        };
        let reopened = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
        let back = log_drive::from_records(&reopened.records);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].body, kept.body);
        assert!(back.iter().all(|l| l.id != taken.id));
        assert_eq!(log_drive::next_id(&reopened.records), taken.id + 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
