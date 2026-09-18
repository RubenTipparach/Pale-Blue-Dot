//! Bevy material integration for the prototype's planetary atmosphere.
//!
//! Planet terrain/ocean are owned by `planet`; this shell contributes sky,
//! orbital haze and sparse clouds. Camera and sphere positions use the same
//! local world frame. See `docs/tenebris-comparison.md` for visual provenance.
use crate::config::WaterSettings;
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
// The shell keeps the ratio it was tuned at (1.2 R, which the scattering
// scale height is normalized against); the clouds sit 300 m up, twice the
// ~150 m summits, and the shell clears them by a wide margin.
pub const ATMOSPHERE_RADIUS: f32 = PLANET_RADIUS * 1.2;
pub const CLOUD_RADIUS: f32 = PLANET_RADIUS + 300.0;
/// How deep the cloud layer is. The clouds are a marched slab between
/// `CLOUD_RADIUS` and this much above it, rather than a surface at one radius:
/// a single sample has no interior to light, and thickness is the whole of what
/// separates a mass from a decal. 260 m against 300 m of base altitude puts the
/// tops at about twice the summit height, which is where the reference's sit.
pub const CLOUD_THICKNESS: f32 = 260.0;
pub const SUN_DIRECTION: Vec3 = Vec3::new(0.65, 0.75, 0.35);

/// The radius the sky treats as solid ground: the water sheet, which sits
/// `depth_offset_m` below sea level. With the sea-level sphere instead, the
/// few pixels between the sheet's silhouette and that sphere's tangent drew
/// the sky shader's ground as a dark line along the sea horizon.
pub fn solid_radius(water: &WaterSettings) -> f32 {
    PLANET_RADIUS - water.depth_offset_m
}

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(Startup, spawn_atmosphere)
            .add_systems(Update, follow_weather)
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
    /// Slab thickness in metres, the cover the weather field says is overhead,
    /// drift seconds, and how dark a cloud's shadowed underside goes.
    pub cloud_slab: Vec4,
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
    water: Res<WaterSettings>,
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
            sky.parameters.center_radius = center.extend(solid_radius(&water));
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

/// Hand the sky what the weather field says is overhead, and the drift clock.
///
/// The cover is the SAME number that decides whether it is raining on the
/// player, read off the same `Weather` resource: the sky a player stands under
/// and the rain falling on them cannot disagree about whether it is overcast,
/// because there is one answer and both read it. What the sky does NOT get is
/// the field at a distance - a cloud on the horizon is still the shader's own
/// noise, because `planet_gen::moisture` is fBm on the CPU. Closing that gap
/// means porting the fBm into WGSL and pinning the two against each other; it
/// is named in the change's tasks rather than pretended away here.
fn follow_weather(
    weather: Res<crate::weather::Weather>,
    clock: Res<Time>,
    shells: Query<&MeshMaterial3d<SkyMaterial>, With<PlanetAtmosphere>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let seconds = clock.elapsed_secs();
    for material in &shells {
        if let Some(sky) = materials.get_mut(&material.0) {
            sky.parameters.cloud_slab.y = weather.cover;
            sky.parameters.cloud_slab.z = seconds;
        }
    }
}

fn spawn_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    water: Res<WaterSettings>,
) {
    let sun = SUN_DIRECTION.normalize();
    let material = materials.add(SkyMaterial {
        parameters: SkyParameters {
            center_radius: Vec3::ZERO.extend(solid_radius(&water)),
            atmosphere: Vec4::new(ATMOSPHERE_RADIUS, 0.22, 0.30, 0.018),
            sun: sun.extend(3.2),
            scatter: Vec4::new(0.16, 0.52, 1.30, 0.64),
            // Radius, the CLEAR-sky density threshold, the extinction scale and
            // the night floor. 0.72 rather than the flat shell's 0.61: the slab
            // integrates a whole path where the shell took one sample, so the
            // same threshold covered far more sky. The overcast end is a
            // fraction of it in the shader, so one knob moves both.
            clouds: Vec4::new(CLOUD_RADIUS, 0.72, 0.52, 0.045),
            cloud_slab: Vec4::new(CLOUD_THICKNESS, 0.0, 0.0, 0.34),
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
            .insert_resource(WaterSettings::default())
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
                    center_radius: Vec3::ZERO.extend(solid_radius(&WaterSettings::default())),
                    atmosphere: Vec4::ZERO,
                    sun: Vec4::ZERO,
                    scatter: Vec4::ZERO,
                    clouds: Vec4::ZERO,
                    cloud_slab: Vec4::ZERO,
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
                center.extend(solid_radius(&WaterSettings::default()))
            );
        }
    }
}
