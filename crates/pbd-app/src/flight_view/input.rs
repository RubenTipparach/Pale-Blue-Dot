//! Immediate captured-mouse view; physical ship attitude remains independently bounded.

use bevy::{
    ecs::system::SystemParam,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use super::{FlightInputState, FlightViewConfig, FlyMode, MOUSE_LOOK_SENSITIVITY};

#[derive(SystemParam)]
pub(super) struct PointerInput<'w> {
    buttons: Option<Res<'w, ButtonInput<MouseButton>>>,
    motion: Option<Res<'w, AccumulatedMouseMotion>>,
}

pub(super) fn read_pilot_input(
    time: Res<Time>,
    config: Res<FlightViewConfig>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pointer: PointerInput,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut intent: ResMut<FlightInputState>,
) {
    let Some(keys) = keys else { return };
    if !intent.enabled {
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        intent.reset = true;
    }
    if config.mode == FlyMode::Tour {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        intent.captured = false;
    }
    if pointer
        .buttons
        .is_some_and(|buttons| buttons.just_pressed(MouseButton::Left))
    {
        intent.captured = true;
    }
    if let Ok((window, mut cursor)) = windows.single_mut() {
        if !window.focused {
            intent.captured = false;
        }
        cursor.grab_mode = if intent.captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !intent.captured;
    }
    if keys.just_pressed(KeyCode::KeyX) {
        intent.dampeners = !intent.dampeners;
    }
    intent.axes = Vec3::ZERO;
    intent.cruise = false;
    intent.brake = keys.pressed(KeyCode::KeyB);
    if !intent.captured {
        return;
    }
    let axis = |positive, negative| {
        i32::from(keys.pressed(positive)) as f32 - i32::from(keys.pressed(negative)) as f32
    };
    intent.axes = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        i32::from(keys.pressed(KeyCode::Space)) as f32
            - i32::from(keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
                as f32,
        axis(KeyCode::KeyS, KeyCode::KeyW),
    )
    .clamp_length_max(1.0);
    intent.cruise = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let delta = pointer.motion.map_or(Vec2::ZERO, |mouse| mouse.delta);
    let delta = if delta.is_finite() { delta } else { Vec2::ZERO };
    let roll = axis(KeyCode::KeyQ, KeyCode::KeyE) * time.delta_secs().min(0.1) * 1.2;
    // Mouse displacement is already accumulated for this frame: no dt factor,
    // input clamp, easing, or lead limit relative to the slower physical ship.
    intent.target_rotation = (intent.target_rotation
        * Quat::from_rotation_y(-delta.x * MOUSE_LOOK_SENSITIVITY)
        * Quat::from_rotation_x(-delta.y * MOUSE_LOOK_SENSITIVITY)
        * Quat::from_rotation_z(roll))
    .normalize();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<FlightViewConfig>()
            .init_resource::<FlightInputState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .add_systems(Update, read_pilot_input);
        app
    }

    #[test]
    fn captured_mouse_moves_up_and_right_and_cruise_diagonals_stay_normalized() {
        let mut app = input_app();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            for key in [
                KeyCode::KeyW,
                KeyCode::KeyD,
                KeyCode::Space,
                KeyCode::ShiftRight,
            ] {
                keys.press(key);
            }
        }
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(35.0, -25.0);
        app.update();
        let intent = app.world().resource::<FlightInputState>();
        let look = intent.target_rotation * Vec3::NEG_Z;
        assert!(
            look.x > 0.0 && look.y > 0.0,
            "mouse signs reversed: {look:?}"
        );
        assert!(intent.captured && intent.cruise);
        assert!((intent.axes.length() - 1.0).abs() < 1e-6);
        assert!(intent.axes.x > 0.0 && intent.axes.y > 0.0 && intent.axes.z < 0.0);
    }

    #[test]
    fn escape_releases_pilot_input_and_right_control_descends() {
        let mut app = input_app();
        app.world_mut().resource_mut::<FlightInputState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlRight);
        app.update();
        assert_eq!(app.world().resource::<FlightInputState>().axes, Vec3::NEG_Y);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        let intent = app.world().resource::<FlightInputState>();
        assert!(!intent.captured);
        assert_eq!(intent.axes, Vec3::ZERO);
    }

    #[test]
    fn fast_mouse_swipe_has_full_displacement_at_any_frame_duration() {
        let current = Quat::from_rotation_z(1.1) * Quat::from_rotation_x(2.0);
        let delta = Vec2::new(1_400.0, -350.0);
        let expected = current
            * Quat::from_rotation_y(-delta.x * MOUSE_LOOK_SENSITIVITY)
            * Quat::from_rotation_x(-delta.y * MOUSE_LOOK_SENSITIVITY);
        for frame_duration in [0.0, 1.0 / 240.0, 1.0 / 30.0] {
            let mut app = input_app();
            {
                let mut intent = app.world_mut().resource_mut::<FlightInputState>();
                intent.captured = true;
                intent.target_rotation = current;
            }
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f64(frame_duration));
            app.world_mut()
                .resource_mut::<AccumulatedMouseMotion>()
                .delta = delta;
            app.update();
            let view = app.world().resource::<FlightInputState>().target_rotation;
            assert!(view.angle_between(expected) < 0.001);
            assert!(view.angle_between(current) > 2.0);
            app.world_mut()
                .resource_mut::<AccumulatedMouseMotion>()
                .delta = Vec2::ZERO;
            app.update();
            assert!(
                app.world()
                    .resource::<FlightInputState>()
                    .target_rotation
                    .angle_between(view)
                    < 0.001,
                "view must stop immediately when mouse displacement stops"
            );
        }
    }

    #[test]
    fn disabled_flight_leaves_pointer_and_keyboard_to_walking() {
        let mut app = input_app();
        app.world_mut()
            .resource_mut::<FlightInputState>()
            .set_enabled(false);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::KeyW);
            keys.press(KeyCode::KeyR);
        }
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::splat(500.0);
        app.update();
        let intent = app.world().resource::<FlightInputState>();
        assert!(!intent.is_captured());
        assert!(!intent.reset);
        assert!(intent.brake);
        assert_eq!(intent.axes, Vec3::ZERO);
        assert_eq!(intent.view_rotation(), Quat::IDENTITY);
    }
}
