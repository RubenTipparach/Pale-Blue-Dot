//! What the weather is HERE: the rain under the player, and the wetness the
//! ground remembers.
//!
//! Every rain effect (the water cap's ripples, the terrain's wet sheet and
//! rivulets, the lens droplets, the precipitation) reads these numbers and
//! keeps no rain state of its own. What changed when the field landed is only
//! where `rain` comes from: `pbd_core::weather` sampled under the player,
//! rather than a global switch. Not one consumer learned anything, which is the
//! one-code-path rule collecting a dividend it was owed.

use crate::config::WeatherSettings;
use crate::planet::terrain::TERRAIN;
use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetRenderFrame, surface_height};
use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::Aabb,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::extract_resource::ExtractResource,
};
use pbd_core::weather::{self as field, Precip};

#[derive(Resource, Clone, Copy, Debug, Default, ExtractResource)]
pub struct Weather {
    /// What the sky is doing now, 0..1.
    pub rain: f32,
    /// What the ground remembers, 0..1; follows `rain` on `wet_fade_tau_s`.
    pub wetness: f32,
    /// Cloud cover over the player, 0..1. The sky shader reads it, so the
    /// overcast a player stands under is the same one that is raining on them.
    pub cover: f32,
    /// What is falling here. The shower draws rain streaks either way until
    /// there is a snow particle; the field knows the difference already.
    pub snowing: bool,
}

/// How hard a storm the P key is currently forcing, 0..1.
///
/// The reference's weather menu forces one with `moisture_boost`, which lerps
/// every cell toward saturation. Forcing rain THROUGH the field rather than
/// around it is what keeps one path deciding the weather: at a boost of one the
/// whole planet is past the rain threshold, and it is still the field saying so.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct StormForcing(pub f32);

/// Seconds added to the planet clock for the weather field alone: the
/// `--weather-at` harness flag, and nothing else sets it.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WeatherEpoch(pub f32);

/// The weather field as it stands now: its knobs with the storm forcing folded
/// in, and the time it is asked at. ONE place, so the rain under the player and
/// the rain drawn on the horizon are the same field at the same time.
#[derive(bevy::ecs::system::SystemParam)]
struct FieldNow<'w> {
    clock: Res<'w, crate::planet::PlanetClock>,
    forcing: Res<'w, StormForcing>,
    epoch: Res<'w, WeatherEpoch>,
    settings: Res<'w, WeatherSettings>,
}

impl FieldNow<'_> {
    fn field(&self) -> field::WeatherField {
        self.settings.field(self.forcing.0)
    }

    fn seconds(&self) -> f32 {
        self.clock.seconds() + self.epoch.0
    }
}

/// Wetness after `dt` seconds chasing `rain` with e-fold time `tau`. Pure so a
/// test can hold the lag to the configured constant.
pub fn settle_wetness(wetness: f32, rain: f32, dt: f32, tau: f32) -> f32 {
    let blend = 1.0 - (-dt / tau.max(1e-6)).exp();
    (wetness + (rain - wetness) * blend).clamp(0.0, 1.0)
}

/// Sample the field under the player. This is the only place `rain` is set.
fn sample_field(
    now: FieldNow,
    frame: Res<PlanetRenderFrame>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut weather: ResMut<Weather>,
    mut reported: Local<Option<i32>>,
) {
    // The ACTIVE camera. Walking keeps the flight camera parked and inactive
    // beside its own, and `single()` over both failed every frame on foot, so
    // the field was never sampled while walking and the weather stayed at
    // whatever the launch forcing wrote.
    let Some(camera) = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform)
    else {
        return;
    };
    // The column under the player, in the body's own frame: the render frame
    // holds where the body is, and weather is a function of a surface
    // direction, so the camera has to come home before it can be asked.
    let body_local = camera.translation().as_dvec3() - frame.center;
    let Some(direction) = body_local.as_vec3().try_normalize() else {
        return;
    };
    let cell = field::cloud_cell(&now.field(), &TERRAIN, direction, now.seconds());
    weather.rain = if cell.raining { cell.cover } else { 0.0 };
    weather.cover = cell.cover;
    weather.snowing = cell.precip == Precip::Snow;
    // Every twentieth the cover moves, so a capture says what weather it was
    // taken under: the cloud tuning is measured against this number.
    let step = (cell.cover * 20.0).round() as i32;
    if *reported != Some(step) {
        *reported = Some(step);
        info!(
            "Weather here: cover {:.2}, rain {:.2}{}",
            cell.cover,
            weather.rain,
            if weather.snowing { ", snow" } else { "" }
        );
    }
}

