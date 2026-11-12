//! `lighting-model` L6, simulated HDR: the eye adapts, what is brighter than
//! white blooms, a grade, and Khronos PBR Neutral (the owner's pick) as the
//! display transform, in our own pass (`tone_grade.wgsl`) since Bevy 0.18 has
//! no Khronos tonemapper. The camera is on `Tonemapping::None`, which also
//! skips Bevy's own auto exposure, so the eye is metered here instead.
//!
//! The meter reads what the game already knows where the camera stands: the
//! sky's light (L2), the sun after the air (L1) and the cloud over it, and the
//! voxel field's sky and lamp light at the eye, so walking into a cave or a
//! house opens the eye up. No GPU readback, no histogram pass. The mockup
//! metered its image; the two agree outdoors, and indoors this one answers
//! the light the field says is there.

use crate::bloom::LiteBloom;
use crate::config::LightingSettings;
use crate::planet::{PlanetContact, PlanetFine, PlanetRenderFrame};
use bevy::{
    core_pipeline::{
        core_3d::graph::Node3d,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
        tonemapping::Tonemapping,
    },
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_graph::{InternedRenderLabel, RenderLabel},
        render_resource::ShaderType,
    },
    shader::ShaderRef,
};
use pbd_core::sky_light;

/// What `tone_grade.wgsl` reads.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct ToneGrade {
    /// x exposure (a multiplier), y saturation, z dither in 8-bit steps, w a
    /// frame counter for the dither.
    pub look: Vec4,
    /// rgb the white balance.
    pub balance: Vec4,
}

impl FullscreenMaterial for ToneGrade {
    fn fragment_shader() -> ShaderRef {
        "shaders/tone_grade.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node3d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node3d::EndMainPassPostProcessing.intern(),
        ]
    }
}

/// The eye's state: where it is, in EV.
#[derive(Component, Clone, Copy, Default)]
pub struct Eye {
    ev: f32,
    primed: bool,
}

/// The sky's light where the camera is (L2), once a frame for whatever else
/// draws by it (`field_lit.wgsl`): the nine coefficients clear then under a
/// full deck, the cover over the camera, and whether the new model draws.
#[derive(Resource, Clone, Copy)]
pub struct SkyLight {
    pub sh: [Vec4; 18],
    pub cover: f32,
    pub new_model: bool,
    pub lamp_glow: f32,
    /// L3: the sun's apparent radius as a tangent, widened by the cover over
    /// the camera (zero: today's shadow filter).
    pub shadow_spread: f32,
}

impl Default for SkyLight {
    fn default() -> Self {
        Self {
            sh: crate::planet::sky_sh_pair(0.7),
            cover: 0.0,
            new_model: false,
            lamp_glow: 1.0,
            shadow_spread: 0.0,
        }
    }
}

/// The player's graphics choices (Settings, GRAPHICS), kept between runs in
/// `saves/preferences.ron`.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphics {
    pub bloom: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
struct Preferences {
    bloom: Option<bool>,
}

fn preferences_path() -> std::path::PathBuf {
    std::path::Path::new(crate::saves::ROOT).join("preferences.ron")
}

impl Graphics {
    /// The player's last choice, else the config's default.
    pub fn load(settings: &LightingSettings) -> Self {
        let kept = std::fs::read_to_string(preferences_path())
            .ok()
            .and_then(|text| ron::from_str::<Preferences>(&text).ok())
            .unwrap_or_default();
        Self {
            bloom: kept.bloom.unwrap_or_else(|| settings.bloom_by_default()),
        }
    }

    /// Keeps the choice for the next run. A failed write is logged and the
    /// game goes on: it is a preference, not a save.
    pub fn keep(&self) {
        let text = ron::ser::to_string_pretty(
            &Preferences {
                bloom: Some(self.bloom),
            },
            Default::default(),
        )
        .unwrap_or_default();
        let path = preferences_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(error) = std::fs::write(&path, text) {
            warn!("could not keep {}: {error}", path.display());
        }
    }
}

