//! The weather field: one rain intensity, and the wetness the ground remembers.
//!
//! Every rain effect (the water cap's ripples, the terrain's wet sheet and
//! rivulets, the lens droplets, the precipitation) reads these two numbers and
//! keeps no rain state of its own. Rain is global until a cloud field shared
//! with the sky shader exists; see `openspec/changes/weather-rain`.

use crate::config::WeatherSettings;
use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetRenderFrame, surface_height};
use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::Aabb,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::extract_resource::ExtractResource,
};

#[derive(Resource, Clone, Copy, Debug, Default, ExtractResource)]
pub struct Weather {
    /// What the sky is doing now, 0..1.
    pub rain: f32,
    /// What the ground remembers, 0..1; follows `rain` on `wet_fade_tau_s`.
    pub wetness: f32,
}

/// Wetness after `dt` seconds chasing `rain` with e-fold time `tau`. Pure so a
/// test can hold the lag to the configured constant.
pub fn settle_wetness(wetness: f32, rain: f32, dt: f32, tau: f32) -> f32 {
    let blend = 1.0 - (-dt / tau.max(1e-6)).exp();
    (wetness + (rain - wetness) * blend).clamp(0.0, 1.0)
}

fn follow_rain(time: Res<Time>, settings: Res<WeatherSettings>, mut weather: ResMut<Weather>) {
    weather.wetness = settle_wetness(
        weather.wetness,
        weather.rain,
        time.delta_secs(),
        settings.wet_fade_tau_s,
    );
}

/// P cycles the rain through clear, half and full, the way Tenebris's storm
/// key does, so a storm can be looked at on demand.
fn cycle_rain(keys: Res<ButtonInput<KeyCode>>, mut weather: ResMut<Weather>) {
    if keys.just_pressed(KeyCode::KeyP) {
        weather.rain = match weather.rain {
            r if r < 0.25 => 0.5,
            r if r < 0.75 => 1.0,
            _ => 0.0,
        };
        info!("Rain {:.1}", weather.rain);
    }
}

/// The near shower: Tenebris's `weather_fx.rs`, on the CPU, as one mesh of
/// camera-facing streak quads rebuilt every frame. Stateless: each streak is
/// hashed to an angle, radius and phase and animated off the clock, so there
/// are no stored particles. Every streak lands on the authoritative surface
/// contact, which is the rendered cap or the sea, whichever is higher, so none
/// is drawn below the waterline or inside the ground.
#[derive(Component)]
struct Shower;

/// One streak's place on the disk and its phase down the column, from its
/// index alone. Pure so a test can hold the layout inside the disk.
pub fn streak_seed(index: u32) -> (f32, f32, f32) {
    let hash = |salt: u32| {
        let mut n = index.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
        n ^= n >> 15;
        n = n.wrapping_mul(0x2c1b_3c6d);
        n ^= n >> 12;
        (n & 0x00ff_ffff) as f32 / 16_777_216.0
    };
    (hash(1) * std::f32::consts::TAU, hash(2).sqrt(), hash(3))
}

/// Fade over the top and bottom `fraction` of the column, so the loop back to
/// the cloud and the cull at the surface never read as a pop.
pub fn column_fade(fallen: f32, fraction: f32) -> f32 {
    let edge = fraction.clamp(1e-3, 0.5);
    (fallen / edge).clamp(0.0, 1.0) * ((1.0 - fallen) / edge).clamp(0.0, 1.0)
}

