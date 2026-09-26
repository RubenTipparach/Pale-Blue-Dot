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

use std::sync::Arc;

use pbd_core::atmosphere::Atmosphere;

use super::FlightViewConfig;
use crate::planet::{self, PLANET_RADIUS};

/// Which route is flown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RouteKind {
    /// Take off, climb at 60 degrees to 3 km, cruise round, land on the far side.
    #[default]
    FarSide,
    /// Low and slow: through the cloud layer, then over the most interesting
    /// ground within reach, following the terrain (`far-side-flight` design,
    /// "The scenic route").
    Scenic,
}

impl RouteKind {
    pub fn name(self) -> &'static str {
        match self {
            RouteKind::FarSide => "far-side",
            RouteKind::Scenic => "scenic",
        }
    }
}

/// The scenic route's target height along its path: the planet-centre radius
/// to fly at, every `step_m` metres of arc from the start.
#[derive(Debug)]
pub struct HeightProfile {
    pub step_m: f32,
    pub radius: Vec<f32>,
    /// The ground's radius at the same samples.
    pub ground: Vec<f32>,
}

impl HeightProfile {
    /// Height above the ground to fly at, `s` metres along the path,
    /// interpolated between samples.
    fn above_ground(&self, s: f32) -> f32 {
        let at = (s / self.step_m).clamp(0.0, (self.radius.len() - 1) as f32);
        let i = at.floor() as usize;
        let j = (i + 1).min(self.radius.len() - 1);
        let f = at - i as f32;
        let r = self.radius[i] + (self.radius[j] - self.radius[i]) * f;
        let g = self.ground[i] + (self.ground[j] - self.ground[i]) * f;
        r - g
    }
}

/// The route's state: where it started, which way it flies, where it lands,
/// and how far it has come. Written once per physics step.
#[derive(Resource, Clone, Debug)]
pub struct RouteState {
    pub kind: RouteKind,
    /// Whether the route is laid out: the scenic route waits for the weather.
    pub planned: bool,
    /// The scenic route's height profile; none for the far side.
    pub profile: Option<Arc<HeightProfile>>,
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
    /// Where along the path, metres, the airborne clearance was least.
    pub min_airborne_at_m: f32,
    pub max_speed_mps: f32,
    pub max_camera_rate_rad_s: f32,
    pub touchdown_error_m: f32,
}

