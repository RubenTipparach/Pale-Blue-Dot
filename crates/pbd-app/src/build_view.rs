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

/// The cutaway (design B11): what is cut over the build layer's floor, how
/// far round the site, and how high what stands beyond the site keeps
/// standing (a storey). `field_lit.wgsl` declares `CUT_REACH_M` alike.
pub const CUT_OVER_M: f32 = 1.2;
pub const CUT_REACH_M: f32 = 40.0;
pub const CUT_STOREY_M: f32 = 3.0;
/// Two presses within this long and [`DOUBLE_CLICK_PX`] are a double-click
/// (design B11); a single click waits this long before it places.
pub const DOUBLE_CLICK_S: f32 = 0.3;
pub const DOUBLE_CLICK_PX: f32 = 6.0;

/// The cutaway as the field-lit shader reads it (design B11), in the
/// planet's frame: the building planner's isometric cutaway (M6 version 3)
/// stood up in 3D. Over [`Cutaway::cut_m`] everything round the site is cut
/// but what stands beyond the site from the camera, which keeps standing to
/// the storey's ceiling, as the planner keeps its back outer walls whole.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cutaway {
    /// The site: a unit direction from the planet's centre.
    pub up: Vec3,
    /// The cut's radius from the planet's centre, metres.
    pub cut_m: f32,
    /// Along the ground away from the camera, at the site.
    pub far: Vec3,
    /// The storey's ceiling's radius, metres.
    pub ceiling_m: f32,
}

impl Cutaway {
    /// The planner's rule for a wall of the shown storey: an outer wall on
    /// the far side stands whole, every other is cut low. The planner's
    /// painter and [`Cutaway::hides`] ask it alike.
    pub fn stands_full(outer: bool, beyond: bool) -> bool {
        outer && beyond
    }

    /// Whether the cutaway takes away a planet-centred point.
    pub fn hides(&self, p: Vec3) -> bool {
        let r = p.length();
        if r <= self.cut_m {
            return false;
        }
        let side = p - self.up * p.dot(self.up);
        if side.length() > CUT_REACH_M {
            return false;
        }
        // Whatever stands beyond the site is the far wall's, as an outer
        // wall on the far side is in the planner's view.
        !(Self::stands_full(true, side.dot(self.far) > 0.0) && r <= self.ceiling_m)
    }

    /// As `field_lit.wgsl`'s `cut` and `cut_far` read it; zeros when off.
    pub fn packed(cut: Option<Self>) -> [Vec4; 2] {
        cut.map_or([Vec4::ZERO; 2], |c| {
            [c.up.extend(c.cut_m), c.far.extend(c.ceiling_m)]
        })
    }
}

/// A double-click from presses (design B11): each press either starts the
/// wait for a second or, within [`DOUBLE_CLICK_S`] and [`DOUBLE_CLICK_PX`]
/// of the first, is the double-click; a press left alone through the wait is
/// a single click.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clicks {
    pending: Option<(f64, Vec2)>,
}

/// What a press or the wait made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    None,
    Single,
    Double,
}

impl Clicks {
    /// A press at `now` seconds at the pointer's `at`.
    pub fn press(&mut self, now: f64, at: Vec2) -> Click {
        if let Some((t, p)) = self.pending
            && now - t <= f64::from(DOUBLE_CLICK_S)
            && p.distance(at) <= DOUBLE_CLICK_PX
        {
            self.pending = None;
            return Click::Double;
        }
        let was = self.pending.take();
        self.pending = Some((now, at));
        // A press far from the one waiting: that one was a single click.
        if was.is_some() {
            Click::Single
        } else {
            Click::None
        }
    }

    /// The wait at `now`: a single click once it has passed.
    pub fn wait(&mut self, now: f64) -> Click {
        match self.pending {
            Some((t, _)) if now - t > f64::from(DOUBLE_CLICK_S) => {
                self.pending = None;
                Click::Single
            }
            _ => Click::None,
        }
    }

    /// Forget a press (the mode closed).
    pub fn clear(&mut self) {
        self.pending = None;
    }
}

