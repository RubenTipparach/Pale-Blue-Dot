//! Architecture mode's camera (`player-building` design B9): a build camera
//! over the plot, toggled in build mode, that eases where it goes.
//!
//! The camera is kept in the planet's own frame, in f64: the site as a unit
//! direction, the build layer as a column layer (so its radius is
//! `column::layer_altitude` over the planet's), the heading as a tangent
//! direction. Nothing of it is local, so a rebase of the simulation's origin
//! moves nothing here; it is put in render space only where the camera is
//! written ([`BuildView::shown_pose`], `walking::follow_walker`).
//!
//! While it is on the walker reads no keys and no mouse (as under a menu), so
//! the walker's pose is untouched; leaving it, the walker's camera is written
//! from that pose on the same frame, exactly as it was.

use crate::planet::PLANET_RADIUS;
use bevy::math::DVec3;
use bevy::prelude::*;

/// The tilt below the horizon, radians: the lowest and highest.
pub const TILT_MIN: f32 = 0.35;
pub const TILT_MAX: f32 = 1.48;
/// How far the camera stands from the site, metres.
pub const DISTANCE_MIN: f32 = 6.0;
pub const DISTANCE_MAX: f32 = 60.0;
/// Where it starts: 22 m back, 50 degrees down.
pub const START_DISTANCE: f32 = 22.0;
pub const START_TILT: f32 = 0.87;
/// The easing's time constant, seconds.
pub const EASE_S: f32 = 0.12;
/// A wheel step's zoom.
pub const ZOOM_STEP: f32 = 1.15;

/// Architecture mode's state.
#[derive(Resource, Clone, Debug, Default)]
pub struct BuildView {
    pub on: bool,
    /// The site: a unit direction from the planet's centre.
    pub focus: DVec3,
    /// The build layer: a column layer, where the cursor lands in open air.
    pub layer: u16,
    /// The way the camera looks, along the ground at the site.
    pub heading: DVec3,
    /// Radians below the horizon.
    pub tilt: f32,
    /// Metres from the site.
    pub distance: f32,
    /// The eye and the point looked at, planet-centred, as drawn: eased
    /// toward [`BuildView::target`].
    shown: Option<(DVec3, DVec3)>,
}

/// `v` made square to `up`, or another tangent where it is along it.
fn tangent(v: DVec3, up: DVec3) -> DVec3 {
    (v - up * v.dot(up))
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector())
}

impl BuildView {
    /// Turn it on over `focus` (a planet-centred point) at `layer`, looking
    /// along `look`, starting from the walker's `eye` looking at `at` so the
    /// camera pulls up and back from where the walker's was.
    pub fn enter(&mut self, focus: DVec3, layer: u16, look: DVec3, eye: DVec3, at: DVec3) {
        let up = focus.normalize_or(DVec3::Y);
        self.on = true;
        self.focus = up;
        self.layer = layer;
        self.heading = tangent(look, up);
        self.tilt = START_TILT;
        self.distance = START_DISTANCE;
        self.shown = Some((eye, at));
    }

    /// Turn it off: nothing of it is kept for the walker's camera.
    pub fn leave(&mut self) {
        self.on = false;
        self.shown = None;
    }

    /// The build layer's radius from the planet's centre, metres.
    pub fn radius(&self) -> f64 {
        f64::from(PLANET_RADIUS) + f64::from(pbd_core::column::layer_altitude(self.layer as usize))
    }

    /// Where the camera is going: its eye and the point it looks at,
    /// planet-centred.
    pub fn target(&self) -> (DVec3, DVec3) {
        let up = self.focus;
        let at = up * self.radius();
        let back = -tangent(self.heading, up);
        let (s, c) = (f64::from(self.tilt).sin(), f64::from(self.tilt).cos());
        (at + (back * c + up * s) * f64::from(self.distance), at)
    }

