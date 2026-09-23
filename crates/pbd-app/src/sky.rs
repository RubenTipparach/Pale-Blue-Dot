//! Bevy material integration for the prototype's planetary atmosphere.
//!
//! Planet terrain/ocean are owned by `planet`; this shell contributes sky,
//! orbital haze and sparse clouds. Camera and sphere positions use the same
//! local world frame. See `docs/tenebris-comparison.md` for visual provenance.
use crate::config::{WaterSettings, WeatherSettings};
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
/// The tallest a cloud stands above its base. The clouds are a marched layer
/// between `CLOUD_RADIUS` and this much above it, and each place's own cloud
/// reaches a share of it that the weather map gives (`cloud_top`): a storm
/// towers to the full 450 m and a stratus deck lies in the bottom third. The
/// tops stay under the atmosphere shell (960 m up) with room to spare.
pub const CLOUD_THICKNESS: f32 = 450.0;
/// Where the sun is in the system frame: the core's fixed sun, which the
/// planet turns under. Noon is exactly this, so every capture framed against
/// the old constant still reads.
pub const SUN_DIRECTION: Vec3 = pbd_core::daylight::SUN_FIXED;

/// The world's clock and the one sun direction derived from it.
///
/// It was a `const` that six call sites each normalised their own copy of -
/// the sky, the water, the terrain, the scene's directional light, the capture
/// harness. One resource now, advanced in one place, read everywhere: a fact
/// written six times is a fact five of them eventually have wrong, and a sun
/// that MOVES is exactly the kind of fact that finds them.
#[derive(Resource, Clone, Copy, Debug, bevy::render::extract_resource::ExtractResource)]
pub struct Sun {
    pub clock: pbd_core::daylight::Clock,
    /// Whether the clock advances. A capture pins the hour instead, or the
    /// picture is a different one every run.
    pub running: bool,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            clock: pbd_core::daylight::Clock::default(),
            running: true,
        }
    }
}

impl Sun {
    /// The direction toward the sun, in the planet's frame: the fixed sun
    /// carried in by the planet's spin, which is the core's one rotation.
    pub fn direction(&self) -> Vec3 {
        self.clock.sun()
    }

    /// What carries anything fixed in the system frame - the star field, the
    /// moon's orbit - into the planet's frame this frame. The same rotation
    /// the sun direction is made of, so the sky turns as one.
    pub fn sky_rotation(&self) -> Quat {
        self.clock.sky_from_system()
    }

    /// How high the sun stands over a point, as a cosine. Positive is day.
    pub fn elevation(&self, up: Vec3) -> f32 {
        self.direction().dot(up.normalize_or(Vec3::Y))
    }
}

/// Advance the clock. One writer, so the hour cannot differ between systems
/// inside a frame.
fn run_clock(time: Res<Time>, mut sun: ResMut<Sun>) {
    if sun.running {
        let step = time.delta_secs();
        sun.clock.advance(step);
    }
}

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
            .init_resource::<Sun>()
            .init_resource::<CloudNow>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                CloudNow,
            >::default())
            .add_systems(Startup, (load_cloud_module, spawn_atmosphere))
            .add_systems(Update, (run_clock, follow_weather))
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
}

/// The cloud layer as the shaders' `pbd::clouds::CloudLayer` reads it, this
/// frame: its five lanes, built from `weather.ron` and the weather, extracted
/// for the clouds pass. One writer (`follow_weather`).
#[derive(
    Resource, Clone, Copy, Debug, Default, bevy::render::extract_resource::ExtractResource,
)]
pub struct CloudNow {
    pub clouds: Vec4,
    pub slab: Vec4,
    pub storm: Vec4,
    pub flash: Vec4,
    pub light: Vec4,
}

impl CloudNow {
    /// The lanes, in the order `pbd::clouds::CloudLayer` declares them.
    pub fn of(settings: &WeatherSettings, weather: &crate::weather::Weather, seconds: f32) -> Self {
        Self {
            clouds: Vec4::new(
                CLOUD_RADIUS,
                settings.cloud_ambient_sky,
                settings.cloud_extinction,
                settings.cloud_night_floor,
            ),
            slab: Vec4::new(
                CLOUD_THICKNESS,
                settings.cloud_ambient_ground,
                seconds,
                settings.cloud_phase_forward,
            ),
            storm: Vec4::new(
                settings.cloud_phase_back,
                settings.cloud_phase_blend,
                settings.cloud_storm_extinction,
                settings.lightning_cloud,
            ),
            flash: weather.flash,
            light: Vec4::new(
                settings.cloud_sun,
                settings.cloud_scatter_extinction_falloff,
                settings.cloud_scatter_energy_falloff,
                settings.cloud_scatter_phase_falloff,
            ),
        }
    }
}

