//! The floating render origin (`movement-smoothness` design 6).
//!
//! Three spaces:
//! - **physics space**: Avian's `Position`, and the physics bodies' own
//!   `Transform`s. Planet-centred in play.
//! - **planet-local**: relative to the planet's centre.
//! - **render space**: every other `Transform`/`GlobalTransform` and the
//!   views: `physics - origin`, with the origin a `DVec3` in physics space
//!   that follows the camera on a 16 m grid.
//!
//! So nothing drawn near the eye sits 4 800 m from 0, where an f32 holds
//! only 0.49 mm. `PlanetRenderFrame` carries the origin and the planet's
//! centre in render space (`center`); [`rebase`] moves the origin when the
//! camera leaves its cell and moves every root render transform by the same
//! amount in the same system, so nothing is drawn half in the old frame.

use avian3d::prelude::RigidBody;
use bevy::{math::DVec3, prelude::*};

use crate::planet::PlanetRenderFrame;

/// The grid the origin snaps to: the sim origin's cell (`world-frames`; the
/// owner: "16 m is kinda small, 128 m probably good"). A whole number of
/// metres that is a power of two, so a rebase moves every coordinate by an
/// exact f32 amount and repeated rebases never drift. Each rebase moves
/// every root and so recomputes the whole hierarchy (`river-hitches`
/// design 4): at 16 m that was a hitch every two seconds on foot.
pub const ORIGIN_CELL_M: f64 = pbd_core::world_frame::ORIGIN_CELL_M;

/// How far the eye may be from its origin's cell, on an axis, before the
/// origin moves: three quarters of a cell, as the sim origin, so walking
/// along a boundary never rebases back and forth. Within it an f32 holds
/// 7.6 µm.
pub const REBASE_AT_M: f64 = 0.75 * ORIGIN_CELL_M;

/// `--render-offset`: the render origin held this far (render space) from
/// the camera's cell, so the planet's centre and the eye both sit away from
/// 0. A test of everything that reads render space; zero in play.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct RenderOffset(pub DVec3);

/// The origin for an eye at `eye` (physics space): its 16 m cell, less the
/// offset.
pub fn origin_for(eye: DVec3, offset: DVec3) -> DVec3 {
    (eye / ORIGIN_CELL_M).round() * ORIGIN_CELL_M - offset
}

/// Move the render origin to the active camera's cell, at most once a frame.
/// Runs in `PostUpdate` before `update_planet_frame` and propagation.
#[allow(clippy::type_complexity)]
pub fn rebase(
    offset: Option<Res<RenderOffset>>,
    mut frame: ResMut<PlanetRenderFrame>,
    mut roots: Query<
        &mut Transform,
        (
            Without<ChildOf>,
            Without<RigidBody>,
            Without<Node>,
            Without<Camera3d>,
        ),
    >,
    mut cameras: Query<
        (&Camera, &mut Transform),
        (With<Camera3d>, Without<ChildOf>, Without<RigidBody>),
    >,
    mut globals: Query<&mut GlobalTransform, (Without<RigidBody>, Without<Node>)>,
    mut held: Local<Option<bool>>,
) {
    // MEASUREMENT (`movement-smoothness` design 6.7): `PBD_HOLD_ORIGIN=1`
    // never moves the origin, so it stays where it starts (the planet's
    // centre in play). Every conversion still runs; a still taken so must
    // match the build before the floating origin pixel for pixel, which
    // shows that what differs with the origin moving is the origin alone.
    if *held.get_or_insert_with(|| std::env::var_os("PBD_HOLD_ORIGIN").is_some_and(|v| v == "1")) {
        return;
    }
    let Some(eye) = cameras
        .iter()
        .find(|(c, _)| c.is_active)
        .map(|(_, t)| t.translation)
    else {
        return;
    };
    let offset = offset.map_or(DVec3::ZERO, |o| o.0);
    // The camera's physics-space point: render space plus the origin, in
    // f64, from an f32 that is already small.
    let physics_eye = eye.as_dvec3() + frame.origin;
    if !physics_eye.is_finite() {
        return;
    }
    // Still within reach of the origin's cell: no rebase.
    let off = physics_eye - (frame.origin + offset);
    if off.abs().max_element() <= REBASE_AT_M {
        return;
    }
    let origin = origin_for(physics_eye, offset);
    let delta = origin - frame.origin;
    if delta == DVec3::ZERO {
        return;
    }
    frame.origin = origin;
    frame.center -= delta;
    let shift = delta.as_vec3();
    for mut transform in &mut roots {
        transform.translation -= shift;
    }
    for (_, mut transform) in &mut cameras {
        transform.translation -= shift;
    }
    // Last frame's propagated transforms too, so a system that reads one
    // before this frame's propagation sees it in the same frame as `center`.
    for mut global in &mut globals {
        let mut affine = global.affine();
        affine.translation -= bevy::math::Vec3A::from(shift);
        *global = GlobalTransform::from(affine);
    }
}

