//! The local simulation origin in the game (`world-frames`): one origin for
//! Avian, the simulation and the renderer.
//!
//! Physics space (Avian's `Position`, every simulated local `Vec3`) is the
//! local space of [`SimFrame`]: a [`SimOrigin`] in the home planet's frame.
//! The only way between a local `Vec3` and a planet-frame [`WorldPos`] is
//! that origin's methods (`pbd_core::world_frame`).

use bevy::{ecs::system::SystemParam, math::DVec3, prelude::*};
use pbd_core::world_frame::{FrameId, FrameKind, FrameMotion, FrameTree, SYSTEM, SimOrigin};

/// The home planet's frame: its centre, axes not turning (the planet does
/// not turn today; when it does, the ground gets a `BodyFixed` child).
pub const HOME: FrameId = FrameId(1);

/// The frames of the world: the system and the home planet today.
#[derive(Resource, Clone, Debug)]
pub struct WorldFrames(pub FrameTree);

impl Default for WorldFrames {
    fn default() -> Self {
        let mut tree = FrameTree::new();
        let home = tree.add(
            SYSTEM,
            FrameKind::BodyInertial,
            FrameMotion::Rails(pbd_core::orbit::CircularOrbit::stationary()),
        );
        assert_eq!(home, HOME);
        Self(tree)
    }
}

/// Where local (physics) space sits: its origin in the home planet's frame.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct SimFrame(pub SimOrigin);

impl Default for SimFrame {
    fn default() -> Self {
        Self(SimOrigin::new(HOME, DVec3::ZERO))
    }
}

/// Read access to the local origin. With no [`SimFrame`] (a test app
/// without the planet) local space is the home frame itself.
#[derive(SystemParam)]
pub struct SimSpace<'w> {
    frame: Option<Res<'w, SimFrame>>,
}

impl SimSpace<'_> {
    pub fn origin(&self) -> SimOrigin {
        self.frame.as_deref().copied().unwrap_or_default().0
    }
}

/// The origin from a `World`, for exclusive systems.
pub fn origin_of(world: &World) -> SimOrigin {
    world
        .get_resource::<SimFrame>()
        .copied()
        .unwrap_or_default()
        .0
}

/// Something kept in local space outside Avian's `Position`, moved by a
/// rebase: each module that keeps local positions (the walker's ground
/// state, the fishery, the flight's protected position) registers one.
/// A hook takes the shift (`local -= shift`) and touches only its own data,
/// so a rebase is O(things simulated), never O(things drawn).
pub type RebaseHook = fn(&mut World, Vec3);

#[derive(Resource, Default)]
pub struct RebaseHooks(pub Vec<RebaseHook>);

/// Register a [`RebaseHook`] (from a plugin's `build`).
pub fn add_rebase_hook(app: &mut App, hook: RebaseHook) {
    app.init_resource::<RebaseHooks>();
    app.world_mut().resource_mut::<RebaseHooks>().0.push(hook);
}

/// The rebases done, for the trace and the tests.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct RebaseCount(pub u64);

pub struct SimOriginPlugin;

impl Plugin for SimOriginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldFrames>()
            .init_resource::<SimFrame>()
            .init_resource::<RebaseHooks>()
            .init_resource::<RebaseCount>()
            // Between frames' fixed loops: a tick boundary, once a frame.
            .add_systems(PreUpdate, follow_player);
    }
}

/// Whether the origin is held where it starts (`PBD_HOLD_SIM_ORIGIN=1`, or
/// `PBD_HOLD_ORIGIN=1`, which holds the render origin too): the A/B with the
/// planet-centred simulation before `world-frames`.
pub fn held() -> bool {
    static HELD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *HELD.get_or_init(|| {
        ["PBD_HOLD_SIM_ORIGIN", "PBD_HOLD_ORIGIN"]
            .iter()
            .any(|k| std::env::var_os(k).is_some_and(|v| v == "1"))
    })
}

/// Rebase so the player stays near the local origin: the walker while
/// walking, else the piloted ship, else the active camera.
pub fn follow_player(world: &mut World) {
    if held() || world.get_resource::<SimFrame>().is_none() {
        return;
    }
    let Some((anchor, speed)) = player_anchor(world) else {
        return;
    };
    let dt = world
        .get_resource::<Time<Fixed>>()
        .map_or(1.0 / crate::FIXED_HZ, |t| t.timestep().as_secs_f64());
    let cell = pbd_core::world_frame::cell_for_speed(speed as f64, dt);
    let origin = origin_of(world);
    if let Some(rebase) = origin.rebase_for(anchor, cell) {
        apply(world, rebase);
    }
}

/// Rebase now onto `anchor` (local), whatever the slack: a teleport
/// (respawn, restore, boarding) that put the player far from the origin.
pub fn recenter_on(world: &mut World, anchor: Vec3) {
    let point = origin_of(world).origin + anchor.as_dvec3();
    recenter_on_point(world, point);
}

