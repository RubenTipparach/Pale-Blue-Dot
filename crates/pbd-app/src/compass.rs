//! The compass bar's sums (`compass-bar`): which way the view is facing as a
//! compass reads it, and where each letter, tick and place lands on a bar
//! that spans half the horizon. Pure, so a test can turn the player round
//! and look at the marks without a window; the desktop draws them.
//!
//! North is the compass's (`pbd_core::geo::COMPASS_NORTH`), the pole the
//! planet turns counter-clockwise about, so facing N the sun rises on the
//! right, under E.

use bevy::prelude::*;
use pbd_core::geo;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// How far either side of the heading the bar reaches, radians: half the
/// horizon across the whole bar (decision 3).
pub const HALF_SPAN: f32 = FRAC_PI_2;

/// The share of each half, at its outer end, over which marks fade out.
const EDGE: f32 = 0.25;

/// How far a place may be and still show, metres along the ground
/// (decision 5), and how near it is shown at full strength.
pub const PLACE_REACH_M: f32 = 3000.0;
const PLACE_FULL_M: f32 = 1000.0;
/// The faintest a place in reach is drawn.
const PLACE_FAINT: f32 = 0.4;

/// How near the middle a place must be to have its name shown, radians.
pub const NAMED_WITHIN: f32 = 7.0 * PI / 180.0;

/// The bar fades out between these heights above the ground, metres
/// (decision 6): higher up the planet is a globe and is its own compass.
pub const FADE_FROM_M: f32 = 2000.0;
pub const FADE_TO_M: f32 = 3000.0;

/// The 24 marks round the horizon, 15 degrees apart from north: the letters
/// where there is one, a tick where there is not.
pub const DIRECTIONS: [&str; 24] = [
    "N", "", "", "NE", "", "", "E", "", "", "SE", "", "", "S", "", "", "SW", "", "", "W", "", "",
    "NW", "", "",
];

/// What a mark is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    /// N, E, S or W.
    Cardinal,
    /// NE, SE, SW or NW.
    Intercardinal,
    /// A 15-degree step with no letter.
    Tick,
    /// One of the places passed in, by index.
    Place(usize),
}

/// A mark on the bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub kind: MarkKind,
    /// Which direction of [`DIRECTIONS`] it is, or which place.
    pub index: usize,
    /// Its letter, empty for a tick or a place.
    pub label: &'static str,
    /// Where it is along the bar, -1 at the left end to 1 at the right.
    pub offset: f32,
    /// How strongly it is drawn, 0..1.
    pub fade: f32,
}

/// A place the bar can point to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    /// Compass bearing from the player, radians clockwise from north.
    pub bearing: f32,
    /// Metres along the ground.
    pub distance_m: f32,
}

impl Place {
    /// A place at `to` seen from `from`, both body-local, on a body of
    /// `radius` metres.
    pub fn seen(from: Vec3, to: Vec3, radius: f32) -> Self {
        Place {
            bearing: geo::bearing(from, to),
            distance_m: from.angle_between(to) * radius,
        }
    }
}

