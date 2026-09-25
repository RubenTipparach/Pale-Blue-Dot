//! The far-side route (`far-side-flight`): take off from dry land, climb at a
//! set angle to a cruise height above the atmosphere, follow the great
//! circle, and glide down at the same angle to land on the far side.
//!
//! One rule shapes the whole flight: the target height at a point is the
//! lowest of the cruise height, the arc flown times the climb slope, and the
//! arc remaining times that slope. The climb, the cruise and the descent are
//! the three pieces of that one continuous line, so no phase change can jerk
//! the ship or the camera. Like `tour.rs`, the guidance only REQUESTS an
//! acceleration and an attitude from the ship controller; the physics stays
//! authoritative.
//!
//! The camera is a rig on the ship, not the player's mouse: it looks down at
//! the planet as the horizon drops away, sways and banks slowly, and turns to
//! the landing site on the way down. `CLAUDE.md`'s rule against easing is for
//! raw mouse look; this camera eases, without overshoot, because no hand is on
//! it.

use bevy::prelude::*;

use super::FlightViewConfig;
use crate::planet::{self, PLANET_RADIUS};

/// The route's state: where it started, which way it flies, where it lands,
/// and how far it has come. Written once per physics step.
#[derive(Resource, Clone, Copy, Debug)]
pub struct RouteState {
    /// The start's direction from the centre.
    pub start: Vec3,
    /// The great circle's normal: travel is `normal x up`.
    pub normal: Vec3,
    /// The arc to the destination, radians: the antipode, or the dry land
    /// nearest it along the circle.
    pub destination_rad: f32,
    /// Arc flown so far, radians, unwrapped.
    pub flown_rad: f64,
    previous_direction: Vec3,
    pub elapsed_s: f32,
    /// When the ship was first down at the destination.
    pub landed_at_s: Option<f32>,
    pub completed: bool,
    /// The camera rig's current orientation, eased toward its target.
    pub camera: Quat,
    camera_ready: bool,
    /// Instruments for `--verify-route`.
    pub peak_height_m: f32,
    pub min_airborne_clearance_m: f32,
    pub max_speed_mps: f32,
    pub max_camera_rate_rad_s: f32,
    pub touchdown_error_m: f32,
}

impl Default for RouteState {
    fn default() -> Self {
        Self {
            start: Vec3::Y,
            normal: Vec3::Z,
            destination_rad: std::f32::consts::PI,
            flown_rad: 0.0,
            previous_direction: Vec3::Y,
            elapsed_s: 0.0,
            landed_at_s: None,
            completed: false,
            camera: Quat::IDENTITY,
            camera_ready: false,
            peak_height_m: 0.0,
            min_airborne_clearance_m: f32::INFINITY,
            max_speed_mps: 0.0,
            max_camera_rate_rad_s: 0.0,
            touchdown_error_m: f32::INFINITY,
        }
    }
}

/// How far the camera looks below the horizon, radians: with the 75 degree
/// vertical field of view, 11 degrees puts the horizon about a third of the
/// way down the frame.
const HORIZON_BELOW_CENTRE_RAD: f32 = 0.20;

/// Seconds the ship holds still on the ground before it lifts.
pub const HOLD_S: f32 = 1.5;
/// Seconds it rests after touchdown before the route counts as complete.
pub const SETTLE_S: f32 = 2.0;
/// How far from the destination, and how high, still counts as down there.
pub const LANDED_WITHIN_M: f32 = 4.0;
pub const LANDED_BELOW_M: f32 = 3.0;

impl RouteState {
    /// A route from `start` to the dry land nearest its antipode, along the
    /// great circle through the two. `fallback_normal` is the circle flown
    /// when the far side has no land at all.
    pub fn new(start: Vec3, fallback_normal: Vec3) -> Self {
        let start = start.normalize_or(Vec3::Y);
        let (normal, destination_rad) = match far_side_landing(start) {
            Some(site) => (
                start.cross(site).normalize_or(fallback_normal),
                start.dot(site).clamp(-1.0, 1.0).acos(),
            ),
            None => (fallback_normal.normalize_or(Vec3::Z), std::f32::consts::PI),
        };
        Self {
            start,
            normal,
            destination_rad,
            previous_direction: start,
            ..Default::default()
        }
    }

    /// Metres of arc flown and left, on the sphere the bands and the tour
    /// measure on.
    pub fn arcs_m(&self) -> (f32, f32) {
        let flown = self.flown_rad as f32 * PLANET_RADIUS;
        let left = (self.destination_rad - self.flown_rad as f32) * PLANET_RADIUS;
        (flown.max(0.0), left.max(0.0))
    }

    /// The destination's direction from the centre.
    pub fn destination(&self) -> Vec3 {
        Quat::from_axis_angle(self.normal, self.destination_rad) * self.start
    }

    /// Advance the arc flown and the clock, after a physics step.
    pub fn advance(&mut self, direction: Vec3, dt: f32) {
        let sine = self.normal.dot(self.previous_direction.cross(direction));
        let cosine = self.previous_direction.dot(direction).clamp(-1.0, 1.0);
        self.flown_rad += sine.atan2(cosine) as f64;
        self.previous_direction = direction;
        self.elapsed_s += dt;
    }
}

