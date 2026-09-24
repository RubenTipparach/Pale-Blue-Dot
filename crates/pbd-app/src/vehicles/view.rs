//! The camera aboard: the seat, looking out of the craft's own frame, or the
//! chase view, standing off behind it. The look is the mouse's raw
//! displacement this frame and nothing else - no easing, no follow spring -
//! which is the flight rule for every player camera; the craft moves the
//! camera because the camera is fixed to the craft, not because it chases it.

use super::{Aboard, Vehicle};
use crate::flight_view::MOUSE_LOOK_SENSITIVITY;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use pbd_core::vehicle::Kind;

#[derive(Component)]
pub struct VehicleCamera;

/// How far the look turns, rad.
const PITCH_LIMIT: f32 = 1.45;
/// The chase camera's stand-off, m, per craft, and its range.
const CHASE_M: [f32; 3] = [20.0, 16.0, 8.0];
const CHASE_RANGE_M: [f32; 2] = [4.0, 80.0];
/// How far one wheel line moves it, as a share.
const ZOOM_STEP: f32 = 0.12;

pub(super) fn default_distance(kind: Kind) -> f32 {
    CHASE_M[match kind {
        Kind::Kestrel => 0,
        Kind::Tern => 1,
        Kind::Loon => 2,
    }]
}

/// Where the camera aboard is looking.
#[derive(Resource)]
pub struct VehicleView {
    /// In the seat, rather than the chase view.
    pub seat: bool,
    pub captured: bool,
    /// The look off the craft's bow, rad: about its up, then up from level.
    pub yaw: f32,
    pub pitch: f32,
    /// The chase stand-off, m; nought until a craft is boarded.
    pub distance: f32,
}

impl Default for VehicleView {
    fn default() -> Self {
        Self {
            seat: false,
            captured: false,
            yaw: 0.0,
            pitch: -0.25,
            distance: 0.0,
        }
    }
}

impl VehicleView {
    /// Boarding: the look faces ahead and the pointer is kept as it was.
    pub fn aboard(&mut self, captured: bool) {
        self.captured = captured;
        self.yaw = 0.0;
        self.pitch = if self.seat { 0.0 } else { -0.25 };
        self.distance = 0.0;
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Vehicle camera"),
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::default(),
        VehicleCamera,
    ));
}

/// The mouse, while aboard: raw displacement turns the look, the wheel moves
/// the chase camera in and out, and a click takes the pointer.
pub fn look(
    aboard: Res<Aboard>,
    mut view: ResMut<VehicleView>,
    mut pointer: crate::controls::Pointer,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse: Option<Res<AccumulatedMouseMotion>>,
    scroll: Option<Res<AccumulatedMouseScroll>>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if aboard.0.is_none() {
        return;
    }
    let menu = pointer.menu_holds(&mut view.captured);
    if !menu && buttons.is_some_and(|b| b.just_pressed(MouseButton::Left)) {
        view.captured = true;
    }
    if let Ok((window, mut cursor)) = windows.single_mut() {
        if !window.focused {
            view.captured = false;
        }
        cursor.visible = !view.captured;
        cursor.grab_mode = if view.captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
    if !view.captured {
        return;
    }
    let delta = mouse.map_or(Vec2::ZERO, |m| m.delta);
    if delta.is_finite() {
        view.yaw -= delta.x * MOUSE_LOOK_SENSITIVITY;
        view.yaw = (view.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        view.pitch =
            (view.pitch - delta.y * MOUSE_LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
    let lines = scroll.map_or(0.0, |s| s.delta.y);
    if lines.is_finite() && lines != 0.0 && view.distance > 0.0 {
        view.distance = (view.distance * (-lines.signum() * ZOOM_STEP).exp())
            .clamp(CHASE_RANGE_M[0], CHASE_RANGE_M[1]);
    }
}

/// Fix the camera to the craft aboard.
pub fn follow(
    aboard: Res<Aboard>,
    mut view: ResMut<VehicleView>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    contact: Option<Res<crate::planet::PlanetContact>>,
    vehicles: Query<&Vehicle>,
    mut cameras: Query<&mut Transform, With<VehicleCamera>>,
) {
    let Some(vehicle) = aboard.0.and_then(|e| vehicles.get(e).ok()) else {
        return;
    };
    let craft = &vehicle.craft;
    if view.distance <= 0.0 {
        view.distance = default_distance(craft.kind);
    }
    let orientation = craft.body.orientation.as_quat();
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    if view.seat {
        // The craft's own frame, turned by the head.
        camera.translation = (frame.center + craft.eye()).as_vec3();
        camera.rotation =
            orientation * Quat::from_rotation_y(view.yaw) * Quat::from_rotation_x(view.pitch);
        return;
    }
    // The chase view keeps the horizon level: it turns with the craft's
    // heading, not its roll or pitch, or a heeling boat would tip the world.
    let (eye, target, up) = super::chase::pose(craft, &view, |at| {
        super::ground_under(contact.as_deref(), at)
    });
    camera.translation = (frame.center + eye).as_vec3();
    camera.look_at((frame.center + target).as_vec3(), up);
}