fn follow_rain(time: Res<Time>, settings: Res<WeatherSettings>, mut weather: ResMut<Weather>) {
    weather.wetness = settle_wetness(
        weather.wetness,
        weather.rain,
        time.delta_secs(),
        settings.wet_fade_tau_s,
    );
}

/// P cycles the storm forcing through none, half and full, the way Tenebris's
/// weather menu does. It moves the FIELD rather than the rain, so a forced storm
/// is a real one: clouds thicken, the sky closes and it rains because the field
/// says it is overcast, not because a number was written past it.
fn cycle_rain(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    mut forcing: ResMut<StormForcing>,
) {
    if menu.is_some_and(|open| open.0) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        forcing.0 = match forcing.0 {
            f if f < 0.25 => 0.5,
            f if f < 0.75 => 1.0,
            _ => 0.0,
        };
        info!("Storm forcing {:.1}", forcing.0);
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

/// Whether rain is drawn at all from a camera `altitude` metres above sea
/// level: the streaks, the shafts, the curtains and the drops on the lens.
/// Above `rain_lod_alt_m` a streak is sub-pixel and the storm reads through the
/// clouds, the overcast and the haze. ONE gate, which the lens asks too.
pub fn rain_drawn_at(altitude: f32, settings: &WeatherSettings) -> bool {
    altitude <= settings.rain_lod_alt_m
}

/// How a raining cell at `distance` metres is drawn: the share of its streaks
/// kept and their opacity scale, and the opacity of its curtain. Streaks thin
/// toward `rain_lod_far_frac` across the detail range and fade out over the
/// blend band ending at it while the curtain fades in, so the handover is a
/// cross-fade and never a pop; beyond the detail range it is curtain only.
/// Opacity falls from `rain_alpha_near` to `rain_alpha_far` over the whole
/// range, which is what makes a far storm read as haze rather than a wall.
/// Pure so a test can hold the handover.
pub fn cell_lod(distance: f32, settings: &WeatherSettings) -> CellLod {
    let s = settings;
    let range_fade = (1.0 - distance / s.rain_range_m.max(1.0)).clamp(0.0, 1.0);
    let alpha = s.rain_alpha_far + (s.rain_alpha_near - s.rain_alpha_far) * range_fade;
    let blend = s.rain_lod_blend_m.max(1.0);
    let curtain = ((distance - (s.rain_detail_range_m - blend)) / blend).clamp(0.0, 1.0);
    let detail_fade = (1.0 - distance / s.rain_detail_range_m.max(1.0)).clamp(0.0, 1.0);
    let keep = (s.rain_lod_far_frac + (1.0 - s.rain_lod_far_frac) * detail_fade) * (1.0 - curtain);
    CellLod {
        streak_share: keep.clamp(0.0, 1.0),
        streak_alpha: alpha * (1.0 - curtain),
        curtain_alpha: alpha * s.rain_impostor_alpha * curtain,
    }
}

/// See [`cell_lod`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLod {
    pub streak_share: f32,
    pub streak_alpha: f32,
    pub curtain_alpha: f32,
}

/// One cell streak's offset across its cell (two numbers in -0.5..0.5) and its
/// phase down the column, from the cell and the streak's index alone.
fn cell_streak_seed(row: u32, column: u32, index: u32) -> (f32, f32, f32) {
    let (a, b, c) = streak_seed(
        index.wrapping_mul(0x27d4_eb2f)
            ^ row.wrapping_mul(0x1656_67b1)
            ^ column.wrapping_mul(0x9e37_79b9),
    );
    (a / std::f32::consts::TAU - 0.5, b * b - 0.5, c)
}