/// A drawn (render-space) point as planet-local, in f64 before the cast.
/// With no frame (a test app without the planet) render space is
/// planet-local.
pub fn local_of(frame: Option<&PlanetRenderFrame>, p: Vec3) -> Vec3 {
    frame.map_or(p, |f| (p.as_dvec3() - f.center).as_vec3())
}

/// A planet-local point as render space, in f64 before the cast.
pub fn drawn_of(frame: Option<&PlanetRenderFrame>, p: Vec3) -> Vec3 {
    frame.map_or(p, |f| (f.center + p.as_dvec3()).as_vec3())
}

/// The physics-space position `p` as render space: subtracted in f64, cast
/// after.
pub fn render_from_physics(frame: &PlanetRenderFrame, p: Vec3) -> Vec3 {
    (p.as_dvec3() - frame.origin).as_vec3()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::update_planet_frame;

    const R: f64 = 4800.0;

    fn app(offset: DVec3) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .insert_resource(crate::CelestialScene::planet_at_origin(R, 1.0))
            .init_resource::<crate::PhysicsFrame>()
            .insert_resource(RenderOffset(offset))
            .insert_resource(PlanetRenderFrame {
                center: offset,
                origin: -offset,
            })
            .add_systems(
                PostUpdate,
                (rebase, update_planet_frame)
                    .chain()
                    .before(bevy::transform::TransformSystems::Propagate),
            );
        app
    }

    /// The eye at planet-local `eye`, written as a camera writer does.
    fn place_camera(app: &mut App, camera: Entity, eye: DVec3) {
        let frame = *app.world().resource::<PlanetRenderFrame>();
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation = (frame.center + eye).as_vec3();
    }

    fn planet_local(app: &App, e: Entity) -> DVec3 {
        let frame = app.world().resource::<PlanetRenderFrame>();
        app.world()
            .get::<GlobalTransform>(e)
            .unwrap()
            .translation()
            .as_dvec3()
            - frame.center
    }

    /// A camera far from the planet's centre derives the same planet-local
    /// eye, up and altitude whatever the origin: at 0, at 4 800 m (the
    /// surface), and at a random offset.
    #[test]
    fn a_far_camera_derives_the_same_values_with_any_origin() {
        let eye = DVec3::new(1234.567, 4321.0625, -2001.75).normalize() * (R + 31.4);
        let mut seen = Vec::new();
        for offset in [
            DVec3::ZERO,
            DVec3::new(0.0, 4800.0, 0.0),
            DVec3::new(8192.0, -4096.0, 2048.0),
            DVec3::new(-3517.25, 977.5, 6131.0),
        ] {
            let mut app = app(offset);
            let camera = app
                .world_mut()
                .spawn((Camera3d::default(), Transform::default()))
                .id();
            let prop = app
                .world_mut()
                .spawn(Transform::from_translation(Vec3::ZERO))
                .id();
            app.update();
            place_camera(&mut app, camera, eye);
            // A prop five metres ahead, written from planet-local.
            let frame = *app.world().resource::<PlanetRenderFrame>();
            app.world_mut()
                .get_mut::<Transform>(prop)
                .unwrap()
                .translation = (frame.center + eye + DVec3::X * 5.0).as_vec3();
            for _ in 0..3 {
                app.update();
                place_camera(&mut app, camera, eye);
            }
            app.update();
            let frame = *app.world().resource::<PlanetRenderFrame>();
            // The origin sits in the camera's cell, less the offset.
            assert_eq!(frame.origin, origin_for(eye, offset));
            let local = planet_local(&app, camera);
            let up = local.normalize();
            let altitude = local.length() - R;
            let drawn = app
                .world()
                .get::<GlobalTransform>(camera)
                .unwrap()
                .translation();
            // Render space is bounded round the eye when no offset is asked.
            if offset == DVec3::ZERO {
                assert!(drawn.length() < ORIGIN_CELL_M as f32, "{drawn}");
            }
            let prop_local = planet_local(&app, prop);
            assert!((prop_local - (eye + DVec3::X * 5.0)).length() < 2e-3);
            seen.push((local, up, altitude));
        }
        for (local, up, altitude) in &seen {
            assert!((*local - eye).length() < 2e-3, "{local} {eye}");
            assert!((*up - eye.normalize()).length() < 1e-6);
            assert!((altitude - 31.4).abs() < 2e-3);
        }
    }

    /// Walking across many cells: every rebase moves the camera and the
    /// world together, so a prop stays where it is in the world and the
    /// camera moves only by its own walk. No jump across a rebase.
    #[test]
    fn nothing_jumps_across_a_rebase() {
        let mut app = app(DVec3::ZERO);
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        let start = DVec3::new(0.3, 1.0, 0.2).normalize() * (R + 1.6);
        let east = DVec3::Y.cross(start).normalize();
        let prop_at = start + east * 40.0;
        app.update();
        place_camera(&mut app, camera, start);
        let frame = *app.world().resource::<PlanetRenderFrame>();
        let prop = app
            .world_mut()
            .spawn(Transform::from_translation(
                (frame.center + prop_at).as_vec3(),
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, -0.5), ChildOf(camera)))
            .id();
        let mut rebases = 0;
        let mut last_origin = frame.origin;
        let mut last_eye_view = None;
        for i in 0..4000 {
            let eye = start + east * (i as f64 * 0.25);
            place_camera(&mut app, camera, eye);
            app.update();
            let frame = *app.world().resource::<PlanetRenderFrame>();
            if frame.origin != last_origin {
                rebases += 1;
                last_origin = frame.origin;
            }
            // The prop stays put in the world.
            assert!((planet_local(&app, prop) - prop_at).length() < 1e-3);
            // The camera is where it was placed.
            assert!((planet_local(&app, camera) - eye).length() < 1e-4);
            // The held child stays half a metre ahead, to a micrometre.
            let held = app
                .world()
                .get::<GlobalTransform>(child)
                .unwrap()
                .translation()
                - app
                    .world()
                    .get::<GlobalTransform>(camera)
                    .unwrap()
                    .translation();
            assert!((held - Vec3::new(0.0, 0.0, -0.5)).length() < 1e-5, "{held}");
            // The prop seen from the eye moves smoothly: by the walk's step.
            let view = (planet_local(&app, prop) - planet_local(&app, camera)).as_vec3();
            if let Some(last) = last_eye_view {
                let step: Vec3 = view - last;
                assert!((step.length() - 0.25).abs() < 1e-3, "{step}");
            }
            last_eye_view = Some(view);
        }
        assert!(rebases >= 5, "{rebases}");
    }
}
