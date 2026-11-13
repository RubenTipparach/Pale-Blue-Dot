//! The coast walk (`coast-walk`): a recording instrument's steering. The
//! walker keeps to the shore with the sea on one side, crosses a river where
//! it meets one, and keeps off cliffs, by looking along each candidate heading
//! at the generator's height field (`surface_height`). It reads no rendered
//! state, so the same steering can be simulated over the same height field to
//! prove a route before it is filmed.

use bevy::math::{Quat, Vec3};

use crate::planet::{PLANET_RADIUS, river_channel, surface_height, terrain_config};

/// How far along each heading the ground is read, m.
const AHEAD_M: [f32; 4] = [4.0, 8.0, 14.0, 22.0];
/// A stretch that rises or falls more than this between two reads is a cliff.
const CLIFF_M: f32 = 3.0;
/// Water ahead is a river if land comes back within this much further on.
const RIVER_REACH_M: f32 = 30.0;
/// The shore's band: the ground 8 m ahead is best this high over the sea.
const SHORE_M: f32 = 2.5;
/// Where the sea is looked for to the side, m from the point 8 m ahead.
const SIDE_M: f32 = 15.0;
/// What a turn costs against the shore's score, per radian.
const TURN_COST: f32 = 1.5;
/// What the sea on its side is worth.
const SEA_SIDE: f32 = 3.0;
/// High ground costs this much per metre over 12 m.
const HIGH_COST: f32 = 0.5;
/// The candidate headings: this many 15 degree steps either way.
const STEPS: i32 = 5;

/// A point `metres` along the ground from the unit direction `here` toward
/// the tangent `heading`.
pub fn along(here: Vec3, heading: Vec3, metres: f32) -> Vec3 {
    let a = metres / PLANET_RADIUS;
    (here * a.cos() + heading * a.sin()).normalize()
}

/// Whether a direction is in a river's channel: the generator's own carve
/// test, which tells a watercourse from a shallow bay as no height can.
pub fn in_river(direction: Vec3) -> bool {
    let cfg = terrain_config();
    river_channel(cfg, direction) > cfg.river_threshold
}

/// What a walker stands on at a direction: the ground, or the water over it.
fn walked_on(direction: Vec3) -> f32 {
    surface_height(direction).max(0.0)
}

/// The side of a heading the sea is kept on: `side` +1 is the left.
fn side_of(up: Vec3, heading: Vec3, side: f32) -> Vec3 {
    up.cross(heading).normalize_or_zero() * side
}

/// A heading to walk, and whether it crosses water.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Choice {
    pub heading: Vec3,
    pub river: bool,
}

/// The heading to walk from `here` (a direction from the planet's centre)
/// with the walker facing `forward`, keeping the sea on `side` (+1 left, -1
/// right): the shore's band 8 m out, little turning, the sea to the side,
/// no cliff, and water only where land comes back within `RIVER_REACH_M`.
/// None when every heading is ruled out.
pub fn steer(here: Vec3, forward: Vec3, side: f32) -> Option<Choice> {
    let up = here.normalize_or_zero();
    let ahead = (forward - up * forward.dot(up)).normalize_or_zero();
    if up == Vec3::ZERO || ahead == Vec3::ZERO {
        return None;
    }
    let mut best: Option<(f32, Choice)> = None;
    for k in -STEPS..=STEPS {
        let turn = (k as f32 * 15.0).to_radians();
        let heading = Quat::from_axis_angle(up, turn) * ahead;
        let mut last = walked_on(up);
        let mut cliff = false;
        let mut water = false;
        for d in AHEAD_M {
            let point = along(up, heading, d);
            water |= surface_height(point) < 0.0;
            let h = walked_on(point);
            cliff |= (h - last).abs() > CLIFF_M;
            last = h;
        }
        if cliff {
            continue;
        }
        let river = water;
        if water {
            let back = (1..=6).any(|i| {
                let d = AHEAD_M[3] + RIVER_REACH_M * i as f32 / 6.0;
                surface_height(along(up, heading, d)) > 0.3
            });
            if !back {
                continue;
            }
        }
        let out = along(up, heading, AHEAD_M[1]);
        let h8 = walked_on(out);
        let sea_side = surface_height(along(out, side_of(out, heading, side), SIDE_M)) < 0.0;
        let score =
            (h8 - SHORE_M).abs() + TURN_COST * turn.abs() + HIGH_COST * (h8 - 12.0).max(0.0)
                - if sea_side { SEA_SIDE } else { 0.0 };
        if best.as_ref().is_none_or(|(s, _)| score < *s) {
            best = Some((score, Choice { heading, river }));
        }
    }
    best.map(|(_, choice)| choice)
}

/// A start on the shore at `direction`: ground in the band with the sea
/// within 15 m to one side, and a heading along the shore with the sea on
/// that `side` (+1 left). None off the shore.
pub fn shore_start(direction: Vec3) -> Option<(Vec3, f32)> {
    let up = direction.normalize_or_zero();
    let h = surface_height(up);
    if !(0.3..=4.0).contains(&h) {
        return None;
    }
    let (north, east) = pbd_core::geo::north_east(up);
    for k in 0..16 {
        let a = k as f32 * std::f32::consts::TAU / 16.0;
        let heading = north * a.cos() + east * a.sin();
        for side in [1.0, -1.0] {
            let sea = surface_height(along(up, side_of(up, heading, side), 15.0)) < 0.0;
            let ahead = surface_height(along(up, heading, 15.0));
            if sea && (0.3..=6.0).contains(&ahead) {
                return Some((heading, side));
            }
        }
    }
    None
}

