use bevy::{
    asset::RenderAssetUsages,
    mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use pbd_core::{DQuat, orbit::CircularOrbit};

#[derive(Component)]
pub struct MoonOrbit(CircularOrbit);

/// The star field, which is fixed in the system frame and turns with the sky.
#[derive(Component)]
pub struct Stars;

/// The stars' material (`steady-stars`): quads sized in screen pixels by
/// `stars.wgsl`, so a star never falls between pixels and flickers.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct StarMaterial {}

impl Material for StarMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/stars.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/stars.wgsl".into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(2),
        ])?];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// A star `metres` in half-width at the star sphere, as the field was laid
/// out: the half-width of its quad in screen pixels and its intensity. One
/// pixel is 110 m there (1,800 px over a 60 degree field of view, 180 km out).
/// It is drawn as a Gaussian at least 0.8 px wide (the quad three widths
/// out), which sums to the same light at any offset between pixel centres;
/// the light its true footprint held is spread over the Gaussian, so a faint
/// star stays faint.
fn star_pixels(metres: f32) -> (f32, f32) {
    const METRES_PER_PIXEL: f32 = 110.0;
    let sigma = (0.66 * metres / METRES_PER_PIXEL).max(0.8);
    let footprint = (2.0 * metres / METRES_PER_PIXEL).powi(2);
    let area = std::f32::consts::TAU * sigma * sigma;
    (3.0 * sigma, (footprint / area).min(1.0))
}

#[allow(clippy::too_many_arguments)]
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut star_materials: ResMut<Assets<StarMaterial>>,
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
    let mut corners = Vec::new();
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
        let size = 20.0 + hash(i * 3 + 11).powi(8) * 150.0;
        let center = direction * 180_000.0;
        let (half, intensity) = star_pixels(size);
        let tint = if i % 5 == 0 {
            [0.56, 0.72, 1.0]
        } else {
            [0.9, 0.88, 0.75]
        };
        let tint = [
            tint[0] * intensity,
            tint[1] * intensity,
            tint[2] * intensity,
            1.0,
        ];
        for (x, y) in [
            (-1.0, -1.0),
            (1.0, -1.0),
            (1.0, 1.0),
            (-1.0, -1.0),
            (1.0, 1.0),
            (-1.0, 1.0),
        ] {
            positions.push(center.to_array());
            corners.push([x * half, y * half]);
            colors.push(tint);
        }
    }
    let star_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    commands.spawn((
        Mesh3d(meshes.add(star_mesh)),
        MeshMaterial3d(star_materials.add(StarMaterial::default())),
        bevy::camera::visibility::NoFrustumCulling,
        Transform::from_translation(offset),
        Stars,
    ));
}

/// The moon on its orbit, in the system frame, carried into the planet's by
/// the spin: it crosses the sky as the planet turns under it, as the sun does.
pub fn move_moon(
    time: Res<pbd_app::SimulationClock>,
    frame: Res<pbd_app::PhysicsFrame>,
    sun: Res<pbd_app::sky::Sun>,
    mut moons: Query<(&MoonOrbit, &mut Transform)>,
) {
    let turn = sun.sky_rotation().as_dquat();
    for (orbit, mut transform) in &mut moons {
        let position = turn * orbit.0.sample(time.seconds).position;
        transform.translation = (position - frame.0.origin).as_vec3();
    }
}

/// The star field turns with the sky. One rotation, the sun's own, so a night
/// in which the sun set and the stars stood still cannot be drawn.
pub fn turn_stars(sun: Res<pbd_app::sky::Sun>, mut stars: Query<&mut Transform, With<Stars>>) {
    for mut transform in &mut stars {
        transform.rotation = sun.sky_rotation();
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
