//! Latitude and longitude, in one place (`world-map` decision 5).
//!
//! A body's FRAME has +Y through the pole this module's frame functions call
//! north. A body-local unit direction's frame latitude is `asin(y)` and its
//! longitude `atan2(z, x)`, so longitude 0 is +X and longitude 90 degrees
//! east is +Z. The map's rasters, the site placer and the towns' layouts all
//! go through here, so they agree by construction rather than by four copies
//! of one formula.
//!
//! That frame north is not the COMPASS's (`compass-bar`). The planet turns
//! about -Y: the sun rises toward +Z at +X and the atmosphere's prograde wind
//! is `c x Y`. By the rule every compass on Earth follows, the north pole is
//! the one the planet turns counter-clockwise about, so a compass points to
//! -Y, and facing it, east (the same +Z) is on the right and the sun rises
//! there. In frame terms, east is on the LEFT of a player facing +Y. What a
//! player reads - a heading, a latitude, a bearing, the map's way up - goes
//! through the `compass_*` functions; the frame functions keep their meaning
//! because layouts and saved records were built on them.
//!
//! The map is equirectangular at every zoom (the owner, survey M2): `u` runs
//! from 0 at longitude -180 degrees to 1 at +180, and `v` from 0 at the north
//! pole to 1 at the south, as a raster's rows run. A pixel `(col, row)` of a
//! `width x height` raster is centred on `((col + 0.5) / width, (row + 0.5) /
//! height)`.

use glam::{DVec3, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// A place on a body, in radians: latitude from -pi/2 (south pole) to pi/2
/// (north pole), longitude from -pi to pi.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatLon {
    pub lat: f32,
    pub lon: f32,
}

impl LatLon {
    /// Latitude and longitude in degrees, for readouts.
    pub fn degrees(self) -> (f32, f32) {
        (self.lat.to_degrees(), self.lon.to_degrees())
    }
}

/// The latitude and longitude of a body-local direction. It need not be
/// unit length; a zero direction reads as the north pole. Latitude is taken
/// as `atan2` of the height over the distance from the axis, which is
/// `asin(y)` for a unit direction but keeps its precision near the poles,
/// where `asin` of an `f32` loses the last hundredth of a degree.
pub fn lat_lon(direction: Vec3) -> LatLon {
    let d = direction.normalize_or(Vec3::Y);
    LatLon {
        lat: d.y.atan2((d.x * d.x + d.z * d.z).sqrt()),
        lon: d.z.atan2(d.x),
    }
}

/// The latitude and longitude of a system position on a body centred at
/// `centre`. The centre is subtracted in `f64` before anything is cast
/// (CLAUDE.md), so a body far from the origin reads the same as one at it.
pub fn lat_lon_of(position: DVec3, centre: DVec3) -> LatLon {
    lat_lon((position - centre).as_vec3())
}

/// The unit body-local direction at a latitude and longitude.
pub fn direction(at: LatLon) -> Vec3 {
    let (sin_lat, cos_lat) = at.lat.sin_cos();
    let (sin_lon, cos_lon) = at.lon.sin_cos();
    Vec3::new(cos_lat * cos_lon, sin_lat, cos_lat * sin_lon)
}

/// The FRAME's north and east at a direction: unit vectors in the ground's
/// plane, north toward +Y along the meridian and east along the parallel.
/// Layouts orient by these; a player reads [`compass_north_east`].
/// At a pole, where both are undefined, they are taken along longitude 0,
/// so they stay continuous along that meridian.
pub fn north_east(direction: Vec3) -> (Vec3, Vec3) {
    let d = direction.normalize_or(Vec3::Y);
    let across = (d.x * d.x + d.z * d.z).sqrt();
    let east = if across > 1e-6 {
        Vec3::new(-d.z / across, 0.0, d.x / across)
    } else {
        Vec3::Z
    };
    (east.cross(d).normalize(), east)
}

/// A heading in the FRAME's terms, radians from +Y's meridian toward frame
/// east. Facing frame north this runs the wrong way round for a compass;
/// a player reads [`compass_heading`].
pub fn heading(direction: Vec3, forward: Vec3) -> f32 {
    let (north, east) = north_east(direction);
    forward.dot(east).atan2(forward.dot(north))
}