/// The dry land nearest the antipode of `start`, anywhere on the sphere: the
/// route lands on ground, not in the sea. A Fibonacci sweep, as the capture
/// spawns search, with a margin above the waterline. Never the start's own
/// hemisphere, so a route is always a flight to the far side.
fn far_side_landing(start: Vec3) -> Option<Vec3> {
    let antipode = -start;
    let count = 40_000;
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    (0..count)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = golden * i as f32;
            Vec3::new(a.cos() * r, y, a.sin() * r)
        })
        .filter(|d| d.dot(start) < -0.5 && planet::surface_height(*d) > 2.0)
        .max_by(|a, b| a.dot(antipode).total_cmp(&b.dot(antipode)))
}

/// Where the route wants the ship: a velocity, from the one height line, and
/// the acceleration that gets it there.
pub(super) fn route_command(
    position: Vec3,
    velocity: Vec3,
    route: &RouteState,
    config: &FlightViewConfig,
) -> (Vec3, Quat) {
    let radius = position.length().max(1.0);
    let up = position / radius;
    let tangent = route.normal.cross(up).normalize_or(Vec3::X);
    let height = radius - planet::terrain_radius(up);
    let (flown, left) = route.arcs_m();
    let slope = config.route_climb_angle_deg.to_radians().tan();
    let tangential_speed = velocity.dot(tangent);

    let desired = if route.elapsed_s < HOLD_S || route.landed_at_s.is_some() {
        // Standing on the ground, before the lift and after the landing.
        Vec3::ZERO
    } else {
        // The height line and its slope along the ground here, from the
        // numerical derivative of the same line so the two cannot disagree.
        let target = height_line(flown, left, slope, config);
        let gradient = (height_line(flown + 1.0, left - 1.0, slope, config)
            - height_line(flown - 1.0, left + 1.0, slope, config))
            / 2.0;
        // Speed along the path: the climb speed on the steep legs, the cruise
        // speed on the level, eased up from the start and down to the
        // destination at the route's ground acceleration.
        let steep = (gradient.abs() / slope).clamp(0.0, 1.0);
        let path = config.route_speed + (config.route_climb_speed - config.route_speed) * steep;
        let a = config.route_ground_accel;
        let along = path
            .min((4.0f32.powi(2) + 2.0 * a * flown).sqrt())
            .min((2.0 * a * left).sqrt());
        let mut ground = along / (1.0 + gradient * gradient).sqrt();
        if left < 0.5 {
            ground = 0.0;
        }
        let mut vertical = gradient * ground + (target - height) * 0.8;
        // The flare: slower the nearer the ground, easing to a stop just above
        // the terrain guard's height. Descending at the touchdown speed all
        // the way, the guard caught the ship at touchdown instead.
        let above = height - (config.minimum_clearance + 0.4);
        let floor =
            (config.route_touchdown_mps * (above / 2.0).clamp(0.0, 1.0)).max(above.max(0.0) * 0.8);
        vertical = vertical.max(-floor);
        tangent * ground + up * vertical
    };
    // Stay on the circle, as the tour does.
    let plane_error = position.dot(route.normal);
    let plane = -plane_error * 0.3 - velocity.dot(route.normal) * 1.5;
    // Feed-forward the curve of the path over a round world, then close the
    // velocity error.
    let centripetal = -up * tangential_speed.powi(2) / radius;
    let acceleration = ((desired - velocity) * 1.4 + centripetal + route.normal * plane)
        .clamp_length_max(config.acceleration * 0.96);
    // The ship's nose follows its path; at a standstill, the direction of
    // travel.
    let heading = if velocity.length() > 3.0 {
        velocity.normalize()
    } else {
        tangent
    };
    let side = heading.cross(up);
    let rotation = if side.length_squared() > 1e-6 {
        Transform::IDENTITY.looking_to(heading, up).rotation
    } else {
        Transform::IDENTITY.looking_to(tangent, up).rotation
    };
    (acceleration, rotation)
}

/// The route's one height line at a point `flown` metres from the start and
/// `left` from the destination: the lowest of the climb, the cruise and the
/// descent, its corners rounded by `route_corner_m` (a polynomial smooth
/// minimum) so the ship is never asked to turn faster than it can at speed.
/// The climb is offset a few metres so the lift begins at once.
fn height_line(flown: f32, left: f32, slope: f32, config: &FlightViewConfig) -> f32 {
    let climb = (flown + 6.0) * slope;
    let descend = left.max(0.0) * slope;
    let k = config.route_corner_m;
    let line = smooth_min(
        smooth_min(climb, config.route_cruise_height_m, k),
        descend,
        k,
    );
    // The rounding lifts the ends a little off the ground; never below zero,
    // and exactly the descent line in the last metres, so it lands.
    line.min(descend.max(0.0) + (left * 0.02).min(8.0)).max(0.0)
}