/// Rebase now onto `point` in the home frame, before a teleport writes
/// anything there, so the position written is small and nothing is rounded
/// at the planet's scale.
pub fn recenter_on_point(world: &mut World, world_point: DVec3) {
    if held() || world.get_resource::<SimFrame>().is_none() {
        return;
    }
    let origin = origin_of(world);
    if !world_point.is_finite() {
        return;
    }
    let snapped = (world_point / pbd_core::world_frame::ORIGIN_CELL_M).round()
        * pbd_core::world_frame::ORIGIN_CELL_M;
    let delta = snapped - origin.origin;
    if delta != DVec3::ZERO {
        apply(
            world,
            pbd_core::world_frame::Rebase {
                shift: delta.as_vec3(),
                delta,
            },
        );
    }
}

fn player_anchor(world: &mut World) -> Option<(Vec3, f32)> {
    use avian3d::prelude::{LinearVelocity, Position};
    let walking = world
        .get_resource::<crate::walking::WalkingState>()
        .is_some_and(|s| s.active);
    if walking
        && let Some((p, v)) = world
            .query_filtered::<(&Position, &LinearVelocity), With<crate::walking::Walker>>()
            .iter(world)
            .next()
    {
        return Some((p.0, v.0.length()));
    }
    if let Some((p, v)) = world
        .query_filtered::<(&Position, &LinearVelocity), With<crate::flight_view::PilotShip>>()
        .iter(world)
        .next()
        && !walking
        && world
            .get_resource::<crate::vehicles::Aboard>()
            .is_none_or(|a| a.0.is_none())
    {
        return Some((p.0, v.0.length()));
    }
    // A vehicle's camera, or a capture's: where the eye is, in physics space.
    let render_origin = world
        .get_resource::<crate::planet::PlanetRenderFrame>()
        .map_or(DVec3::ZERO, |f| f.origin);
    world
        .query::<(&Camera, &Transform)>()
        .iter(world)
        .find(|(c, _)| c.is_active)
        .map(|(_, t)| ((t.translation.as_dvec3() + render_origin).as_vec3(), 0.0))
}

/// Move the local origin by `rebase.delta`: every body's `Position` (and
/// its physics-space `Transform`), every registered hook's data, the
/// physics frame, and the render origin, so nothing drawn moves and no
/// velocity changes.
pub fn apply(world: &mut World, rebase: pbd_core::world_frame::Rebase) {
    use avian3d::prelude::{Position, RigidBody};
    let shift = rebase.shift;
    for (mut position, transform, global) in world
        .query_filtered::<(
            &mut Position,
            Option<&mut Transform>,
            Option<&mut GlobalTransform>,
        ), With<RigidBody>>()
        .iter_mut(world)
    {
        position.0 -= shift;
        if let Some(mut t) = transform {
            t.translation -= shift;
        }
        // Avian reads a body's `GlobalTransform` back into its `Position`
        // where they differ and the `Position` did not change: kept in step.
        if let Some(mut g) = global {
            let mut affine = g.affine();
            affine.translation -= bevy::math::Vec3A::from(shift);
            *g = GlobalTransform::from(affine);
        }
    }
    let hooks: Vec<RebaseHook> = world
        .get_resource::<RebaseHooks>()
        .map(|h| h.0.clone())
        .unwrap_or_default();
    for hook in hooks {
        hook(world, shift);
    }
    if let Some(mut frame) = world.get_resource_mut::<crate::PhysicsFrame>() {
        frame.0.origin += rebase.delta;
    }
    if let Some(mut sim) = world.get_resource_mut::<SimFrame>() {
        sim.0 = sim.0.rebased(rebase);
    }
    // Render space is physics space less the render origin: moving the
    // render origin with physics space leaves every drawn transform where
    // it is (`world-frames` design 4), so a rebase walks no render roots.
    if let Some(mut frame) = world.get_resource_mut::<crate::planet::PlanetRenderFrame>() {
        frame.origin -= rebase.delta;
    }
    if let Some(mut count) = world.get_resource_mut::<RebaseCount>() {
        count.0 += 1;
    }
}

/// The local origin as the render frame carries it: the planet's centre in
/// physics space is `-origin` (`PlanetRenderFrame::physics_center`). For a
/// system whose parameters are full; it agrees with [`SimFrame`] at every
/// moment, a rebase moving both.
pub fn from_render_frame(frame: &crate::planet::PlanetRenderFrame) -> SimOrigin {
    SimOrigin::new(HOME, -frame.physics_center())
}

/// A drawn (render-space) point as local (physics) space: render space is
/// physics space less the render origin.
pub fn local_from_render(frame: &crate::planet::PlanetRenderFrame, p: Vec3) -> Vec3 {
    (p.as_dvec3() + frame.origin).as_vec3()
}