/// The pole a compass points to: the one the planet turns counter-clockwise
/// about, which the sun's path and the atmosphere's winds agree on.
pub const COMPASS_NORTH: Vec3 = Vec3::NEG_Y;

/// North and east as a compass reads them at a direction: unit vectors in
/// the ground's plane, north toward [`COMPASS_NORTH`] and east on its right,
/// which is the side the sun rises on. Compass east is frame east; compass
/// north is frame north turned round.
pub fn compass_north_east(direction: Vec3) -> (Vec3, Vec3) {
    let (north, east) = north_east(direction);
    (-north, east)
}

/// A heading, radians clockwise from compass north as a compass reads it, of
/// a forward vector at a direction, in `-pi..=pi`. Forward need not lie in
/// the ground's plane.
pub fn compass_heading(direction: Vec3, forward: Vec3) -> f32 {
    let (north, east) = compass_north_east(direction);
    forward.dot(east).atan2(forward.dot(north))
}

/// The compass bearing from one place to another, radians clockwise from
/// compass north: the heading the great circle between them sets off on.
/// Zero when the two are the same place or opposite.
pub fn bearing(from: Vec3, to: Vec3) -> f32 {
    let from = from.normalize_or(Vec3::Y);
    let to = to.normalize_or(Vec3::Y);
    let toward = to - from * from.dot(to);
    if toward.length_squared() < 1e-12 {
        return 0.0;
    }
    compass_heading(from, toward)
}

/// Latitude and longitude as a compass reads them: north positive, toward
/// [`COMPASS_NORTH`]. The frame latitude negated; longitude is the same,
/// because compass east is frame east.
pub fn compass_lat_lon(direction: Vec3) -> LatLon {
    let at = lat_lon(direction);
    LatLon {
        lat: -at.lat,
        lon: at.lon,
    }
}

/// The equirectangular map position of a direction, `(u, v)` in `0..=1`.
pub fn project(direction: Vec3) -> Vec2 {
    let at = lat_lon(direction);
    Vec2::new(at.lon / TAU + 0.5, 0.5 - at.lat / PI)
}

/// The direction at an equirectangular map position. `u` wraps, since the
/// map is a whole number of turns round; `v` is clamped to the poles.
pub fn unproject(uv: Vec2) -> Vec3 {
    let u = uv.x.rem_euclid(1.0);
    let v = uv.y.clamp(0.0, 1.0);
    direction(LatLon {
        lat: (0.5 - v) * PI,
        lon: (u - 0.5) * TAU,
    })
}

/// The direction through the centre of pixel `(col, row)` of a `width x
/// height` equirectangular raster.
pub fn pixel_direction(col: usize, row: usize, width: usize, height: usize) -> Vec3 {
    unproject(Vec2::new(
        (col as f32 + 0.5) / width as f32,
        (row as f32 + 0.5) / height as f32,
    ))
}

