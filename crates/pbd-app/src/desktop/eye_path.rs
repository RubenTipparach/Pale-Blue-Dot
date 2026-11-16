//! MEASUREMENT (`ocean-environments` B17, the owner's waterline videos): a
//! capture-only eye path. `PBD_EYE_PATH=<file>` moves the flying camera along
//! keyframes, one a line: `frame lat lon altitude yaw pitch` (the capture's
//! frame, as `PBD_CAPTURE_RUN`'s first frame counts it, degrees, metres over the sea's level), lerped
//! between keys, the look fixed by the key's yaw and pitch. Each frame it
//! writes the eye's height against the water surface over or under it into a
//! corner (and an `EYE_PATH` log line), so a frame sequence can be stepped and
//! every artifact read against its height. Nothing of it exists without the
//! variable: no text, no system doing anything, never in the game.

use bevy::prelude::*;
use pbd_app::planet::{EyeWater, EyeWaterState, PLANET_RADIUS, PlanetContact, PlanetFine};

/// One keyframe: the frame it is reached at, where, and the look.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Key {
    frame: f32,
    lat: f32,
    lon: f32,
    alt: f32,
    yaw: f32,
    pitch: f32,
}

/// The parsed path, or `None` without the variable.
#[derive(Resource, Default)]
pub struct EyePath(Option<Vec<Key>>);

/// The corner label.
#[derive(Component)]
pub struct EyePathLabel;

fn parse(text: &str) -> Vec<Key> {
    text.lines()
        .filter_map(|line| {
            let line = line.split('#').next()?.trim();
            let v: Vec<f32> = line
                .split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect();
            (v.len() == 6).then(|| Key {
                frame: v[0],
                lat: v[1],
                lon: v[2],
                alt: v[3],
                yaw: v[4],
                pitch: v[5],
            })
        })
        .collect()
}

/// The key at a frame: lerped between the two round it, held past the ends.
fn at(keys: &[Key], frame: f32) -> Option<Key> {
    let first = *keys.first()?;
    if frame <= first.frame {
        return Some(first);
    }
    for pair in keys.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if frame <= b.frame {
            let t = ((frame - a.frame) / (b.frame - a.frame).max(1e-6)).clamp(0.0, 1.0);
            let l = |x: f32, y: f32| x + (y - x) * t;
            return Some(Key {
                frame,
                lat: l(a.lat, b.lat),
                lon: l(a.lon, b.lon),
                alt: l(a.alt, b.alt),
                yaw: l(a.yaw, b.yaw),
                pitch: l(a.pitch, b.pitch),
            });
        }
    }
    keys.last().copied()
}

/// Read the variable and, with a path, put the label up.
pub fn setup(mut commands: Commands) {
    let keys = std::env::var("PBD_EYE_PATH")
        .ok()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .map(|text| parse(&text))
        .filter(|keys| !keys.is_empty());
    if keys.is_some() {
        commands.spawn((
            Text::new(""),
            TextFont {
                font_size: 26.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.2)),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                bottom: Val::Px(70.0),
                ..default()
            },
            GlobalZIndex(1000),
            EyePathLabel,
        ));
    }
    commands.insert_resource(EyePath(keys));
}

/// The water surface over or under the eye, metres over the sea's level, as
/// the column has it; the sea's own water at the drawn, tided sheet.
fn surface_near(
    contact: &PlanetContact,
    fine: Option<&PlanetFine>,
    direction: Vec3,
    alt: f32,
    sheet: f32,
) -> Option<f32> {
    let sea = pbd_app::planet::terrain_config().sea_level_m;
    let mut a = alt + 4.0;
    while a > alt - 40.0 {
        if let Some(Some(top)) = contact.column_water(direction * (PLANET_RADIUS + a)) {
            if (top - sea).abs() < 0.01 {
                return Some(sheet);
            }
            // The drawn top, as the eye's own decision has it.
            let drawn = fine.and_then(|fine| {
                let record = contact.finest_cell(direction)?;
                let slot = fine.set.columns.slot(record)?;
                let layer = pbd_core::column::layer_at(top - 1e-3)?;
                pbd_app::water_flow::drawn_top_m(&fine.set.columns, slot, layer)
            });
            return Some(drawn.unwrap_or(top));
        }
        a -= 0.05;
    }
    (pbd_app::planet::surface_height(direction) < sheet).then_some(sheet)
}