/// A walk of the same steering in `step_m` strides over the height field,
/// for proving a route: the heading turns toward the choice by at most
/// `turn_per_step` radians a stride.
#[derive(Clone, Debug, Default)]
pub struct Simulated {
    /// The largest rise or fall of what is walked on between two strides, m.
    pub worst_step_m: f32,
    /// Strides with the sea within 40 m to the side.
    pub by_the_sea: usize,
    pub strides: usize,
    /// Rivers crossed: water left again within the river's reach, some of
    /// it in a river's channel.
    pub rivers: usize,
    /// Other water waded and left again (a bay, the sea's edge).
    pub wades: usize,
    /// Strides where every heading was ruled out.
    pub stuck: usize,
    pub end: Vec3,
}

pub fn simulate(
    start: Vec3,
    forward: Vec3,
    side: f32,
    metres: f32,
    step_m: f32,
    turn_per_step: f32,
) -> Simulated {
    let mut here = start.normalize();
    let mut heading = forward;
    let mut out = Simulated::default();
    let mut in_water_m: Option<(f32, bool)> = None;
    let strides = (metres / step_m).ceil() as usize;
    for _ in 0..strides {
        let up = here;
        heading = (heading - up * heading.dot(up)).normalize_or(heading);
        match steer(here, heading, side) {
            Some(choice) => {
                let angle = heading.angle_between(choice.heading);
                let sign = up.dot(heading.cross(choice.heading)).signum();
                let turn = angle.min(turn_per_step) * sign;
                heading = Quat::from_axis_angle(up, turn) * heading;
            }
            None => {
                out.stuck += 1;
                heading = Quat::from_axis_angle(up, std::f32::consts::FRAC_PI_2 * -side) * heading;
            }
        }
        let next = along(here, heading, step_m);
        out.worst_step_m = out
            .worst_step_m
            .max((walked_on(next) - walked_on(here)).abs());
        let wet = surface_height(next) < 0.0;
        let river = wet && in_river(next);
        in_water_m = match (in_water_m, wet) {
            (None, true) => Some((step_m, river)),
            (Some((m, r)), true) => Some((m + step_m, r || river)),
            (Some((m, r)), false) => {
                if m <= RIVER_REACH_M {
                    if r {
                        out.rivers += 1;
                    } else {
                        out.wades += 1;
                    }
                }
                None
            }
            (None, false) => None,
        };
        let side_dir = side_of(next, heading, side);
        if (1..=4).any(|i| surface_height(along(next, side_dir, 10.0 * i as f32)) < 0.0) {
            out.by_the_sea += 1;
        }
        out.strides += 1;
        here = next;
    }
    out.end = here;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::geo;

    fn at(lat: f32, lon: f32) -> Vec3 {
        geo::direction(geo::LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        })
    }

    /// The start the proof video walks from (`--at 30 177.5`, map 30.000S
    /// 177.500E): 2 km of the same steering over the height field, at the
    /// script's 8 m/s and 50 degrees a second, keeps off cliffs, keeps the
    /// sea beside it and crosses rivers.
    #[test]
    fn the_video_s_coast_walk_is_sound() {
        let start = at(30.0, 177.5);
        let (heading, side) = shore_start(start).expect("the start is on the shore");
        let s = simulate(start, heading, side, 2000.0, 2.0, 12.5f32.to_radians());
        assert_eq!(s.stuck, 0, "{s:?}");
        assert!(s.worst_step_m <= 2.5, "{s:?}");
        assert!(s.rivers >= 1, "{s:?}");
        assert!(s.by_the_sea * 10 >= s.strides * 8, "{s:?}");
    }

    /// Exploratory: simulate 2 km from coastal starts on a grid and print the
    /// ones with rivers and no cliff. `cargo test -- --ignored find_a_coast`.
    #[test]
    #[ignore]
    fn find_a_coast_with_rivers() {
        let mut found = 0;
        for lat10 in (-500..=500).step_by(25) {
            for lon10 in (-1800..1800).step_by(25) {
                let (lat, lon) = (lat10 as f32 / 10.0, lon10 as f32 / 10.0);
                let start = at(lat, lon);
                let Some((heading, side)) = shore_start(start) else {
                    continue;
                };
                let s = simulate(start, heading, side, 2000.0, 2.0, 12.5f32.to_radians());
                if s.rivers >= 1 && s.worst_step_m <= 2.5 && s.stuck == 0 {
                    println!(
                        "--at {lat} {lon}: side {side}, rivers {}, wades {}, by the sea {}/{}, worst step {:.2} m",
                        s.rivers, s.wades, s.by_the_sea, s.strides, s.worst_step_m
                    );
                    found += 1;
                }
            }
        }
        println!("{found} starts");
    }
}
