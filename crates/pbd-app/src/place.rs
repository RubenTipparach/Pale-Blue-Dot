//! Where the camera is (`position-readout`): the place a screenshot was taken,
//! on the HUD's debug line and in the log, in numbers that take the walker
//! back there.
//!
//! Two latitudes, both printed: `--at` takes the body FRAME's (`geo::lat_lon`,
//! +Y north), so that is the pair to launch with; the map and the compass read
//! the compass's, whose north is the frame's -Y (`geo::compass_lat_lon`). The
//! height above ground is the eye over the generated surface, the number the
//! rain's altitude gate reads; it does not see digs, buildings or cave roofs.

use bevy::prelude::*;
use pbd_core::geo;

use crate::planet::{PLANET_RADIUS, PlanetRenderFrame};
use crate::weather::height_above_ground;

/// A body-local point's place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    /// Degrees, as `--at` takes them.
    pub lat: f32,
    pub lon: f32,
    /// The map's latitude, degrees, north positive.
    pub map_lat: f32,
    /// Metres above sea level.
    pub elevation_m: f32,
    /// Metres above the generated ground under it.
    pub above_ground_m: f32,
}

impl Place {
    /// The place of a point in the body's frame, metres from its centre.
    pub fn of(point: Vec3) -> Self {
        let (lat, lon) = geo::lat_lon(point).degrees();
        let (map_lat, _) = geo::compass_lat_lon(point).degrees();
        Self {
            lat,
            lon,
            map_lat,
            elevation_m: point.length() - PLANET_RADIUS,
            above_ground_m: height_above_ground(point),
        }
    }

    /// One line, for the HUD and the log.
    pub fn line(&self) -> String {
        let ns = if self.map_lat < 0.0 { 'S' } else { 'N' };
        let ew = if self.lon < 0.0 { 'W' } else { 'E' };
        format!(
            "at {:.3} {:.3} (map {:.3}{ns} {:.3}{ew}) | {:.1} m elevation | {:.1} m above ground",
            self.lat,
            self.lon,
            self.map_lat.abs(),
            self.lon.abs(),
            self.elevation_m,
            self.above_ground_m
        )
    }
}

/// The active 3D camera's place, if one is drawing.
pub(crate) fn of_active_camera<'a>(
    frame: &PlanetRenderFrame,
    cameras: impl IntoIterator<Item = (&'a Camera, &'a GlobalTransform)>,
) -> Option<Place> {
    cameras
        .into_iter()
        .find(|(camera, _)| camera.is_active)
        .map(|(_, at)| Place::of((at.translation().as_dvec3() - frame.center).as_vec3()))
}

/// The active camera's place now, from the world (for the `READY` and
/// capture lines).
pub fn of_world(world: &mut World) -> Option<Place> {
    let frame = *world.get_resource::<PlanetRenderFrame>()?;
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    of_active_camera(&frame, cameras.iter(world))
}

/// The active camera's place this frame, for the HUD.
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct CameraPlace(pub Option<Place>);

/// Keep [`CameraPlace`] current, after the transforms are propagated.
fn follow(
    frame: Option<Res<PlanetRenderFrame>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut place: ResMut<CameraPlace>,
) {
    let now = frame.and_then(|frame| of_active_camera(&frame, cameras.iter()));
    place.set_if_neq(CameraPlace(now));
}

/// `PLACE spawn ...`, once, the first frame a 3D camera draws.
fn log_spawn(
    mut done: Local<bool>,
    frame: Option<Res<PlanetRenderFrame>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if *done {
        return;
    }
    let Some(frame) = frame else {
        return;
    };
    if let Some(place) = of_active_camera(&frame, cameras.iter()) {
        info!("PLACE spawn {}", place.line());
        *done = true;
    }
}

/// [`CameraPlace`] kept current, and the `PLACE spawn` line.
pub struct PlacePlugin;

impl Plugin for PlacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraPlace>().add_systems(
            PostUpdate,
            (follow, log_spawn).after(bevy::transform::TransformSystems::Propagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_at_spot_reads_back_and_the_map_reads_it_turned_round() {
        let (lat, lon) = (20.0f32, 40.0f32);
        let direction = geo::direction(geo::LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        });
        let ground = PLANET_RADIUS + crate::planet::surface_height(direction).max(0.0);
        let place = Place::of(direction * (ground + 1.6));
        assert!((place.lat - lat).abs() < 1e-3, "{place:?}");
        assert!((place.lon - lon).abs() < 1e-3, "{place:?}");
        assert!((place.map_lat + lat).abs() < 1e-3, "{place:?}");
        assert!((place.above_ground_m - 1.6).abs() < 0.01, "{place:?}");
        assert!(
            (place.elevation_m - (ground + 1.6 - PLANET_RADIUS)).abs() < 0.01,
            "{place:?}"
        );
        let line = place.line();
        assert!(
            line.starts_with("at 20.000 40.000 (map 20.000S 40.000E)"),
            "{line}"
        );
        assert!(line.ends_with("| 1.6 m above ground"), "{line}");
    }
}