pub struct TonePlugin;

impl Plugin for TonePlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world()
            .get_resource::<LightingSettings>()
            .cloned()
            .unwrap_or_default();
        app.init_resource::<SkyLight>()
            .insert_resource(Graphics::load(&settings))
            .add_plugins(FullscreenMaterialPlugin::<ToneGrade>::default())
            .add_systems(
                Update,
                (dress_cameras, follow_bloom, meter_and_grade).chain(),
            );
    }
}

/// Bloom follows the player's choice, on every camera the new model dresses:
/// our own lighter one (`bloom.rs`) or Bevy's, as `lighting.ron` says.
fn follow_bloom(
    mut commands: Commands,
    graphics: Res<Graphics>,
    settings: Res<LightingSettings>,
    cameras: Query<(Entity, Has<Bloom>, Has<LiteBloom>), With<ToneGrade>>,
) {
    // `PBD_BLOOM=bevy|lite` turns one on whatever the player chose, for A/B
    // runs (a Mac's default is off).
    let forced = std::env::var("PBD_BLOOM").ok();
    let want = (graphics.bloom || forced.is_some())
        && settings.bloom_intensity > 0.0
        && std::env::var_os("PBD_NO_BLOOM").is_none();
    let lite = match forced.as_deref() {
        Some("bevy") => false,
        Some("lite") => true,
        _ => settings.bloom_lite,
    };
    for (entity, has_bevy, has_lite) in &cameras {
        if want && lite && !has_lite {
            commands.entity(entity).remove::<Bloom>().insert(LiteBloom {
                intensity: settings.bloom_intensity,
                threshold: settings.bloom_threshold,
            });
        } else if want && !lite && !has_bevy {
            commands.entity(entity).remove::<LiteBloom>().insert(Bloom {
                intensity: settings.bloom_intensity,
                prefilter: BloomPrefilter {
                    threshold: settings.bloom_threshold,
                    threshold_softness: 0.5,
                },
                composite_mode: BloomCompositeMode::Additive,
                ..Bloom::OLD_SCHOOL
            });
        } else if !want && (has_bevy || has_lite) {
            commands.entity(entity).remove::<(Bloom, LiteBloom)>();
        }
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// A 3D camera under the new model takes the display transform: no Bevy
/// tonemapping, bloom, our pass and an eye. Under today's it is left as
/// `configure_camera` made it.
#[allow(clippy::type_complexity)]
fn dress_cameras(
    mut commands: Commands,
    settings: Res<LightingSettings>,
    cameras: Query<Entity, (With<Camera3d>, Without<ToneGrade>)>,
) {
    // `PBD_NO_TONE=1` keeps the new shading but leaves the camera as it was
    // (a measurement switch: it prices the display transform and bloom).
    if !settings.new_model_on() || std::env::var_os("PBD_NO_TONE").is_some() {
        return;
    }
    for entity in &cameras {
        commands.entity(entity).insert((
            Tonemapping::None,
            ToneGrade {
                look: Vec4::new(1.0, settings.saturation, settings.dither, 0.0),
                balance: Vec4::ONE,
            },
            Eye::default(),
        ));
    }
}

/// What the eye meters, and so the exposure it goes toward, in EV: the
/// scene's mean luminance off a mid-dark ground (albedo 0.25) under the sky,
/// the sun and the lamps at the eye, against `exposure_key`.
pub fn target_ev(settings: &LightingSettings, e: f32, cover: f32, sky: f32, block: f32) -> f32 {
    let cover = cover.clamp(0.0, 1.0);
    let sky = sky.clamp(0.0, 1.0);
    let sh = crate::planet::sky_sh_pair(e);
    let mix = |k: usize| sh[k].lerp(sh[k + 9], cover).truncate().to_array();
    let coeffs: [[f32; 3]; 9] = std::array::from_fn(mix);
    let up = sky_light::sh_irradiance(&coeffs, [0.0, 1.0, 0.0]);
    let sun = sky_light::sun_air(e);
    let daylight = smooth(-0.13, 0.20, e);
    let lamp =
        luma(pbd_core::light::lamp_light(block)) * pbd_core::light::lamp_share(sky, daylight);
    let mean = 0.25
        * (luma(up) * sky + luma(sun) * e.max(0.0) * (1.0 - 0.85 * cover) * sky * 0.5 + lamp * 0.6)
        + 1e-5;
    let night = 1.0 - smooth(-0.12, 0.02, e);
    let top = settings.exposure_max_ev - settings.night_ev_cut * night;
    ((settings.exposure_key / mean).log2() * settings.exposure_gain)
        .clamp(settings.exposure_min_ev, top.max(settings.exposure_min_ev))
}

#[allow(clippy::too_many_arguments)]
fn meter_and_grade(
    time: Res<Time>,
    settings: Res<LightingSettings>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<PlanetRenderFrame>>,
    weather: Option<Res<crate::weather::Weather>>,
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut ToneGrade, &mut Eye)>,
    mut sky_light_now: ResMut<SkyLight>,
) {
    let (Some(sun), Some(frame)) = (sun, frame) else {
        return;
    };
    let cover = weather.map(|w| w.cover).unwrap_or(0.0);
    for (camera, at, mut grade, mut eye) in &mut cameras {
        if !camera.is_active {
            continue;
        }
        let position = (at.translation().as_dvec3() - frame.center).as_vec3();
        let e = position.normalize_or(Vec3::Y).dot(sun.direction());
        let (sky, block) = match (&fine, &contact) {
            (Some(fine), Some(contact)) => crate::field_light::field_at(fine, contact, position),
            _ => (1.0, 0.0),
        };
        *sky_light_now = SkyLight {
            sh: crate::planet::sky_sh_pair(e),
            cover,
            new_model: settings.new_model_on(),
            lamp_glow: settings.lamp_glow,
            shadow_spread: settings.shadow_spread() * (1.0 + settings.shadow_cover_soften * cover),
        };
        let target = target_ev(&settings, e, cover, sky, block);
        if eye.primed {
            let step = settings.adapt_ev_per_s * time.delta_secs();
            eye.ev += (target - eye.ev).clamp(-step, step);
        } else {
            eye.ev = target;
            eye.primed = true;
        }
        let golden = smooth(0.35, 0.05, e) * smooth(-0.06, 0.03, e);
        let night = 1.0 - smooth(-0.12, 0.02, e);
        let g = Vec3::from_array(settings.golden_balance);
        let n = Vec3::from_array(settings.night_balance);
        let balance = Vec3::ONE + g * golden + n * night;
        let saturation = settings.saturation
            - settings.night_desaturate * night
            - settings.cover_desaturate * cover;
        grade.look = Vec4::new(
            eye.ev.exp2(),
            saturation.max(0.0),
            settings.dither,
            (grade.look.w + 1.0) % 64.0,
        );
        grade.balance = balance.extend(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eye_opens_at_night_and_in_a_cave_but_only_so_far() {
        let s = LightingSettings::default();
        let noon = target_ev(&s, 0.9, 0.0, 1.0, 0.0);
        let night = target_ev(&s, -0.4, 0.0, 1.0, 0.0);
        let cave = target_ev(&s, 0.9, 0.0, 0.05, 0.0);
        assert!(night > noon && cave > noon, "{noon} {night} {cave}");
        // at night the top is cut, so lamp-lit walls keep their warmth
        assert!(
            night <= s.exposure_max_ev - s.night_ev_cut + 1e-4,
            "{night}"
        );
        assert!(cave <= s.exposure_max_ev + 1e-4);
        // a lamp at the eye brings it back down
        let lit = target_ev(&s, 0.9, 0.0, 0.05, 0.9);
        assert!(lit < cave, "{lit} {cave}");
    }
}