fn spawn_shower(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Born with one degenerate triangle rather than no vertices: a mesh with
    // nothing in it has nothing to allocate on the GPU, and the first real
    // rebuild must replace a buffer, not create one.
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; 3])
    .with_inserted_indices(Indices::U32(vec![0, 1, 2]));
    commands.spawn((
        Name::new("Near shower"),
        Shower,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Transform::IDENTITY,
        // The mesh is rebuilt around the camera every frame; a bound computed
        // once from its first contents would cull it the moment the camera moved.
        bevy::camera::visibility::NoFrustumCulling,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
}

#[allow(clippy::too_many_arguments)]
fn rebuild_shower(
    time: Res<Time>,
    weather: Res<Weather>,
    settings: Res<WeatherSettings>,
    frame: Res<PlanetRenderFrame>,
    contact: Option<Res<PlanetContact>>,
    mut meshes: ResMut<Assets<Mesh>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut showers: Query<(Entity, &mut Transform, &Mesh3d, &mut Visibility), With<Shower>>,
    mut commands: Commands,
) {
    // Walking keeps the flight camera parked and inactive beside its own;
    // the shower follows whichever one is drawing.
    let active = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform);
    let (Some(camera), Some(contact)) = (active, contact) else {
        return;
    };
    let Ok((entity, mut transform, mesh, mut visibility)) = showers.single_mut() else {
        return;
    };
    let center = frame.center.as_vec3();
    transform.translation = center;
    let eye = camera.translation() - center;
    let radius = eye.length();
    let up = eye / radius.max(1e-3);
    // No shower when the camera cannot see the sky: under water, or under
    // the terrain beneath it. The rendered cap, not the point-sampled noise,
    // which can sit a whole step above a walker standing on a cell edge.
    let ground = contact.sample(up).radius;
    let submerged = surface_height(up) < 0.0 && radius < PLANET_RADIUS;
    let count = if radius < 1.0 || submerged || radius < ground - 0.8 {
        0
    } else {
        (settings.shower_max_streaks as f32 * weather.rain) as usize
    };
    *visibility = if count == 0 {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let Some(mesh) = meshes.get_mut(&mesh.0) else {
        return;
    };
    let reference = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let t1 = reference.cross(up).normalize();
    let t2 = up.cross(t1);
    let mut right = camera.right().as_vec3();
    right -= up * right.dot(up);
    let right = right.try_normalize().unwrap_or(t1);
    let now = time.elapsed_secs();
    let column = settings.shower_column_m.max(1.0);
    let mut positions = Vec::with_capacity(count * 4);
    let mut colors = Vec::with_capacity(count * 4);
    let mut indices = Vec::with_capacity(count * 6);
    for index in 0..count as u32 {
        let (angle, spread, phase) = streak_seed(index);
        let distance = spread * settings.shower_radius_m;
        let mark = eye + (t1 * angle.cos() + t2 * angle.sin()) * distance;
        let direction = mark.normalize();
        let base = direction * contact.sample(direction).radius;
        let fallen = (now * settings.rain_fall_mps / column + phase).rem_euclid(1.0);
        let top = base + up * (column * (1.0 - fallen));
        let bottom = top - up * settings.rain_streak_m;
        let half = right * settings.rain_width_m;
        let alpha = settings.rain_alpha_near
            * column_fade(fallen, 0.15)
            * (1.0 - spread * (1.0 - settings.rain_alpha_far / settings.rain_alpha_near.max(1e-3)));
        let color = [
            settings.rain_color[0],
            settings.rain_color[1],
            settings.rain_color[2],
            alpha.clamp(0.0, 1.0),
        ];
        let first = positions.len() as u32;
        positions.extend([
            (top - half).to_array(),
            (top + half).to_array(),
            (bottom + half).to_array(),
            (bottom - half).to_array(),
        ]);
        colors.extend([color; 4]);
        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }
    // The bound follows the streaks: a bound computed once from the mesh's
    // first contents would sit at the planet's centre and cull the shower on
    // the GPU whatever the CPU-side culling was told.
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &positions {
        low = low.min(Vec3::from_array(*p));
        high = high.max(Vec3::from_array(*p));
    }
    if positions.is_empty() {
        low = eye;
        high = eye;
    }
    commands
        .entity(entity)
        .insert(Aabb::from_min_max(low, high));
    // Normals are not optional for a standard-material mesh: without the
    // attribute the varying is never written and the fragment comes out NaN,
    // which blends to nothing. The scene's star mesh learned the same thing.
    let normals = vec![up.to_array(); positions.len()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
}

pub struct WeatherPlugin {
    /// Rain intensity at launch, 0..1.
    pub rain: f32,
}

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Weather {
            rain: self.rain.clamp(0.0, 1.0),
            // Launching into rain starts with wet ground; a capture at frame 60
            // should not be waiting on a 1.6 s fade.
            wetness: self.rain.clamp(0.0, 1.0),
        })
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            Weather,
        >::default())
        .add_systems(Startup, spawn_shower)
        .add_systems(Update, (follow_rain, cycle_rain, rebuild_shower).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaks_stay_inside_the_disk_and_fade_at_both_ends_of_the_column() {
        for index in 0..2000 {
            let (angle, spread, phase) = streak_seed(index);
            assert!((0.0..=std::f32::consts::TAU).contains(&angle));
            assert!((0.0..=1.0).contains(&spread));
            assert!((0.0..1.0).contains(&phase));
        }
        // Distinct streaks land in distinct places.
        assert_ne!(streak_seed(1), streak_seed(2));
        assert_eq!(column_fade(0.0, 0.15), 0.0);
        assert_eq!(column_fade(1.0, 0.15), 0.0);
        assert_eq!(column_fade(0.5, 0.15), 1.0);
        assert!((column_fade(0.075, 0.15) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn wetness_lags_rain_by_the_configured_time_constant() {
        let tau = 1.6;
        let mut wet = 0.0;
        for _ in 0..16 {
            wet = settle_wetness(wet, 1.0, 0.1, tau);
        }
        // One e-fold: 1 - 1/e.
        assert!((wet - 0.632).abs() < 0.01, "wetness after tau was {wet}");
        let mut dry = 1.0;
        for _ in 0..80 {
            dry = settle_wetness(dry, 0.0, 0.1, tau);
        }
        assert!(dry < 0.01, "five e-folds should be dry, got {dry}");
        assert_eq!(settle_wetness(0.5, 0.5, 100.0, tau), 0.5);
        assert_eq!(settle_wetness(0.3, 1.0, 0.0, tau), 0.3);
    }
}
