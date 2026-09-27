//! Latitude and longitude, in one place (`world-map` decision 5).
//!
//! A body's frame has +Y through its north pole. A body-local unit
//! direction's latitude is `asin(y)` and its longitude `atan2(z, x)`, so longitude 0 is
//! +X and longitude 90 degrees east is +Z. The readouts, the map, the site
//! placer and the rasters the mockup was drawn from all go through here, so
//! they agree by construction rather than by four copies of one formula.
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

/// North and east at a direction: unit vectors in the ground's plane, north
/// toward the north pole along the meridian and east along the parallel.
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

/// A heading, radians clockwise from north as a compass reads, of a forward
/// vector at a direction. Forward need not lie in the ground's plane.
pub fn heading(direction: Vec3, forward: Vec3) -> f32 {
    let (north, east) = north_east(direction);
    forward.dot(east).atan2(forward.dot(north))
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