/// An angle wrapped into `-pi..pi`.
fn wrapped(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

fn smoothstep(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How strongly something at `offset` along the bar is drawn: full in the
/// middle, fading over the outer quarter of each half.
fn edge_fade(offset: f32) -> f32 {
    1.0 - smoothstep(1.0 - EDGE, 1.0, offset.abs())
}

/// Where a bearing lands on the bar for a heading, if it is on it.
fn offset_of(bearing: f32, heading: f32) -> Option<f32> {
    let offset = wrapped(bearing - heading) / HALF_SPAN;
    (offset.abs() <= 1.0).then_some(offset)
}

/// The view's heading, radians clockwise from compass north, at `up` (the
/// body-local direction of the camera) facing `forward` with the camera's own
/// up `camera_up` (decision 4). Forward taken off the vertical, and where it
/// is lost to a steep pitch the camera's up stands in for it, so the heading
/// holds still while the view tips to the feet or the sky. `None` only when
/// the view is degenerate.
pub fn view_heading(up: Vec3, forward: Vec3, camera_up: Vec3) -> Option<f32> {
    let up = up.try_normalize()?;
    let flat = |v: Vec3| v - up * v.dot(up);
    let toward = flat(forward) - flat(camera_up) * forward.dot(up);
    let toward = toward.try_normalize()?;
    Some(geo::compass_heading(up, toward))
}

/// Every mark on the bar for a heading: the directions in reach, then the
/// places in reach, each with where it lands and how strongly it is drawn.
pub fn marks(heading: f32, places: &[Place]) -> Vec<Mark> {
    let mut marks = Vec::new();
    for (i, label) in DIRECTIONS.iter().enumerate() {
        let bearing = i as f32 * TAU / DIRECTIONS.len() as f32;
        let Some(offset) = offset_of(bearing, heading) else {
            continue;
        };
        let kind = if i % 6 == 0 {
            MarkKind::Cardinal
        } else if i % 3 == 0 {
            MarkKind::Intercardinal
        } else {
            MarkKind::Tick
        };
        marks.push(Mark {
            kind,
            index: i,
            label,
            offset,
            fade: edge_fade(offset),
        });
    }
    for (i, place) in places.iter().enumerate() {
        if place.distance_m > PLACE_REACH_M {
            continue;
        }
        let Some(offset) = offset_of(place.bearing, heading) else {
            continue;
        };
        let near = 1.0 - smoothstep(PLACE_FULL_M, PLACE_REACH_M, place.distance_m);
        marks.push(Mark {
            kind: MarkKind::Place(i),
            index: i,
            label: "",
            offset,
            fade: edge_fade(offset) * (PLACE_FAINT + (1.0 - PLACE_FAINT) * near),
        });
    }
    marks
}

/// The place whose name is shown: the one nearest the middle of the bar,
/// within [`NAMED_WITHIN`] of the heading and in reach.
pub fn named(heading: f32, places: &[Place]) -> Option<usize> {
    places
        .iter()
        .enumerate()
        .filter(|(_, p)| p.distance_m <= PLACE_REACH_M)
        .map(|(i, p)| (i, wrapped(p.bearing - heading).abs()))
        .filter(|(_, off)| *off <= NAMED_WITHIN)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// What the bar reads off the view each frame: where the active camera is
/// and which way it faces, as a compass reads them.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct CompassView {
    /// The body-local unit direction of the camera; zero before there is one.
    pub up: Vec3,
    /// Radians clockwise from compass north; `None` before there is a view.
    pub heading: Option<f32>,
    /// Metres above the ground under the camera.
    pub height_m: f32,
}

/// Read the active camera into [`CompassView`]. Walking keeps the flight
/// camera parked and inactive beside its own, so the active one is asked for.
fn read_view(
    frame: Res<crate::planet::PlanetRenderFrame>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut view: ResMut<CompassView>,
) {
    let Some(camera) = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform)
    else {
        return;
    };
    let body_local = (camera.translation().as_dvec3() - frame.center).as_vec3();
    let Some(up) = body_local.try_normalize() else {
        return;
    };
    *view = CompassView {
        up,
        heading: view_heading(up, camera.forward().as_vec3(), camera.up().as_vec3()),
        height_m: crate::weather::height_above_ground(body_local),
    };
}

/// The compass's view, read every frame for the bar to draw.
pub struct CompassPlugin;

impl Plugin for CompassPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompassView>()
            .add_systems(Update, read_view);
    }
}

/// How strongly the bar is drawn at a height above the ground, metres.
pub fn altitude_fade(height_m: f32) -> f32 {
    1.0 - smoothstep(FADE_FROM_M, FADE_TO_M, height_m)
}

