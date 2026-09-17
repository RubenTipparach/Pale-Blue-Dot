//! Bevy material integration for the prototype's planetary atmosphere.
//!
//! Planet terrain/ocean are owned by `planet`; this shell contributes sky,
//! orbital haze and sparse clouds. Camera and sphere positions use the same
//! local world frame. See `docs/tenebris-comparison.md` for visual provenance.
use crate::planet::{PlanetRenderFrame, update_planet_frame};
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderType,
        SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

pub use crate::planet::PLANET_RADIUS;
// The surface prototype peaks near +426 m after its elevation compression.
// Keep clouds above those peaks and the outer shell above the cloud layer.
pub const ATMOSPHERE_RADIUS: f32 = PLANET_RADIUS + 800.0;
pub const CLOUD_RADIUS: f32 = PLANET_RADIUS + 600.0;
pub const SUN_DIRECTION: Vec3 = Vec3::new(0.65, 0.75, 0.35);

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(Startup, spawn_atmosphere)
            .add_systems(
                PostUpdate,
                position_atmosphere
                    .after(update_planet_frame)
                    .before(TransformSystems::Propagate),
            );
    }
}

#[derive(Clone, Copy, ShaderType, Debug)]
pub struct SkyParameters {
    /// Planet center xyz, solid sea-level radius w, all in local world metres.
    pub center_radius: Vec4,
    /// Outer radius, normalized scale height, Rayleigh scale, Mie scale.
    pub atmosphere: Vec4,
    /// Unit vector toward sun xyz, radiance scale w.
    pub sun: Vec4,
    /// Rayleigh wavelength ratios RGB and Mie anisotropy w.
    pub scatter: Vec4,
    /// Clouds radius, coverage threshold, opacity and night-floor brightness.
    pub clouds: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyMaterial {
    #[uniform(0)]
    pub parameters: SkyParameters,
}

#[derive(Component)]
struct PlanetAtmosphere;

fn position_atmosphere(
    frame: Res<PlanetRenderFrame>,
    mut shells: Query<(&mut Transform, &MeshMaterial3d<SkyMaterial>), With<PlanetAtmosphere>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    // This is already bounded by the f64 system-origin subtraction shared with
    // the terrain. Mesh positions and the shader's body centre must move as one.
    let center = frame.center.as_vec3();
    for (mut transform, material) in &mut shells {
        if transform.translation != center {
            transform.translation = center;
        }
        if materials
            .get(&material.0)
            .is_some_and(|sky| sky.parameters.center_radius.truncate() != center)
            && let Some(sky) = materials.get_mut(&material.0)
        {
            sky.parameters.center_radius = center.extend(PLANET_RADIUS);
        }
    }
}

impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky_atmosphere.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
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
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Front surfaces are used from orbit, back surfaces from the ground.
        // The fragment shader discards the other side to avoid double blending.
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = false;
            depth.depth_compare = CompareFunction::GreaterEqual;
        }
        Ok(())
    }
}

fn spawn_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let sun = SUN_DIRECTION.normalize();
    let material = materials.add(SkyMaterial {
        parameters: SkyParameters {
            center_radius: Vec3::ZERO.extend(PLANET_RADIUS),
            atmosphere: Vec4::new(ATMOSPHERE_RADIUS, 0.22, 0.30, 0.018),
            sun: sun.extend(3.2),
            scatter: Vec4::new(0.16, 0.52, 1.30, 0.64),
            clouds: Vec4::new(CLOUD_RADIUS, 0.61, 0.52, 0.045),
        },
    });
    commands.spawn((
        Name::new("Planet atmosphere and cloud shell"),
        PlanetAtmosphere,
        Mesh3d(meshes.add(Sphere::new(ATMOSPHERE_RADIUS).mesh().uv(96, 64))),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CelestialScene, PhysicsFrame};
    use bevy::math::DVec3;

    #[test]
    fn sky_shell_and_uniform_follow_the_same_f64_body_frame_as_terrain() {
        let body = DVec3::new(1e12, -2e12, 3e12);
        let mut scene = CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 25.);
        scene.states[0].position = body;
        let mut app = App::new();
        app.insert_resource(scene)
            .insert_resource(PhysicsFrame(pbd_core::frame::LocalFrame {
                origin: body,
                ..default()
            }))
            .init_resource::<PlanetRenderFrame>()
            .init_resource::<Assets<SkyMaterial>>()
            .add_systems(
                PostUpdate,
                (update_planet_frame, position_atmosphere).chain(),
            );
        let material = app
            .world_mut()
            .resource_mut::<Assets<SkyMaterial>>()
            .add(SkyMaterial {
                parameters: SkyParameters {
                    center_radius: Vec3::ZERO.extend(PLANET_RADIUS),
                    atmosphere: Vec4::ZERO,
                    sun: Vec4::ZERO,
                    scatter: Vec4::ZERO,
                    clouds: Vec4::ZERO,
                },
            });
        let shell = app
            .world_mut()
            .spawn((
                PlanetAtmosphere,
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
            ))
            .id();

        for center in [Vec3::ZERO, Vec3::new(20_000.125, -40_000.25, 80_000.5)] {
            app.world_mut().resource_mut::<PhysicsFrame>().0.origin = body - center.as_dvec3();
            app.update();
            assert_eq!(
                app.world().resource::<PlanetRenderFrame>().center,
                center.as_dvec3()
            );
            assert_eq!(
                app.world().get::<Transform>(shell).unwrap().translation,
                center
            );
            let materials = app.world().resource::<Assets<SkyMaterial>>();
            assert_eq!(
                materials.get(&material).unwrap().parameters.center_radius,
                center.extend(PLANET_RADIUS)
            );
        }
    }
}
