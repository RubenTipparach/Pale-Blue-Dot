use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use pbd_core::{DQuat, orbit::CircularOrbit};

#[derive(Component)]
pub struct MoonOrbit(CircularOrbit);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    frame: Res<pbd_app::PhysicsFrame>,
    sun: Res<pbd_app::sky::Sun>,
) {
    let offset = (-frame.0.origin).as_vec3();
    let sun = sun.direction();
    commands.spawn((
        Name::new("Sun"),
        SunLight,
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(sun * 10_000.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let moon = meshes.add(Sphere::new(850.0).mesh().ico(4).unwrap());
    commands.spawn((
        Mesh3d(moon),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.39, 0.46),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_translation(Vec3::new(-16000.0, 6500.0, -19000.0) + offset),
        MoonOrbit(CircularOrbit::new(26000.0, 8000.0, 3.8, DQuat::from_rotation_x(0.35)).unwrap()),
    ));
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    for i in 0..1400_u32 {
        let hash = |v: u32| {
            let mut x = v.wrapping_mul(747796405).wrapping_add(2891336453);
            x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
            ((x >> 22) ^ x) as f32 / u32::MAX as f32
        };
        let y = hash(i * 3 + 7) * 2.0 - 1.0;
        let angle = hash(i * 3 + 9) * std::f32::consts::TAU;
        let direction = Vec3::new(
            (1.0 - y * y).sqrt() * angle.cos(),
            y,
            (1.0 - y * y).sqrt() * angle.sin(),
        );
        let tangent = direction.any_orthonormal_vector();
        let bitangent = direction.cross(tangent);
        let size = 20.0 + hash(i * 3 + 11).powi(8) * 150.0;
        let center = direction * 180_000.0;
        let tint = if i % 5 == 0 {
            [0.56, 0.72, 1.0, 1.0]
        } else {
            [0.9, 0.88, 0.75, 1.0]
        };
        for (x, y) in [
            (-1.0, -1.0),
            (1.0, -1.0),
            (1.0, 1.0),
            (-1.0, -1.0),
            (1.0, 1.0),
            (-1.0, 1.0),
        ] {
            positions.push((center + (tangent * x + bitangent * y) * size).to_array());
            colors.push(tint);
        }
    }
    let count = positions.len();
    let star_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    commands.spawn((
        Mesh3d(meshes.add(star_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        bevy::camera::visibility::NoFrustumCulling,
        Transform::from_translation(offset),
    ));
}

pub fn move_moon(
    time: Res<pbd_app::SimulationClock>,
    frame: Res<pbd_app::PhysicsFrame>,
    mut moons: Query<(&MoonOrbit, &mut Transform)>,
) {
    for (orbit, mut transform) in &mut moons {
        transform.translation = (orbit.0.sample(time.seconds).position - frame.0.origin).as_vec3();
    }
}

/// The scene's key light. Marked so it can be aimed at the sun every frame
/// rather than at the constant it was spawned with.
#[derive(Component)]
pub struct SunLight;

/// Keep the key light on the sun.
///
/// A directional light spawned once at a fixed direction is a world where the
/// sun's own shell sets and the light on the ground does not, which is worse
/// than no cycle at all: the terminator would cross a landscape that stayed
/// lit from the morning.
pub fn follow_sun(sun: Res<pbd_app::sky::Sun>, mut lights: Query<&mut Transform, With<SunLight>>) {
    let direction = sun.direction();
    for mut transform in &mut lights {
        *transform =
            Transform::from_translation(direction * 10_000.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