/// A distance as the name line says it: metres near, kilometres far.
pub fn distance_text(metres: f32) -> String {
    if metres < 1000.0 {
        format!("{:.0} m", (metres / 10.0).round() * 10.0)
    } else {
        format!("{:.1} km", metres / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::geo::{LatLon, compass_north_east, direction};

    fn place(lat: f32, lon: f32) -> Vec3 {
        direction(LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        })
    }

    /// The letter at the middle of the bar for a heading.
    fn middle(heading: f32) -> &'static str {
        marks(heading, &[])
            .into_iter()
            .filter(|m| !m.label.is_empty())
            .min_by(|a, b| a.offset.abs().total_cmp(&b.offset.abs()))
            .map(|m| m.label)
            .unwrap_or("")
    }

    /// Turning right from north through a whole turn passes N, NE, E, SE, S,
    /// SW, W, NW and N in that order across the middle of the bar (the spec's
    /// "Turning right"), with the view turned by the camera's own right.
    #[test]
    fn turning_right_reads_the_compass_round() {
        let up = place(23.0, -40.0);
        let (north, _) = compass_north_east(up);
        let mut seen = Vec::new();
        for step in 0..=8 {
            // Turning right is turning clockwise seen from above: about -up.
            let turn = Quat::from_axis_angle(up, -(step as f32) * TAU / 8.0);
            let forward = turn * north;
            let heading = view_heading(up, forward, up).expect("a view");
            seen.push(middle(heading));
        }
        assert_eq!(seen, ["N", "NE", "E", "SE", "S", "SW", "W", "NW", "N"]);
    }

    /// Pitching from level to straight down or up without turning keeps the
    /// heading (the spec's "Looking at their feet").
    #[test]
    fn pitching_to_the_feet_keeps_the_heading() {
        let up = place(-12.0, 130.0);
        let (north, east) = compass_north_east(up);
        let facing = (north + east * 0.4).normalize();
        let level = view_heading(up, facing, up).expect("a view");
        let right = facing.cross(up);
        for degrees in [-89.9f32, -60.0, -20.0, 20.0, 60.0, 89.9] {
            let pitch = Quat::from_axis_angle(right, degrees.to_radians());
            let (forward, camera_up) = (pitch * facing, pitch * up);
            let heading = view_heading(up, forward, camera_up).expect("a view");
            assert!(
                wrapped(heading - level).abs() < 1e-3,
                "pitched {degrees} degrees: {heading} against {level}"
            );
        }
    }

    /// A town a little compass-east of the player, 1.2 km away, lands at its
    /// bearing: in the middle when the player faces it, a third of the way to
    /// the right end when they face 30 degrees left of it, and named with
    /// its distance when it is near the middle.
    #[test]
    fn a_town_lands_at_its_bearing_and_is_named_near_the_middle() {
        let radius = 4800.0f32;
        let here = place(5.0, 5.0);
        let (_, east) = compass_north_east(here);
        let angle = 1200.0 / radius;
        let town = (here * angle.cos() + east * angle.sin()).normalize();
        let seen = Place::seen(here, town, radius);
        assert!((seen.bearing - FRAC_PI_2).abs() < 1e-3);
        assert!((seen.distance_m - 1200.0).abs() < 0.5);
        let at = |heading: f32| {
            marks(heading, &[seen])
                .into_iter()
                .find(|m| m.kind == MarkKind::Place(0))
        };
        assert!(at(FRAC_PI_2).expect("on the bar").offset.abs() < 1e-3);
        let left = at(FRAC_PI_2 - 30f32.to_radians()).expect("on the bar");
        assert!((left.offset - 1.0 / 3.0).abs() < 1e-3, "{}", left.offset);
        assert!(
            at(FRAC_PI_2 + PI).is_none(),
            "behind the player it is off the bar"
        );
        assert_eq!(named(FRAC_PI_2 + 0.05, &[seen]), Some(0));
        assert_eq!(
            named(FRAC_PI_2 + 0.2, &[seen]),
            None,
            "too far from the middle"
        );
        assert_eq!(distance_text(seen.distance_m), "1.2 km");
        assert_eq!(distance_text(437.0), "440 m");
        let far = Place {
            distance_m: PLACE_REACH_M + 1.0,
            ..seen
        };
        assert!(
            marks(FRAC_PI_2, &[far])
                .iter()
                .all(|m| m.kind != MarkKind::Place(0))
        );
    }

    /// Marks fade toward the ends of the bar, and the bar fades out between
    /// two and three kilometres up.
    #[test]
    fn the_ends_and_the_heights_fade() {
        let all = marks(0.0, &[]);
        let north = all.iter().find(|m| m.label == "N").expect("N");
        assert_eq!((north.offset, north.fade), (0.0, 1.0));
        let east = all.iter().find(|m| m.label == "E").expect("E at the end");
        assert!((east.offset - 1.0).abs() < 1e-5 && east.fade < 1e-3);
        assert!(all.iter().all(|m| m.label != "S"), "S is behind");
        assert_eq!(altitude_fade(500.0), 1.0);
        assert_eq!(altitude_fade(3500.0), 0.0);
        assert!((altitude_fade(2500.0) - 0.5).abs() < 1e-3);
    }
}