/// Put the camera where the path says this frame, and label it.
#[allow(clippy::too_many_arguments)]
pub fn drive(
    path: Option<Res<EyePath>>,
    count: Option<Res<super::CaptureState>>,
    contact: Option<Res<PlanetContact>>,
    fine: Option<Res<PlanetFine>>,
    eye: Option<Res<EyeWaterState>>,
    sea: Option<Res<pbd_app::sea::Sea>>,
    water: Option<Res<pbd_app::config::WaterSettings>>,
    mut cameras: Query<(&mut Transform, &Camera), With<Camera3d>>,
    mut labels: Query<&mut Text, With<EyePathLabel>>,
) {
    let (Some(keys), Some(count), Some(water)) =
        (path.as_ref().and_then(|p| p.0.as_ref()), count, water)
    else {
        return;
    };
    let frame = count.frame;
    let Some(key) = at(keys, frame as f32) else {
        return;
    };
    let up = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: key.lat.to_radians(),
        lon: key.lon.to_radians(),
    });
    let (north, east) = pbd_core::geo::compass_north_east(up);
    let yaw = key.yaw.to_radians();
    let ahead = (-east * yaw.cos() + north * yaw.sin()).normalize_or(east);
    let pitch = key.pitch.to_radians();
    let look = ahead * pitch.cos() + up * pitch.sin();
    // `PBD_EYE_PATH_RELATIVE=1`: the altitudes are over the water surface
    // under the eye (the ground where it is dry), so a path follows the
    // water across a step.
    let tide0 = sea.as_deref().map_or(0.0, |s| s.tide_at(up));
    let sheet0 = -water.depth_offset_m + tide0;
    let mut key = key;
    if std::env::var("PBD_EYE_PATH_RELATIVE").is_ok_and(|v| v == "1")
        && let Some(c) = contact.as_deref()
    {
        let ground = pbd_app::planet::surface_height(up);
        let base = surface_near(c, fine.as_deref(), up, ground.max(sheet0) + 30.0, sheet0)
            .map_or(ground, |s| s.max(ground.min(s)));
        key.alt += base;
    }
    let position = up * (PLANET_RADIUS + key.alt);
    for (mut transform, camera) in &mut cameras {
        if camera.is_active {
            *transform = Transform::from_translation(position).looking_to(look, up);
        }
    }
    let tide = sea.as_deref().map_or(0.0, |s| s.tide_at(up));
    let sheet = -water.depth_offset_m + tide;
    let surface = contact
        .as_deref()
        .and_then(|c| surface_near(c, fine.as_deref(), up, key.alt, sheet));
    let in_water = matches!(eye.as_deref().map(|e| e.0), Some(EyeWater::Water { .. }));
    let rel = surface.map(|s| key.alt - s);
    let line = match rel {
        Some(rel) => format!(
            "{rel:+.2} m to water  eye:{}  f{}",
            if in_water { "water" } else { "air" },
            frame
        ),
        None => format!("no water  f{}", frame),
    };
    info!(
        "EYE_PATH frame {} alt {:.3} surface {:?} in_water {in_water}",
        frame, key.alt, surface
    );
    for mut text in &mut labels {
        text.0 = line.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_lerps_between_its_keys_and_holds_past_its_ends() {
        let keys = parse("0 10 20 1.5 0 0\n# comment\n100 10 20 -2.5 90 10\n");
        assert_eq!(keys.len(), 2);
        let mid = at(&keys, 50.0).unwrap();
        assert!((mid.alt - (-0.5)).abs() < 1e-5);
        assert!((mid.yaw - 45.0).abs() < 1e-5);
        assert_eq!(at(&keys, -5.0).unwrap().alt, 1.5);
        assert_eq!(at(&keys, 500.0).unwrap().alt, -2.5);
    }
}