/// Polynomial smooth minimum (Quilez): equal to `min` when `a` and `b` are
/// more than `k` apart, a smooth blend within it.
fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

/// Where the camera wants to look at time `t`, from the ship's pose and the
/// route. A pure function, so the headless check measures the same rig the
/// window draws.
pub fn camera_target(
    position: Vec3,
    route: &RouteState,
    config: &FlightViewConfig,
    t: f32,
) -> Quat {
    let radius = position.length().max(1.0);
    let up = position / radius;
    let tangent = route.normal.cross(up).normalize_or(Vec3::X);
    let ground = planet::terrain_radius(up);
    let height = (radius - ground).max(0.0);
    let (_, left) = route.arcs_m();
    let motion = if config.route_reduced_motion {
        0.3
    } else {
        1.0
    };

    // How far below level the horizon lies: 52 degrees from 3 km up on this
    // small world. The camera pitches down with it and a little more, so the
    // horizon (the planet's limb, aloft) stays in the upper third of the frame
    // and the planet fills the rest. Held level instead, the climb filmed
    // black space with the planet a sliver at the bottom.
    let dip = (ground / radius).clamp(-1.0, 1.0).acos();
    let aloft = smoothstep(40.0, config.route_cruise_height_m * 0.8, height);
    let pitch = dip + HORIZON_BELOW_CENTRE_RAD;
    let mut look = tangent * pitch.cos() - up * pitch.sin();

    // On the way down, turn to the landing site and keep it framed.
    let slope = config.route_climb_angle_deg.to_radians().tan();
    let descent_start = config.route_cruise_height_m / slope;
    let approach = 1.0 - smoothstep(descent_start * 0.4, descent_start * 1.6, left);
    if approach > 0.0 {
        // Aim a little past the touchdown point, along the way the ship is
        // going: aimed AT it, the direction swings fast as the ship passes over.
        let site_up = route.destination();
        let beyond = route.normal.cross(site_up).normalize_or(tangent);
        let aim = site_up * planet::terrain_radius(site_up) + beyond * 250.0;
        let to_site = (aim - position).normalize_or(look);
        look = look.lerp(to_site, approach).normalize_or(look);
    }

    // A pilot's drift: a slow sway side to side, banking into it, calmed on the
    // ground at both ends and in the final approach.
    let calm = aloft.min(1.0 - approach * 0.8) * motion;
    let period = 16.0;
    let phase = std::f32::consts::TAU * t / period;
    let sway = config.route_sway_deg.to_radians() * phase.sin() * calm;
    let bank = -config.route_bank_deg.to_radians() * phase.cos() * calm;
    look = Quat::from_axis_angle(up, sway) * look;
    let forward = look.normalize_or(tangent);
    let level = Transform::IDENTITY.looking_to(forward, up).rotation;
    level * Quat::from_rotation_z(bank)
}

/// Ease the camera toward its target: first order, so it never overshoots,
/// with the route's ease time. The first frame snaps.
pub fn ease_camera(
    route: &mut RouteState,
    target: Quat,
    dt: f32,
    ease_s: f32,
    max_rate_rad_s: f32,
) {
    if !route.camera_ready {
        route.camera = target;
        route.camera_ready = true;
        return;
    }
    let previous = route.camera;
    // First order toward the target, and never faster than the rate cap: an
    // eased camera still whips if its target jumps.
    let blend = 1.0 - (-dt / ease_s.max(1e-3)).exp();
    let wanted = previous.angle_between(target) * blend;
    let allowed = max_rate_rad_s * dt;
    let step = if wanted > allowed && wanted > 0.0 {
        blend * allowed / wanted
    } else {
        blend
    };
    route.camera = previous.slerp(target, step).normalize();
    if dt > 0.0 {
        let rate = previous.angle_between(route.camera) / dt;
        route.max_camera_rate_rad_s = route.max_camera_rate_rad_s.max(rate);
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_destination_is_dry_land_near_the_antipode() {
        let start = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let east = Vec3::Y.cross(start).normalize();
        let route = RouteState::new(start, start.cross(east));
        // Dry land on the far hemisphere, as near the antipode as land goes.
        assert!(route.destination_rad > 2.0, "{} rad", route.destination_rad);

        assert!(planet::surface_height(route.destination()) > 2.0);
    }

    #[test]
    fn the_camera_target_is_a_unit_rotation_everywhere_on_the_route() {
        let config = FlightViewConfig::default();
        let start = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let east = Vec3::Y.cross(start).normalize();
        let mut route = RouteState::new(start, start.cross(east));
        for i in 0..=40 {
            route.flown_rad = route.destination_rad as f64 * i as f64 / 40.0;
            let direction = Quat::from_axis_angle(route.normal, route.flown_rad as f32) * start;
            let height = [2.0, 400.0, 3_000.0][i % 3];
            let position = direction * (planet::terrain_radius(direction) + height);
            let q = camera_target(position, &route, &config, i as f32 * 1.7);
            assert!(q.is_finite() && (q.length() - 1.0).abs() < 1e-4);
        }
    }
}