impl Default for RouteState {
    fn default() -> Self {
        Self {
            kind: RouteKind::FarSide,
            planned: true,
            profile: None,
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
            min_airborne_at_m: 0.0,
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

/// How far the scenic route looks for interesting ground, metres of arc.
/// 5 km is 60 degrees of arc on this 4.8 km world; 9 km was over a quarter of
/// the way round, and the first recording flew into the night over open sea.
pub const SCENIC_REACH_M: f32 = 5_000.0;
/// The sun's least height over the path, as the dot of the local up with the
/// sun: the scenic route stays in daylight.
const SCENIC_MIN_SUN: f32 = 0.2;
/// The scenic route's cloud leg: its height above sea level (the middle of the
/// cloud layer, 300-750 m up) and the share of the path it covers.
/// Cover that counts as a cloud to fly into, and how much a metre of it is
/// worth against the land's interest when picking the heading: enough that a
/// heading with cloud always wins over one without.
const SCENIC_CLOUD_COVER: f32 = 0.55;
const SCENIC_CLOUD_WEIGHT: f32 = 12.0;

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

    /// A scenic route waiting to be planned: it plans once the weather is in
    /// hand (`plan_scenic`), and holds the ship on the ground until then.
    pub fn scenic_pending(start: Vec3) -> Self {
        Self {
            kind: RouteKind::Scenic,
            planned: false,
            start: start.normalize_or(Vec3::Y),
            previous_direction: start.normalize_or(Vec3::Y),
            ..Default::default()
        }
    }

    /// The scenic route: from `start`, along the heading that flies INTO the
    /// most cloud and then over the most interesting ground within
    /// `SCENIC_REACH_M`, to the flattest dry ground near its end. Scored from
    /// the weather (`atmosphere`, when there is one) and the terrain
    /// generator, and logged.
    pub fn scenic(
        start: Vec3,
        config: &FlightViewConfig,
        atmosphere: Option<&Atmosphere>,
        sun: Option<Vec3>,
    ) -> Self {
        let start = start.normalize_or(Vec3::Y);
        let east = Vec3::Y.cross(start).normalize_or(Vec3::X);
        let at = |normal: Vec3, s: f32| Quat::from_axis_angle(normal, s / PLANET_RADIUS) * start;
        struct Pick {
            score: f32,
            normal: Vec3,
            landing: f32,
            cloud: Option<(f32, f32, f32)>,
            parts: [f32; 3],
        }
        let mut best: Option<Pick> = None;
        for step in 0..36 {
            let heading = Quat::from_axis_angle(start, step as f32 * 10f32.to_radians()) * east;
            // Travel along `heading` from `start` is `normal x up` for this normal.
            let normal = start.cross(heading).normalize_or(Vec3::Z);
            let samples: Vec<(f32, f32, pbd_core::planet_gen::Biome)> =
                (0..=((SCENIC_REACH_M - 800.0) / 50.0) as usize)
                    .map(|i| {
                        let s = 800.0 + i as f32 * 50.0;
                        let d = at(normal, s);
                        (
                            s,
                            planet::surface_height(d),
                            pbd_core::planet_gen::biome(&planet::TERRAIN, d),
                        )
                    })
                    .collect();
            // The land: relief, coastline crossings, how many kinds of ground.
            let relief: f32 = samples.windows(2).map(|w| (w[1].1 - w[0].1).abs()).sum();
            let coast = samples
                .windows(2)
                .filter(|w| (w[0].1 > 0.5) != (w[1].1 > 0.5))
                .count() as f32;
            let mut kinds: Vec<_> = samples.iter().map(|x| x.2).collect();
            kinds.sort();
            kinds.dedup();
            // In daylight all the way, or not at all.
            if let Some(sun) = sun
                && samples
                    .iter()
                    .any(|x| at(normal, x.0).dot(sun) < SCENIC_MIN_SUN)
            {
                continue;
            }
            // Open sea is a flat, dark picture: the share of the path over it
            // costs as much as a whole coastline gains.
            let sea = samples.iter().filter(|x| x.1 <= 0.5).count() as f32 / samples.len() as f32;
            let land = relief + coast * 80.0 + kinds.len() as f32 * 150.0 - sea * 1_500.0;
            // The cloud: the longest unbroken stretch of solid cover in the
            // first half of the path, which the ship will fly through.
            let mut cloud = None;
            if let Some(air) = atmosphere {
                let (mut run_start, mut longest) = (None, (0.0f32, 0.0f32, 0.0f32));
                let mut tops = 0.0f32;
                for &(s, _, _) in samples.iter().filter(|x| x.0 < SCENIC_REACH_M * 0.6) {
                    let sample = air.sample(at(normal, s));
                    if sample.cover >= SCENIC_CLOUD_COVER {
                        let from = *run_start.get_or_insert(s);
                        tops = tops.max(sample.cloud_top);
                        if s - from > longest.1 - longest.0 {
                            longest = (from, s, tops);
                        }
                    } else {
                        run_start = None;
                        tops = 0.0;
                    }
                }
                if longest.1 - longest.0 >= 200.0 {
                    cloud = Some(longest);
                }
            }
            let cloud_m = cloud.map_or(0.0, |c| c.1 - c.0);
            // A landing: the flattest dry ground in the last third of the reach.
            let landing = samples
                .windows(3)
                .filter(|w| w[1].0 >= SCENIC_REACH_M * 0.65 && w[1].1 > 2.0)
                .min_by(|a, b| {
                    let slope = |w: &[(f32, f32, _)]| (w[2].1 - w[0].1).abs();
                    slope(a).total_cmp(&slope(b))
                })
                .map(|w| w[1].0);
            let Some(landing) = landing else { continue };
            let score = cloud_m * SCENIC_CLOUD_WEIGHT + land;
            if best.as_ref().is_none_or(|b| score > b.score) {
                best = Some(Pick {
                    score,
                    normal,
                    landing,
                    cloud,
                    parts: [cloud_m, land, kinds.len() as f32],
                });
            }
        }
        let pick = best.unwrap_or(Pick {
            score: 0.0,
            normal: start.cross(east),
            landing: SCENIC_REACH_M,
            cloud: None,
            parts: [0.0; 3],
        });
        let step_m = 25.0;
        let count = (pick.landing / step_m).ceil() as usize + 1;
        let ground: Vec<f32> = (0..count)
            .map(|i| planet::terrain_radius(at(pick.normal, i as f32 * step_m)))
            .collect();
        // Follow the terrain: the highest ground from a little behind to well
        // ahead, plus the flying height, so the ship rises before a ridge; then
        // smoothed, so it does not bob over every block. Through the cloud
        // stretch it holds the cloud's own middle height instead, with a
        // lead-in and lead-out, so it flies into the cloud and out of it.
        let behind = (100.0 / step_m) as usize;
        let ahead = (600.0 / step_m) as usize;
        let raw: Vec<f32> = (0..count)
            .map(|i| {
                let s = i as f32 * step_m;
                let lo = i.saturating_sub(behind);
                let hi = (i + ahead).min(count - 1);
                let high = ground[lo..=hi].iter().copied().fold(0.0f32, f32::max);
                let follow = high + config.route_scenic_height_m;
                match pick.cloud {
                    Some((from, to, top)) if s > from - 700.0 && s < to + 400.0 => {
                        let base = crate::sky::CLOUD_RADIUS;
                        let thickness = crate::sky::CLOUD_THICKNESS * top.clamp(0.2, 1.0);
                        follow.max(base + thickness * 0.4)
                    }
                    _ => follow,
                }
            })
            .collect();
        let smooth = (200.0 / step_m) as usize;
        let radius: Vec<f32> = (0..count)
            .map(|i| {
                let lo = i.saturating_sub(smooth);
                let hi = (i + smooth).min(count - 1);
                let mean = raw[lo..=hi].iter().sum::<f32>() / (hi - lo + 1) as f32;
                mean.max(ground[i] + config.route_scenic_height_m * 0.6)
            })
            .collect();
        match pick.cloud {
            Some((from, to, top)) => info!(
                "scenic route: into {:.0} m of cloud (cover >= {SCENIC_CLOUD_COVER}, top {top:.2}) from {from:.0} m, then {:.0} of land interest over {:.0} kinds of ground, landing on flat ground {:.0} m out; score {:.0}",
                to - from,
                pick.parts[1],
                pick.parts[2],
                pick.landing,
                pick.score
            ),
            None => info!(
                "scenic route: no cloud found on any heading{}; {:.0} of land interest, landing {:.0} m out",
                if atmosphere.is_none() {
                    " (no weather)"
                } else {
                    ""
                },
                pick.parts[1],
                pick.landing
            ),
        }
        Self {
            kind: RouteKind::Scenic,
            planned: true,
            profile: Some(Arc::new(HeightProfile {
                step_m,
                radius,
                ground,
            })),
            start,
            normal: pick.normal,
            destination_rad: pick.landing / PLANET_RADIUS,
            previous_direction: start,
            ..Default::default()
        }
    }

    /// The climb and glide angle, the level speed and the steep-leg speed for
    /// this route.
    fn legs(&self, config: &FlightViewConfig) -> (f32, f32, f32) {
        match self.kind {
            RouteKind::FarSide => (
                config.route_climb_angle_deg,
                config.route_speed,
                config.route_climb_speed,
            ),
            RouteKind::Scenic => (
                config.route_scenic_climb_deg,
                config.route_scenic_speed,
                config.route_scenic_speed * 0.8,
            ),
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
        // Unplanned, the clock does not start: the hold before lift-off
        // counts from when the route is laid out.
        if !self.planned {
            self.previous_direction = direction;
            return;
        }
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
    let (angle, level_speed, steep_speed) = route.legs(config);
    let slope = angle.to_radians().tan();
    let tangential_speed = velocity.dot(tangent);

    let desired = if !route.planned || route.elapsed_s < HOLD_S || route.landed_at_s.is_some() {
        // Standing on the ground, before the lift and after the landing.
        Vec3::ZERO
    } else {
        // The height line and its slope along the ground here, from the
        // numerical derivative of the same line so the two cannot disagree.
        let target = height_line(route, flown, left, slope, config);
        let gradient = (height_line(route, flown + 1.0, left - 1.0, slope, config)
            - height_line(route, flown - 1.0, left + 1.0, slope, config))
            / 2.0;
        // Speed along the path: the climb speed on the steep legs, the cruise
        // speed on the level, eased up from the start and down to the
        // destination at the route's ground acceleration.
        let steep = (gradient.abs() / slope).clamp(0.0, 1.0);
        let path = level_speed + (steep_speed - level_speed) * steep;
        let a = config.route_ground_accel;
        let along = path
            .min((4.0f32.powi(2) + 2.0 * a * flown).sqrt())
            .min((2.0 * a * left).sqrt());
        let mut ground = along / (1.0 + gradient * gradient).sqrt();
        if left < 0.5 {
            ground = 0.0;
        }
        // The ground's own slope along the path, fed forward: the height line
        // is a height ABOVE the ground, so over rising ground the ship has to
        // climb with it. Left to the error term, it sank 50 m and more toward
        // every hillside at 150 m/s (`--verify-route scenic`).
        let along = |metres: f32| {
            planet::terrain_radius(Quat::from_axis_angle(route.normal, metres / PLANET_RADIUS) * up)
        };
        let terrain_slope = (along(25.0) - along(-25.0)) / 50.0;
        let mut vertical = (gradient + terrain_slope) * ground + (target - height) * 0.8;
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
fn height_line(
    route: &RouteState,
    flown: f32,
    left: f32,
    slope: f32,
    config: &FlightViewConfig,
) -> f32 {
    // The lift: the line starts a few metres up so the ship rises at once; the
    // low scenic route lifts 30 m straight up first, clear of the hills round
    // the spawn, before it moves off (it scraped them at 30 degrees).
    let lift = match route.kind {
        RouteKind::FarSide => 6.0 * slope,
        RouteKind::Scenic => 30.0,
    };
    let climb = flown * slope + lift;
    let descend = left.max(0.0) * slope;
    // The level part: the far side's cruise height, or the scenic route's
    // profile, both as a height above the ground.
    let (cruise, k) = match &route.profile {
        Some(profile) => (profile.above_ground(flown), config.route_corner_m * 0.3),
        None => (config.route_cruise_height_m, config.route_corner_m),
    };
    let line = smooth_min(smooth_min(climb, cruise, k), descend, k);
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
    let (angle, _, _) = route.legs(config);
    let slope = angle.to_radians().tan();
    let descent_start = match route.kind {
        RouteKind::FarSide => config.route_cruise_height_m / slope,
        RouteKind::Scenic => 600.0,
    };
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