/// Latitude, clamped to the poles.
pub fn clamp_lat(lat: f32) -> f32 {
    lat.clamp(-FRAC_PI_2, FRAC_PI_2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        a.distance(b) < 1e-5
    }

    /// Latitude and longitude go round and back at the equator, the poles and
    /// on either side of the antimeridian, and so does the map projection.
    #[test]
    fn a_direction_goes_round_and_back() {
        for (lat, lon) in [
            (0.0f32, 0.0f32),
            (0.0, 90.0),
            (0.0, -90.0),
            (45.0, 179.9),
            (-45.0, -179.9),
            (10.0, 180.0),
            (89.99, 30.0),
            (-89.99, -60.0),
            (-33.0, 151.0),
        ] {
            let at = LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            };
            let d = direction(at);
            assert!((d.length() - 1.0).abs() < 1e-6);
            let back = direction(lat_lon(d));
            assert!(close(back, d), "({lat}, {lon}) came back as {back}");
            assert!(
                close(unproject(project(d)), d),
                "the map round trip at ({lat}, {lon})"
            );
        }
        // The frame's own axes.
        assert_eq!(lat_lon(Vec3::X).degrees(), (0.0, 0.0));
        let (lat, lon) = lat_lon(Vec3::Z).degrees();
        assert!(lat.abs() < 1e-5 && (lon - 90.0).abs() < 1e-4);
        assert!((lat_lon(Vec3::Y).lat - FRAC_PI_2).abs() < 1e-6);
        assert!((lat_lon(-Vec3::Y).lat + FRAC_PI_2).abs() < 1e-6);
    }

    /// The map's edges: the antimeridian is both ends of a row, the poles are
    /// the top and bottom rows, and the prime meridian at the equator is the
    /// middle. `u` wraps and `v` clamps.
    #[test]
    fn the_map_edges_are_the_antimeridian_and_the_poles() {
        let middle = project(Vec3::X);
        assert!(middle.distance(Vec2::new(0.5, 0.5)) < 1e-6);
        assert!(project(Vec3::Y).y.abs() < 1e-6, "north is the top");
        assert!(
            (project(-Vec3::Y).y - 1.0).abs() < 1e-6,
            "south is the bottom"
        );
        let west = unproject(Vec2::new(0.0, 0.5));
        let east = unproject(Vec2::new(1.0, 0.5));
        assert!(
            close(west, east) && close(west, -Vec3::X),
            "one meridian, both ends"
        );
        assert!(close(
            unproject(Vec2::new(1.25, 0.5)),
            unproject(Vec2::new(0.25, 0.5))
        ));
        assert!(close(unproject(Vec2::new(0.3, -1.0)), Vec3::Y));
        // The first pixel of a 4x2 raster is in the north-west quarter.
        let p = pixel_direction(0, 0, 4, 2);
        let at = lat_lon(p).degrees();
        assert!(
            (at.0 - 45.0).abs() < 1e-4 && (at.1 + 135.0).abs() < 1e-3,
            "{at:?}"
        );
    }

    /// North and east are unit, in the ground's plane, at right angles, and
    /// the right way round: north climbs in latitude and east in longitude,
    /// at the equator, near the poles and at the poles themselves.
    #[test]
    fn north_and_east_point_the_right_ways() {
        for (lat, lon) in [
            (0.0f32, 0.0f32),
            (30.0, 100.0),
            (-60.0, -170.0),
            (89.0, 10.0),
        ] {
            let at = LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            };
            let d = direction(at);
            let (north, east) = north_east(d);
            assert!((north.length() - 1.0).abs() < 1e-5 && (east.length() - 1.0).abs() < 1e-5);
            assert!(north.dot(d).abs() < 1e-5 && east.dot(d).abs() < 1e-5);
            assert!(north.dot(east).abs() < 1e-5);
            let step = 1e-3;
            let up = lat_lon(d + north * step);
            let right = lat_lon(d + east * step);
            assert!(up.lat > at.lat, "north climbs at ({lat}, {lon})");
            let dlon = (right.lon - at.lon + PI).rem_euclid(TAU) - PI;
            assert!(dlon > 0.0, "east goes east at ({lat}, {lon})");
            assert!(heading(d, north).abs() < 1e-5);
            assert!((heading(d, east) - FRAC_PI_2).abs() < 1e-5);
        }
        for pole in [Vec3::Y, -Vec3::Y] {
            let (north, east) = north_east(pole);
            assert!(north.is_finite() && east.is_finite());
            assert!(north.dot(pole).abs() < 1e-6 && east.dot(pole).abs() < 1e-6);
        }
    }

    /// The compass is the right way round everywhere (`compass-bar`): facing
    /// compass north, east is on the screen's right (Bevy's right is forward
    /// x up); from space with compass north up the screen, east is on the
    /// right too; and wherever the sun rises, it rises toward compass east and
    /// then crosses toward west. The frame's north fails all three, which is
    /// what the change measured.
    #[test]
    fn the_compass_has_east_on_the_right_and_the_sun_rising_there() {
        use crate::daylight::{Clock, DAY_S};
        for (lat, lon) in [
            (0.0f32, 0.0f32),
            (30.0, 45.0),
            (-40.0, -120.0),
            (60.0, 170.0),
            // Day 0 is the +Y hemisphere's summer, so far toward -Y is a
            // polar night with no sunrise to test.
            (-50.0, 10.0),
        ] {
            let d = direction(LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            });
            let (north, east) = compass_north_east(d);
            assert!(north.dot(d).abs() < 1e-5 && east.dot(d).abs() < 1e-5);
            let right_facing_north = north.cross(d);
            assert!(
                right_facing_north.dot(east) > 0.999,
                "facing north at ({lat}, {lon}), east is on the right"
            );
            let right_from_space = (-d).cross(north);
            assert!(
                right_from_space.dot(east) > 0.999,
                "seen from space at ({lat}, {lon}), east is on the right"
            );
            let (frame_north, _) = north_east(d);
            assert!(
                frame_north.cross(d).dot(east) < -0.999,
                "the frame's north has east on the left"
            );
            assert!(compass_heading(d, north).abs() < 1e-5);
            assert!((compass_heading(d, east) - FRAC_PI_2).abs() < 1e-5);
            assert!((compass_heading(d, right_facing_north) - FRAC_PI_2).abs() < 1e-4);
            // Sunrise here: the first upward crossing of the horizon in a day.
            let mut last = Clock { seconds: 0.0 }.elevation(d);
            let mut rose = None;
            for second in 1..=DAY_S as u32 {
                let clock = Clock {
                    seconds: f64::from(second),
                };
                let e = clock.elevation(d);
                if last < 0.0 && e >= 0.0 {
                    rose = Some(clock);
                    break;
                }
                last = e;
            }
            let rose = rose.expect("the sun rises everywhere but the poles");
            let sun = rose.sun();
            let flat = (sun - d * sun.dot(d)).normalize();
            assert!(
                flat.dot(east) > 0.5,
                "at ({lat}, {lon}) the sun rises toward {flat:?}, east is {east:?}"
            );
            // At local noon the sun is crossing the meridian, westward.
            let noon = Clock::at(0, (12.0 - lon / 15.0).rem_euclid(24.0));
            let later = Clock {
                seconds: noon.seconds + 60.0,
            };
            assert!(
                (later.sun() - noon.sun()).dot(east) < 0.0,
                "at ({lat}, {lon}) the sun crosses toward west"
            );
        }
    }

    /// Compass latitude is the frame's negated and longitude the same; a
    /// place a little compass-east bears 90 degrees and a little north 0.
    #[test]
    fn compass_latitude_and_bearing() {
        let d = direction(LatLon {
            lat: 20f32.to_radians(),
            lon: -35f32.to_radians(),
        });
        let (lat, lon) = compass_lat_lon(d).degrees();
        assert!((lat + 20.0).abs() < 1e-4 && (lon + 35.0).abs() < 1e-4);
        assert!((compass_lat_lon(COMPASS_NORTH).lat - FRAC_PI_2).abs() < 1e-5);
        let (north, east) = compass_north_east(d);
        assert!((bearing(d, d + east * 1e-3) - FRAC_PI_2).abs() < 1e-3);
        assert!(bearing(d, d + north * 1e-3).abs() < 1e-3);
        assert!((bearing(d, d - north * 1e-3).abs() - PI).abs() < 1e-3);
        assert!(
            compass_lat_lon(d + north * 1e-3).lat > compass_lat_lon(d).lat,
            "compass north climbs in compass latitude"
        );
        assert_eq!(bearing(d, d), 0.0);
    }

    /// A body far from the system origin reads its places as one at the
    /// origin does (CLAUDE.md: test an offset planet too).
    #[test]
    fn an_offset_planet_reads_the_same_places() {
        let centre = DVec3::new(8.0e9, -3.5e9, 1.2e10);
        let radius = 4800.0f64;
        for (lat, lon) in [(12.5f32, -71.25f32), (-80.0, 179.5), (0.0, 0.0)] {
            let at = LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            };
            let local = direction(at).as_dvec3() * radius;
            let here = lat_lon_of(local, DVec3::ZERO);
            let there = lat_lon_of(centre + local, centre);
            assert!((here.lat - there.lat).abs() < 1e-5, "{here:?} vs {there:?}");
            assert!((here.lon - there.lon).abs() < 1e-5, "{here:?} vs {there:?}");
            assert!((here.lat - at.lat).abs() < 1e-5 && (here.lon - at.lon).abs() < 1e-5);
        }
    }
}