/// A `weather.ron` colour with an alpha, as the mesh's LINEAR vertex colour.
/// The file's colours are Tenebris's, authored as display values for a
/// renderer that writes them straight to the screen; taken as linear they come
/// out far lighter here, and a grey curtain reads as a white wall.
fn vertex_color(display: [f32; 3], alpha: f32) -> [f32; 4] {
    let linear = LinearRgba::from(Srgba::rgb(display[0], display[1], display[2]));
    [linear.red, linear.green, linear.blue, alpha.clamp(0.0, 1.0)]
}

/// Share of each side of a curtain that fades to nothing. A curtain is a cell
/// wide, so feathered this far its neighbour's feather overlaps it and the
/// sum across the seam stays roughly flat.
const CURTAIN_FEATHER: f32 = 0.5;

/// The shower mesh under construction: camera-facing quads, one colour each.
#[derive(Default)]
struct Quads {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Quads {
    /// A quad `half` either side of the segment from `top` down to `bottom`.
    fn push(&mut self, top: Vec3, bottom: Vec3, half: Vec3, color: [f32; 4]) {
        self.push_span(top, bottom, -half, half, [color, color]);
    }

    /// The strip between offsets `from` and `to` across the segment, each side
    /// its own colour.
    fn push_span(&mut self, top: Vec3, bottom: Vec3, from: Vec3, to: Vec3, colors: [[f32; 4]; 2]) {
        let first = self.positions.len() as u32;
        self.positions.extend([
            (top + from).to_array(),
            (top + to).to_array(),
            (bottom + to).to_array(),
            (bottom + from).to_array(),
        ]);
        self.colors
            .extend([colors[0], colors[1], colors[1], colors[0]]);
        self.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    /// A curtain: full opacity across its middle, feathered to nothing over
    /// the outer `CURTAIN_FEATHER` of each side, so neighbouring cells blend
    /// into one wall of rain instead of standing as panels with seams.
    fn push_curtain(&mut self, top: Vec3, bottom: Vec3, half: Vec3, color: [f32; 4]) {
        let clear = [color[0], color[1], color[2], 0.0];
        let inner = half * (1.0 - CURTAIN_FEATHER);
        self.push_span(top, bottom, -half, -inner, [clear, color]);
        self.push_span(top, bottom, -inner, inner, [color, color]);
        self.push_span(top, bottom, inner, half, [color, clear]);
    }
}

/// Everything the shower mesh is built from this frame.
struct ShowerFrame<'a> {
    settings: &'a WeatherSettings,
    contact: &'a PlanetContact,
    eye: Vec3,
    up: Vec3,
    right: Vec3,
    now: f32,
}

/// The near shower: a dense disk of streaks round the camera while it is
/// raining HERE. Unchanged from the first port.
fn near_shower(frame: &ShowerFrame, rain: f32, quads: &mut Quads) {
    let s = frame.settings;
    let count = (s.shower_max_streaks as f32 * rain) as usize;
    let reference = if frame.up.y.abs() < 0.9 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let t1 = reference.cross(frame.up).normalize();
    let t2 = frame.up.cross(t1);
    let column = s.shower_column_m.max(1.0);
    for index in 0..count as u32 {
        let (angle, spread, phase) = streak_seed(index);
        let distance = spread * s.shower_radius_m;
        let mark = frame.eye + (t1 * angle.cos() + t2 * angle.sin()) * distance;
        let direction = mark.normalize();
        let base = direction * frame.contact.sample(direction).radius;
        let fallen = (frame.now * s.rain_fall_mps / column + phase).rem_euclid(1.0);
        let top = base + frame.up * (column * (1.0 - fallen));
        let alpha = s.rain_alpha_near
            * column_fade(fallen, 0.15)
            * (1.0 - spread * (1.0 - s.rain_alpha_far / s.rain_alpha_near.max(1e-3)));
        let color = vertex_color(s.rain_color, alpha);
        quads.push(
            top,
            top - frame.up * s.rain_streak_m,
            frame.right * s.rain_width_m,
            color,
        );
    }
}

/// Rain seen from outside it: every raining cell of the body-fixed lattice in
/// range, as shafts of streaks near and one grey curtain far, falling from the
/// cloud base to the ground. Tenebris's `draw_distant_shafts`, on our lattice.
fn distant_rain(frame: &ShowerFrame, field: &field::WeatherField, seconds: f32, quads: &mut Quads) {
    let s = frame.settings;
    let cell_angle = s.rain_cell_m / PLANET_RADIUS;
    let range_angle = s.rain_range_m / PLANET_RADIUS;
    let cells = field::raining_cells(field, &TERRAIN, frame.up, seconds, cell_angle, range_angle);
    let cloud_base = crate::sky::CLOUD_RADIUS;
    let per_cell = s.rain_cell_density * s.rain_cell_m * s.rain_cell_m;
    let mut streaks = 0u32;
    for cell in cells {
        let up = cell.direction;
        let base_r = frame.contact.sample(up).radius;
        let height = cloud_base - base_r;
        if height < 2.0 {
            continue;
        }
        let base = up * base_r;
        let distance = base.distance(frame.eye);
        if distance > s.rain_range_m {
            continue;
        }
        let lod = cell_lod(distance, s);
        // Square to the cell's own up and to the camera's line of sight, so
        // a curtain is a sheet hanging from its cloud and turned to the eye.
        let across = up
            .cross(base - frame.eye)
            .try_normalize()
            .unwrap_or(frame.right);
        if lod.curtain_alpha > 0.001 {
            // Half again a cell wide, so the feathered edges overlap the
            // neighbours' rather than leaving a gap between them.
            quads.push_curtain(
                base + up * height,
                base,
                across * (s.rain_cell_m * 0.75),
                vertex_color(s.rain_impostor_color, lod.curtain_alpha),
            );
        }
        let count = (per_cell * lod.streak_share).round() as u32;
        if count == 0 || lod.streak_alpha <= 0.001 {
            continue;
        }
        let t1 = across;
        let t2 = up.cross(t1);
        // Widened with distance only: at the camera a shaft streak is a near
        // shower streak, and six times one reads as a pole.
        let widen = 1.0
            + (s.rain_cell_width_mult - 1.0) * (distance / s.rain_detail_range_m.max(1.0)).min(1.0);
        let half = across * (s.rain_width_m * widen);
        for index in 0..count {
            if streaks >= s.rain_max_cell_streaks {
                return;
            }
            let (u, v, phase) = cell_streak_seed(cell.row, cell.column, index);
            let foot = base + (t1 * u + t2 * v) * s.rain_cell_m;
            // A cell is fixed to the body, so its clearance is constant and so
            // is this rate. Tenebris's cells drift with the clouds, and keying
            // the rate to a clearance that changed every frame ran its drops
            // backwards; that cannot happen on a lattice that does not move.
            let fallen = (frame.now * s.rain_fall_mps / height.max(1.0) + phase).rem_euclid(1.0);
            let top = foot + up * (height * (1.0 - fallen));
            let alpha = lod.streak_alpha * column_fade(fallen, 0.15);
            quads.push(
                top,
                top - up * s.rain_streak_m,
                half,
                vertex_color(s.rain_color, alpha),
            );
            streaks += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn rebuild_shower(
    time: Res<Time>,
    now: FieldNow,
    weather: Res<Weather>,
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
    // No rain when the camera cannot see the sky: under water, or under the
    // terrain beneath it. The rendered cap, not the point-sampled noise, which
    // can sit a whole step above a walker standing on a cell edge. And none
    // from above the altitude gate.
    let ground = contact.sample(up).radius;
    let submerged = surface_height(up) < 0.0 && radius < PLANET_RADIUS;
    let sheltered = radius < 1.0 || submerged || radius < ground - 0.8;
    let drawn = !sheltered && rain_drawn_at(radius - PLANET_RADIUS, &now.settings);
    let Some(mesh) = meshes.get_mut(&mesh.0) else {
        return;
    };
    let reference = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let mut right = camera.right().as_vec3();
    right -= up * right.dot(up);
    let right = right
        .try_normalize()
        .unwrap_or_else(|| reference.cross(up).normalize());
    let shower = ShowerFrame {
        settings: &now.settings,
        contact: &contact,
        eye,
        up,
        right,
        now: time.elapsed_secs(),
    };
    let mut quads = Quads::default();
    if drawn {
        near_shower(&shower, weather.rain, &mut quads);
        distant_rain(&shower, &now.field(), now.seconds(), &mut quads);
    }
    *visibility = if quads.positions.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    // The bound follows the streaks: a bound computed once from the mesh's
    // first contents would sit at the planet's centre and cull the shower on
    // the GPU whatever the CPU-side culling was told.
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &quads.positions {
        low = low.min(Vec3::from_array(*p));
        high = high.max(Vec3::from_array(*p));
    }
    if quads.positions.is_empty() {
        low = eye;
        high = eye;
    }
    commands
        .entity(entity)
        .insert(Aabb::from_min_max(low, high));
    // Normals are not optional for a standard-material mesh: without the
    // attribute the varying is never written and the fragment comes out NaN,
    // which blends to nothing. The scene's star mesh learned the same thing.
    let normals = vec![up.to_array(); quads.positions.len()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, quads.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, quads.colors);
    mesh.insert_indices(Indices::U32(quads.indices));
}

pub struct WeatherPlugin {
    /// Rain intensity at launch, 0..1.
    pub rain: f32,
    /// Seconds the weather field starts at; see [`WeatherEpoch`].
    pub weather_at: f32,
}

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        // `--rain` is a storm FORCING now, not a rain level: it asks the field
        // for weather rather than overriding it, so the same one path decides
        // what the sky is doing whether or not a flag was passed.
        let forcing = self.rain.clamp(0.0, 1.0);
        app.insert_resource(Weather {
            rain: forcing,
            // Launching into rain starts with wet ground; a capture at frame 60
            // should not be waiting on a 1.6 s fade.
            wetness: forcing,
            cover: forcing,
            snowing: false,
        })
        .insert_resource(StormForcing(forcing))
        .insert_resource(WeatherEpoch(self.weather_at))
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            Weather,
        >::default())
        .add_systems(Startup, spawn_shower)
        .add_systems(
            Update,
            (sample_field, follow_rain, cycle_rain, rebuild_shower).chain(),
        );
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
    fn no_rain_and_no_lens_drops_above_the_altitude_gate() {
        let s = WeatherSettings::default();
        assert!(rain_drawn_at(0.0, &s));
        assert!(rain_drawn_at(s.rain_lod_alt_m, &s));
        assert!(!rain_drawn_at(s.rain_lod_alt_m + 0.5, &s));
        // The coast capture that showed drops on the glass was 420 m up.
        assert!(!rain_drawn_at(420.0, &s));
    }

    #[test]
    fn a_cell_hands_its_streaks_to_a_curtain_without_a_pop() {
        let s = WeatherSettings::default();
        let near = cell_lod(0.0, &s);
        assert_eq!(near.curtain_alpha, 0.0);
        assert!((near.streak_share - 1.0).abs() < 1e-6);
        assert!((near.streak_alpha - s.rain_alpha_near).abs() < 1e-6);
        // Past the detail range: curtain only.
        let far = cell_lod(s.rain_detail_range_m + 1.0, &s);
        assert_eq!(far.streak_share, 0.0);
        assert!(far.curtain_alpha > 0.0);
        // A storm 400 m off is drawn as a curtain.
        assert!(cell_lod(400.0, &s).curtain_alpha > 0.05);
        // Continuous across the blend band: no step bigger than a metre's
        // worth of fade anywhere from the camera to the range.
        let mut last = cell_lod(0.0, &s);
        let mut d = 0.0;
        while d < s.rain_range_m {
            d += 0.5;
            let next = cell_lod(d, &s);
            assert!(
                (next.streak_share - last.streak_share).abs() < 0.02,
                "streaks pop at {d} m"
            );
            assert!(
                (next.curtain_alpha - last.curtain_alpha).abs() < 0.02,
                "curtain pops at {d} m"
            );
            last = next;
        }
        assert_eq!(cell_lod(s.rain_range_m, &s).streak_share, 0.0);
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