/// Set the walker down with its feet at `feet` (planet-centred), looking
/// along `look`, and leave architecture mode (design B11, the mockup's
/// `walkFrom`): the walker's camera takes over from its new pose. The point
/// goes into physics space through the origin, moved there first.
pub fn walk_there(world: &mut World, feet: Vec3, look: Vec3) {
    let planet = feet.as_dvec3();
    crate::sim_origin::recenter_on_point(world, planet);
    let origin = crate::sim_origin::origin_of(world);
    let local = origin.to_local(pbd_core::world_frame::WorldPos::new(
        crate::sim_origin::HOME,
        planet,
    ));
    let up = origin.up_at(local);
    let eye = local + up * (crate::walking::EYE_HEIGHT + 0.02);
    let flat = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let view = Transform::IDENTITY.looking_to(flat, up).rotation;
    crate::walking::drop_walker(world, eye, Vec3::ZERO, view);
    if let Some(mut v) = world.get_resource_mut::<BuildView>() {
        v.leave();
    }
}

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
    /// The cutaway is on (End, design B11).
    pub cut: bool,
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
        self.cut = false;
        self.shown = Some((eye, at));
    }

    /// Turn it off: nothing of it is kept for the walker's camera.
    pub fn leave(&mut self) {
        self.on = false;
        self.cut = false;
        self.shown = None;
    }

    /// The build layer's radius from the planet's centre, metres.
    pub fn radius(&self) -> f64 {
        f64::from(PLANET_RADIUS) + f64::from(pbd_core::column::layer_altitude(self.layer as usize))
    }

    /// The cutaway while it is on (design B11), in the planet's frame.
    pub fn cutaway(&self) -> Option<Cutaway> {
        if !(self.on && self.cut) {
            return None;
        }
        let floor = self.radius();
        Some(Cutaway {
            up: self.focus.as_vec3(),
            cut_m: (floor + f64::from(CUT_OVER_M)) as f32,
            far: tangent(self.heading, self.focus).as_vec3(),
            ceiling_m: (floor + f64::from(CUT_STOREY_M)) as f32,
        })
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

    /// Design B11: the cutaway is off until asked; on, it cuts over the
    /// build layer's floor plus `CUT_OVER_M` near the site, keeps what stands
    /// beyond the site to the storey's ceiling, and leaves alone what is
    /// under the cut or out of reach; Page Up raises the cut a storey.
    #[test]
    fn the_cutaway_cuts_over_the_build_layer_but_the_far_walls() {
        let mut v = view();
        assert_eq!(v.cutaway(), None, "off when the mode opens");
        assert_eq!(Cutaway::packed(None), [Vec4::ZERO; 2]);
        v.cut = true;
        let c = v.cutaway().unwrap();
        let up = v.focus;
        let floor = v.radius();
        let far = tangent(v.heading, up);
        let side = up.cross(far);
        let at = |along: f64, across: f64, over: f64| {
            (up * (floor + over) + far * along + side * across).as_vec3()
        };
        assert!(!c.hides(at(-3.0, 0.0, 0.5)), "under the cut");
        assert!(c.hides(at(-3.0, 0.0, 2.0)), "a near wall's top");
        assert!(!c.hides(at(3.0, 1.0, 2.5)), "a far wall stands");
        assert!(c.hides(at(3.0, 1.0, 4.0)), "but not over the storey");
        assert!(c.hides(at(-3.0, 0.0, 9.0)), "the roof");
        assert!(!c.hides(at(-60.0, 0.0, 9.0)), "out of reach");
        let packed = Cutaway::packed(Some(c));
        assert!((f64::from(packed[0].w) - (floor + f64::from(CUT_OVER_M))).abs() < 1e-2);
        v.raise(3);
        assert!((v.cutaway().unwrap().cut_m - c.cut_m - 3.0).abs() < 1e-2);
        v.leave();
        assert!(!v.cut && v.cutaway().is_none());
        // The shader cuts by the same reach.
        let shader = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/shaders/field_lit.wgsl"
        ));
        assert!(shader.contains(&format!("const CUT_REACH_M: f32 = {CUT_REACH_M:?};")));
    }

    /// Design B11: two presses close in time and place are a double-click;
    /// a press left alone is a single click once the wait has passed, never
    /// before; a press far from the first makes the first a single.
    #[test]
    fn a_double_click_and_a_single() {
        let mut c = Clicks::default();
        assert_eq!(c.press(1.0, Vec2::ZERO), Click::None);
        assert_eq!(c.wait(1.1), Click::None, "still waiting");
        assert_eq!(c.press(1.2, Vec2::new(2.0, 1.0)), Click::Double);
        assert_eq!(c.wait(2.0), Click::None, "the double took both");
        assert_eq!(c.press(3.0, Vec2::ZERO), Click::None);
        assert_eq!(
            c.wait(3.0 + f64::from(DOUBLE_CLICK_S) + 0.01),
            Click::Single
        );
        assert_eq!(c.press(4.0, Vec2::ZERO), Click::None);
        assert_eq!(c.press(4.1, Vec2::new(40.0, 0.0)), Click::Single, "too far");
        assert_eq!(c.wait(4.5), Click::Single);
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