    /// Ease the shown camera toward its target over `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let (eye, at) = self.target();
        let k = f64::from(1.0 - (-dt.max(0.0) / EASE_S).exp());
        self.shown = Some(match self.shown {
            Some((e, a)) => (e + (eye - e) * k, a + (at - a) * k),
            None => (eye, at),
        });
    }

    /// Move the site `right` and `forward` metres along the ground, as the
    /// camera faces.
    pub fn pan(&mut self, right: f32, forward: f32) {
        let up = self.focus;
        let ahead = tangent(self.heading, up);
        let side = ahead.cross(up);
        let moved = up * self.radius() + ahead * f64::from(forward) + side * f64::from(right);
        let now = moved.normalize_or(up);
        self.focus = now;
        self.heading = tangent(ahead, now);
    }

    /// Turn round the site by `yaw` radians and tilt by `tilt`.
    pub fn orbit(&mut self, yaw: f32, tilt: f32) {
        let turn = bevy::math::DQuat::from_axis_angle(self.focus, f64::from(-yaw));
        self.heading = tangent(turn * self.heading, self.focus);
        self.tilt = (self.tilt + tilt).clamp(TILT_MIN, TILT_MAX);
    }

    /// Zoom by `steps` wheel steps, in toward the site for a positive one.
    pub fn zoom(&mut self, steps: i32) {
        self.distance = (self.distance * ZOOM_STEP.powi(-steps)).clamp(DISTANCE_MIN, DISTANCE_MAX);
    }

    /// The build layer moved `layers` up (down for a negative number).
    pub fn raise(&mut self, layers: i32) {
        self.layer = (i32::from(self.layer) + layers).clamp(0, i32::from(u16::MAX)) as u16;
    }

    /// The shown camera: its eye, planet-centred, and its rotation.
    pub fn shown_pose(&self) -> Option<(DVec3, Quat)> {
        let (eye, at) = self.shown?;
        let up = self.focus.as_vec3();
        let look = (at - eye).as_vec3().normalize_or(-up);
        Some((eye, Transform::IDENTITY.looking_to(look, up).rotation))
    }

    /// The camera as render space puts it: `centre` is the planet's centre
    /// there (`PlanetRenderFrame::center`).
    pub fn shown_transform(&self, centre: DVec3) -> Option<Transform> {
        let (eye, rotation) = self.shown_pose()?;
        Some(Transform {
            translation: (centre + eye).as_vec3(),
            rotation,
            ..default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> BuildView {
        let mut v = BuildView::default();
        let focus = DVec3::new(0.3, 0.8, 0.2).normalize() * 4800.0;
        v.enter(focus, 160, DVec3::X, focus * 1.0005, focus);
        v
    }

    /// The camera eases to a point over the site, above it and back along
    /// its heading, and looks down at it.
    #[test]
    fn it_eases_up_and_back_over_the_site() {
        let mut v = view();
        let (eye, at) = v.target();
        let up = v.focus;
        assert!((eye - at).length() - f64::from(START_DISTANCE) < 1e-6);
        assert!((eye - at).dot(up) > 0.0, "above the site");
        let start = v.shown.unwrap().0;
        v.step(EASE_S);
        let half = v.shown.unwrap().0;
        assert!(
            (half - eye).length() < (start - eye).length() * 0.5,
            "eased"
        );
        for _ in 0..120 {
            v.step(1.0 / 60.0);
        }
        assert!((v.shown.unwrap().0 - eye).length() < 1e-3, "arrived");
        let (_, rotation) = v.shown_pose().unwrap();
        let forward = (rotation * Vec3::NEG_Z).as_dvec3();
        assert!(
            forward.dot((at - eye).normalize()) > 0.999,
            "it looks at the site"
        );
    }

    /// Panning moves the site along the ground, orbiting turns the heading
    /// and holds the tilt in its range, zooming holds the distance in its,
    /// and the layer moves the site's radius.
    #[test]
    fn pan_orbit_zoom_and_layer() {
        let mut v = view();
        let before = v.focus * v.radius();
        v.pan(0.0, 10.0);
        let after = v.focus * v.radius();
        assert!(((after - before).length() - 10.0).abs() < 0.01);
        let h = v.heading;
        v.orbit(std::f32::consts::FRAC_PI_2, 10.0);
        assert!(v.heading.dot(h).abs() < 1e-6, "a quarter turn");
        assert_eq!(v.tilt, TILT_MAX);
        v.zoom(100);
        assert_eq!(v.distance, DISTANCE_MIN);
        v.zoom(-100);
        assert_eq!(v.distance, DISTANCE_MAX);
        let r = v.radius();
        v.raise(3);
        assert!(
            (v.radius() - r - 3.0).abs() < 1e-3,
            "a storey is three metres"
        );
        v.leave();
        assert!(!v.on && v.shown_pose().is_none());
    }
}