/// `shaders/clouds.wgsl` is imported (`#import pbd::clouds`) rather than drawn,
/// so nothing loads it unless this does; held for the app's life so the import
/// always resolves.
#[derive(Resource)]
struct CloudModule(#[allow(dead_code)] Handle<Shader>);

fn load_cloud_module(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(CloudModule(assets.load("shaders/clouds.wgsl")));
}

/// The dome's clear-sky Rayleigh scale, Mie scale and sun radiance: what the
/// overcast takes its fractions of. Written once, so the spawn and the per-frame
/// overcast cannot disagree about what a clear sky is.
const CLEAR_RAYLEIGH: f32 = 0.30;
const CLEAR_MIE: f32 = 0.018;
const CLEAR_SUN: f32 = 3.2;

/// The dome's Rayleigh scale, Mie scale and sun radiance under `cover`, as
/// Tenebris's renderer modulates them: the blue cut, the white haze lifted, the
/// whole dimmed. Pure so a test can hold full cover to the configured factors.
pub fn overcast_scatter(settings: &WeatherSettings, cover: f32) -> (f32, f32, f32) {
    let cover = cover.clamp(0.0, 1.0);
    (
        CLEAR_RAYLEIGH * (1.0 - cover * settings.overcast_sky_blue_cut),
        CLEAR_MIE * (1.0 + cover * settings.overcast_sky_haze),
        CLEAR_SUN * (1.0 - cover * settings.overcast_sky_dim),
    )
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

/// Hand the dome the cover overhead, and the clouds pass its lanes.
///
/// The dome's overcast greying reads the cover over the player, the same
/// number that decides whether it is raining on them. The clouds themselves
/// read the cover of every place they stand over, off the weather maps: a
/// cloud on the horizon is the atmosphere's cloud there, not the player's.
fn follow_weather(
    weather: Res<crate::weather::Weather>,
    settings: Res<WeatherSettings>,
    clock: Res<Time>,
    sun: Res<Sun>,
    shells: Query<&MeshMaterial3d<SkyMaterial>, With<PlanetAtmosphere>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut now: ResMut<CloudNow>,
) {
    *now = CloudNow::of(&settings, &weather, clock.elapsed_secs());
    for material in &shells {
        if let Some(sky) = materials.get_mut(&material.0) {
            apply_weather(&mut sky.parameters, &settings, &weather);
            // The shell's sun rides the same clock everything else does; what
            // a moving sun changes is where the light comes FROM, and the
            // terminator the sky already draws turns that into a sunset.
            sky.parameters.sun = sun.direction().extend(sky.parameters.sun.w);
        }
    }
}

/// Everything the weather decides about the dome: the overcast's greying and
/// dimming of the scattering and the sun by the cover overhead. One function
/// for the spawn and every frame after it.
fn apply_weather(
    sky: &mut SkyParameters,
    settings: &WeatherSettings,
    weather: &crate::weather::Weather,
) {
    let cover = weather.cover;
    // The same cover that dims the ground greys the dome, so the two cannot
    // disagree about the weather.
    let (rayleigh, mie, radiance) = overcast_scatter(settings, cover);
    sky.atmosphere.z = rayleigh;
    sky.atmosphere.w = mie;
    sky.sun.w = radiance;
}

fn spawn_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    water: Res<WaterSettings>,
    weather_settings: Res<WeatherSettings>,
    weather: Res<crate::weather::Weather>,
    sun: Res<Sun>,
) {
    // `PBD_NO_SKY` leaves the atmosphere shell unspawned so the clear colour
    // shows through. It is a hole DETECTOR rather than a look: against a flat
    // background, anything that is not terrain is a place the ground failed to
    // close, and no amount of squinting at a blue sky can tell those from the
    // sky over a ridge.
    if std::env::var("PBD_NO_SKY").is_ok() {
        return;
    }
    let sun = sun.direction();
    let mut parameters = SkyParameters {
        center_radius: Vec3::ZERO.extend(solid_radius(&water)),
        atmosphere: Vec4::new(ATMOSPHERE_RADIUS, 0.22, CLEAR_RAYLEIGH, CLEAR_MIE),
        sun: sun.extend(CLEAR_SUN),
        scatter: Vec4::new(0.16, 0.52, 1.30, 0.64),
    };
    apply_weather(&mut parameters, &weather_settings, &weather);
    let material = materials.add(SkyMaterial { parameters });
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

    /// The light march and the view march take one extinction, the layer's,
    /// mixed by the cover: a numeric extinction in the light march is a cloud
    /// that shadows itself by a different rule than it hides the sky by, which
    /// is the `0.05` this replaced.
    #[test]
    fn the_cloud_light_march_uses_the_layers_extinction() {
        let shader = include_str!("../../../assets/shaders/clouds.wgsl");
        assert!(
            shader.contains(
                "let extinction = mix(layer.clouds.z, layer.storm.z, clamp(local.x, 0.0, 1.0));"
            ),
            "the view march's extinction is no longer the layer's"
        );
        let calls: Vec<&str> = shader
            .lines()
            .filter(|line| line.contains("cloud_light_depth(") && !line.contains("fn "))
            .collect();
        assert!(!calls.is_empty());
        for call in calls {
            assert!(
                call.contains(", extinction,"),
                "a light march takes an extinction other than the layer's: {call}"
            );
        }
        assert!(!shader.contains("*0.05"));
    }

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

    #[test]
    fn full_cover_greys_and_dims_the_dome_by_the_configured_factors() {
        let s = WeatherSettings::default();
        assert_eq!(
            overcast_scatter(&s, 0.0),
            (CLEAR_RAYLEIGH, CLEAR_MIE, CLEAR_SUN)
        );
        let (rayleigh, mie, sun) = overcast_scatter(&s, 1.0);
        assert!((rayleigh - CLEAR_RAYLEIGH * (1.0 - s.overcast_sky_blue_cut)).abs() < 1e-6);
        assert!((mie - CLEAR_MIE * (1.0 + s.overcast_sky_haze)).abs() < 1e-6);
        assert!((sun - CLEAR_SUN * (1.0 - s.overcast_sky_dim)).abs() < 1e-6);
        // Cover outside the field's range is clamped, never extrapolated.
        assert_eq!(overcast_scatter(&s, 3.0), overcast_scatter(&s, 1.0));
    }
}
