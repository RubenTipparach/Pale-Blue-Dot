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
    /// Whether something solid stands over the camera's eye, so the rain in
    /// the sky does not reach it: no drops on the lens and no shower round
    /// it. The column's answer (`PlanetContact::open_to_sky`).
    pub sheltered: bool,
    /// A lightning strike: where it is, body-local metres, and how bright it
    /// is right now; zero between strikes. Lights the clouds, the rain and
    /// the ground.
    pub flash: Vec4,
}

impl Weather {
    /// The rain that is WATER: what wets the ground, runs down the lens and
    /// rings the sea. Snow is precipitation and none of those.
    pub fn liquid(&self) -> f32 {
        if self.snowing { 0.0 } else { self.rain }
    }
}

/// What is falling, as the shower draws it: how fast, how long and wide a
/// mark, what colour, and how far it sways. One description for rain and
/// snow, so the near shower and the shafts draw either through one path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fall {
    pub speed_mps: f32,
    pub length_m: f32,
    pub half_width_m: f32,
    pub color: [f32; 3],
    pub sway_m: f32,
    /// Opacity at the camera and at the edge of the shower disk. Snow is
    /// opaque where rain is a streak of water, so it keeps its own: at rain's
    /// 0.3 a white flake vanished against grey rock under a grey sky.
    pub alpha_near: f32,
    pub alpha_far: f32,
}

impl Fall {
    /// Rain or snow, off the settings.
    pub fn of(settings: &WeatherSettings, snow: bool) -> Self {
        if snow {
            Fall {
                speed_mps: settings.snow_fall_mps,
                length_m: settings.snow_size_m * 2.0,
                half_width_m: settings.snow_size_m,
                color: settings.snow_color,
                sway_m: settings.snow_sway_m,
                alpha_near: settings.snow_alpha_near,
                alpha_far: settings.snow_alpha_far,
            }
        } else {
            Fall {
                speed_mps: settings.rain_fall_mps,
                length_m: settings.rain_streak_m,
                half_width_m: settings.rain_width_m,
                color: settings.rain_color,
                sway_m: 0.0,
                alpha_near: settings.rain_alpha_near,
                alpha_far: settings.rain_alpha_far,
            }
        }
    }

    /// How far a mark has drifted sideways at `now`, from its own phase.
    fn sway(&self, now: f32, phase: f32) -> f32 {
        self.sway_m * (now * 0.7 + phase * std::f32::consts::TAU).sin()
    }
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
    contact: Option<Res<PlanetContact>>,
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
    weather.sheltered = contact.is_some_and(|contact| !contact.open_to_sky(body_local.as_vec3()));
    // Every twentieth the cover moves, and whenever the eye goes under or out
    // from under rock, so a capture says what weather it was taken under.
    let step = (cell.cover * 20.0).round() as i32 * 2 + i32::from(weather.sheltered);
    if *reported != Some(step) {
        *reported = Some(step);
        info!(
            "Weather here: cover {:.2}, rain {:.2}{}{}",
            cell.cover,
            weather.rain,
            if weather.snowing { ", snow" } else { "" },
            if weather.sheltered { ", sheltered" } else { "" }
        );
    }
}

fn follow_rain(time: Res<Time>, settings: Res<WeatherSettings>, mut weather: ResMut<Weather>) {
    // Snow does not wet the ground.
    let liquid = weather.liquid();
    weather.wetness = settle_wetness(
        weather.wetness,
        liquid,
        time.delta_secs(),
        settings.wet_fade_tau_s,
    );
}

/// The precipitation round the camera that the rain VOLUME is drawn from:
/// `pbd_core::weather::precipitation_map` on a plane tangent at `anchor`,
/// extracted to the render world and uploaded for the water pass to march.
/// Refilled when the camera strays a quarter of the map from the anchor, and
/// once a second of weather time, since rain moves slowly.
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct RainMap {
    pub anchor: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub cell_m: f32,
    pub size: u32,
    /// Row-major along `v`; negative where it snows.
    pub values: Vec<f32>,
    filled_at: f32,
}

fn fill_rain_map(
    now: FieldNow,
    frame: Res<PlanetRenderFrame>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut map: ResMut<RainMap>,
) {
    let Some(camera) = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform)
    else {
        return;
    };
    let Some(up) = (camera.translation().as_dvec3() - frame.center)
        .as_vec3()
        .try_normalize()
    else {
        return;
    };
    let s = &now.settings;
    let reach = s.rain_map_size as f32 * s.rain_map_cell_m;
    let strayed = up.angle_between(map.anchor) * PLANET_RADIUS > reach * 0.25;
    let reshaped = map.size != s.rain_map_size || map.cell_m != s.rain_map_cell_m;
    let stale = (now.seconds() - map.filled_at).abs() >= 1.0;
    if !(map.values.is_empty() || strayed || reshaped || stale) {
        return;
    }
    if map.values.is_empty() || strayed || reshaped {
        map.anchor = up;
        map.u = up.any_orthonormal_vector();
        map.v = up.cross(map.u);
    }
    map.size = s.rain_map_size;
    map.cell_m = s.rain_map_cell_m;
    map.values = field::precipitation_map(
        &now.field(),
        &TERRAIN,
        map.anchor,
        map.u,
        map.v,
        PLANET_RADIUS,
        map.size as usize,
        map.cell_m,
        now.seconds(),
    );
    map.filled_at = now.seconds();
}

/// Where a map cell is: its unit direction.
fn map_direction(map: &RainMap, row: usize, column: usize) -> Vec3 {
    let half = map.size as f32 * 0.5;
    let x = (column as f32 + 0.5 - half) * map.cell_m;
    let y = (row as f32 + 0.5 - half) * map.cell_m;
    (map.anchor * PLANET_RADIUS + map.u * x + map.v * y).normalize()
}

/// Lightning, off the field alone (`pbd_core::weather::Lightning`), among the
/// map's raining cells. A strike sits at the cloud base over its cell; its
/// brightness is what the clouds, the rain and the ground are lit by.
fn strike_lightning(now: FieldNow, map: Res<RainMap>, mut weather: ResMut<Weather>) {
    let s = &now.settings;
    let size = map.size as usize;
    let mut places = Vec::new();
    // Every fourth cell each way is plenty to find where it pours.
    for row in (0..size).step_by(4) {
        for column in (0..size).step_by(4) {
            let rain = map.values.get(row * size + column).copied().unwrap_or(0.0);
            if rain > 0.0 {
                places.push((map_direction(&map, row, column), rain));
            }
        }
    }
    let bolt = field::Lightning {
        slot_s: s.lightning_slot_s,
        chance: s.lightning_chance,
        storm_min: s.lightning_storm_min,
        flash_s: s.lightning_flash_s,
    };
    weather.flash = match bolt.strike(now.seconds(), &places) {
        Some(strike) => (strike.direction * crate::sky::CLOUD_RADIUS).extend(strike.brightness),
        None => Vec4::ZERO,
    };
}

/// A lightning bolt: a jagged ribbon from the cloud base at the strike down to
/// the ground under it, its kinks hashed off where it struck so a strike keeps
/// its shape through its flickers. Unlit and over white, so it blooms.
fn push_bolt(quads: &mut Quads, flash: Vec4, ground: f32, right: Vec3) {
    let top = flash.truncate();
    let up = top.normalize();
    let side = right - up * right.dot(up);
    let side = side.try_normalize().unwrap_or(up.any_orthonormal_vector());
    let depth = up.cross(side);
    let height = top.length() - ground;
    let seed = (top.x * 7.1 + top.y * 3.3 + top.z * 5.7).to_bits();
    let color = [
        2.4 * flash.w,
        2.6 * flash.w,
        3.2 * flash.w,
        flash.w.clamp(0.0, 1.0),
    ];
    let mut last = top;
    const KINKS: u32 = 9;
    for k in 1..=KINKS {
        let (angle, spread, _) = streak_seed(seed.wrapping_add(k));
        let t = k as f32 / KINKS as f32;
        let wander = if k == KINKS { 0.0 } else { 18.0 * spread };
        let next =
            up * (top.length() - height * t) + (side * angle.cos() + depth * angle.sin()) * wander;
        quads.push(last, next, side * 0.9, color);
        last = next;
    }
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

/// Whether rain is drawn at all from a camera `altitude` metres above the
/// GROUND under it: the streaks, the shafts and the drops on the lens. Above
/// `rain_lod_alt_m` a streak is sub-pixel and the storm reads through the
/// volume, the clouds and the haze. ONE gate, which the lens asks too.
pub fn rain_drawn_at(altitude: f32, settings: &WeatherSettings) -> bool {
    altitude <= settings.rain_lod_alt_m
}

/// How high a body-local point is above the ground under it, off the height
/// field, which both worlds can ask. The ground, not the sea: measured from sea
/// level the gate took every flake off a snowfield on a mountain 220 m up.
pub fn height_above_ground(point: Vec3) -> f32 {
    let Some(direction) = point.try_normalize() else {
        return 0.0;
    };
    point.length() - (PLANET_RADIUS + surface_height(direction).max(0.0))
}

/// How a raining cell at `distance` metres is drawn: the share of its streaks
/// kept and their opacity. Streaks thin toward `rain_lod_far_frac` across the
/// detail range and fade out over the blend band ending at it, where the rain
/// VOLUME has long since taken over the distance, so the handover never pops.
/// Opacity falls from `rain_alpha_near` toward `rain_alpha_far` with distance.
/// Pure so a test can hold the fade.
pub fn cell_lod(distance: f32, settings: &WeatherSettings) -> CellLod {
    let s = settings;
    let detail = s.rain_detail_range_m.max(1.0);
    let near = (1.0 - distance / detail).clamp(0.0, 1.0);
    let alpha = s.rain_alpha_far + (s.rain_alpha_near - s.rain_alpha_far) * near;
    let blend = s.rain_lod_blend_m.max(1.0);
    let out = ((distance - (detail - blend)) / blend).clamp(0.0, 1.0);
    let keep = (s.rain_lod_far_frac + (1.0 - s.rain_lod_far_frac) * near) * (1.0 - out);
    CellLod {
        streak_share: keep.clamp(0.0, 1.0),
        streak_alpha: alpha * (1.0 - out),
    }
}

/// See [`cell_lod`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLod {
    pub streak_share: f32,
    pub streak_alpha: f32,
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

/// The terrain shader's day curve: `smoothstep` over these two sun
/// elevations, and the share of the sky fill a face keeps at night. Written
/// here because the rain is drawn by an unlit CPU mesh that has to take the
/// same light as the ground; `the_rain_takes_the_grounds_day_curve` reads the
/// shader and holds it to these numbers.
const DAY_FROM: f32 = -0.13;
const DAY_TO: f32 = 0.20;
const NIGHT_FILL: f32 = 0.12;

/// How brightly rain is lit, 0..1: the diffuse sky the ground takes, which is
/// the day curve over a night floor, dimmed by the overcast's
/// `overcast_amb_dim`. Rain is lit by the sky around it and never by the sun
/// through the cloud it is falling out of, so there is no direct term.
/// Pure so a test can hold night, noon and a storm.
pub fn rain_light(sun_elevation: f32, cover: f32, settings: &WeatherSettings) -> f32 {
    let t = ((sun_elevation - DAY_FROM) / (DAY_TO - DAY_FROM)).clamp(0.0, 1.0);
    let daylight = t * t * (3.0 - 2.0 * t);
    let fill = NIGHT_FILL + (1.0 - NIGHT_FILL) * daylight;
    fill * (1.0 - cover.clamp(0.0, 1.0) * settings.overcast_amb_dim)
}

/// A `weather.ron` colour lit by `light` with an alpha, as the mesh's LINEAR
/// vertex colour. The file's colours are Tenebris's, authored as display
/// values for a renderer that writes them straight to the screen; taken as
/// linear they come out far lighter here, and grey rain reads as white. The mesh is unlit, so the light is applied here or nowhere: without
/// it a night storm glowed at noon's grey against a black sky.
fn vertex_color(display: [f32; 3], light: f32, alpha: f32) -> [f32; 4] {
    let linear = LinearRgba::from(Srgba::rgb(display[0], display[1], display[2]));
    let light = light.clamp(0.0, 1.0);
    [
        linear.red * light,
        linear.green * light,
        linear.blue * light,
        alpha.clamp(0.0, 1.0),
    ]
}

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
}

/// Everything the shower mesh is built from this frame.
struct ShowerFrame<'a> {
    settings: &'a WeatherSettings,
    contact: &'a PlanetContact,
    eye: Vec3,
    up: Vec3,
    right: Vec3,
    now: f32,
    /// How brightly the rain is lit (`rain_light`), at the camera.
    light: f32,
}

/// The near shower: a dense disk of streaks round the camera while it is
/// raining or snowing HERE: streaks, or flakes that sway.
fn near_shower(frame: &ShowerFrame, rain: f32, snow: bool, quads: &mut Quads) {
    let s = frame.settings;
    let fall = Fall::of(s, snow);
    // Snow falls twenty times slower than rain, so far fewer flakes cross the
    // view at any moment; it takes more of them to read as a snowfall.
    let density = if snow { s.snow_density } else { 1.0 };
    let count = (s.shower_max_streaks as f32 * rain * density) as usize;
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
        let fallen = (frame.now * fall.speed_mps / column + phase).rem_euclid(1.0);
        let top =
            base + frame.up * (column * (1.0 - fallen)) + frame.right * fall.sway(frame.now, phase);
        let alpha = fall.alpha_near
            * column_fade(fallen, 0.15)
            * (1.0 - spread * (1.0 - fall.alpha_far / fall.alpha_near.max(1e-3)));
        // A streak passing a metre from the eye is a bar across the screen,
        // not a raindrop; fade the ones that close out.
        let near = (top.distance(frame.eye) - 1.0).clamp(0.0, 3.0) / 3.0;
        let color = vertex_color(fall.color, frame.light, alpha * near);
        quads.push(
            top,
            top - frame.up * fall.length_m,
            frame.right * fall.half_width_m,
            color,
        );
    }
}

/// Rain near: every raining cell of the body-fixed lattice inside the detail
/// range, as shafts of streaks falling from the cloud base to the ground.
/// Tenebris's `draw_distant_shafts`, on our lattice. Beyond it the rain is the
/// volume the water pass marches (`RainMap`).
fn cell_shafts(frame: &ShowerFrame, field: &field::WeatherField, seconds: f32, quads: &mut Quads) {
    let s = frame.settings;
    let cell_angle = s.rain_cell_m / PLANET_RADIUS;
    let range_angle = s.rain_detail_range_m / PLANET_RADIUS;
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
        if distance > s.rain_detail_range_m {
            continue;
        }
        let lod = cell_lod(distance, s);
        // Square to the cell's own up and to the camera's line of sight: the
        // streaks spread across the cell along this, turned to the eye.
        let across = up
            .cross(base - frame.eye)
            .try_normalize()
            .unwrap_or(frame.right);
        let count = (per_cell * lod.streak_share).round() as u32;
        if count == 0 || lod.streak_alpha <= 0.001 {
            continue;
        }
        let t1 = across;
        let t2 = up.cross(t1);
        let fall = Fall::of(s, cell.precip == Precip::Snow);
        // Widened with distance only: at the camera a shaft streak is a near
        // shower streak, and six times one reads as a pole.
        let widen = 1.0
            + (s.rain_cell_width_mult - 1.0) * (distance / s.rain_detail_range_m.max(1.0)).min(1.0);
        let half = across * (fall.half_width_m * widen);
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
            let fallen = (frame.now * fall.speed_mps / height.max(1.0) + phase).rem_euclid(1.0);
            let top = foot + up * (height * (1.0 - fallen)) + t1 * fall.sway(frame.now, phase);
            let alpha = lod.streak_alpha * column_fade(fallen, 0.15);
            quads.push(
                top,
                top - up * fall.length_m,
                half,
                vertex_color(fall.color, frame.light, alpha),
            );
            streaks += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn rebuild_shower(
    time: Res<Time>,
    sun: Res<crate::sky::Sun>,
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
    // No rain at all from under the sea or above the altitude gate. Under
    // ROCK, no shower round the camera - it is dry there - but the rain
    // outside still draws: it stands on the surface, the rock hides it from
    // inside a cave, and from a cave mouth it is the rain you are sheltering
    // from. Whether the eye is under rock is the column's answer, read off
    // `Weather` so the lens and the shower cannot disagree about it.
    let submerged = surface_height(up) < 0.0 && radius < PLANET_RADIUS;
    let drawn =
        radius >= 1.0 && !submerged && rain_drawn_at(height_above_ground(eye), &now.settings);
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
        light: rain_light(sun.elevation(up), weather.cover, &now.settings),
    };
    let mut quads = Quads::default();
    if weather.flash.w > 0.0 {
        let strike = weather.flash.truncate().normalize();
        push_bolt(
            &mut quads,
            weather.flash,
            contact.sample(strike).radius,
            right,
        );
    }
    if drawn {
        if !weather.sheltered {
            near_shower(&shower, weather.rain, weather.snowing, &mut quads);
        }
        cell_shafts(&shower, &now.field(), now.seconds(), &mut quads);
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
            sheltered: false,
            flash: Vec4::ZERO,
        })
        .insert_resource(StormForcing(forcing))
        .insert_resource(WeatherEpoch(self.weather_at))
        .init_resource::<RainMap>()
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            RainMap,
        >::default())
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            Weather,
        >::default())
        .add_systems(Startup, spawn_shower)
        .add_systems(
            Update,
            (
                sample_field,
                follow_rain,
                cycle_rain,
                fill_rain_map,
                strike_lightning,
                rebuild_shower,
            )
                .chain(),
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
    fn the_rain_takes_the_grounds_day_curve() {
        // The shader the curve is transcribed from, read as shipped.
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        assert!(
            shader.contains(&format!(
                "smoothstep({DAY_FROM:.2},{DAY_TO:.2},sun_elevation)"
            )),
            "the terrain's day curve moved; move DAY_FROM and DAY_TO with it"
        );
        assert!(
            shader.contains(&format!("var night = {NIGHT_FILL:.2};")),
            "the terrain's night fill moved; move NIGHT_FILL with it"
        );
        let s = WeatherSettings::default();
        assert!(
            (rain_light(1.0, 0.0, &s) - 1.0).abs() < 1e-6,
            "clear noon is full light"
        );
        assert!(
            (rain_light(-1.0, 0.0, &s) - NIGHT_FILL).abs() < 1e-6,
            "night is the floor"
        );
        let storm_noon = rain_light(1.0, 1.0, &s);
        assert!((storm_noon - (1.0 - s.overcast_amb_dim)).abs() < 1e-6);
        assert!(
            rain_light(-1.0, 1.0, &s) < 0.07,
            "a night storm's rain is near black"
        );
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
    fn a_cells_streaks_fade_out_across_the_detail_range_without_a_pop() {
        let s = WeatherSettings::default();
        let near = cell_lod(0.0, &s);
        assert!((near.streak_share - 1.0).abs() < 1e-6);
        assert!((near.streak_alpha - s.rain_alpha_near).abs() < 1e-6);
        assert_eq!(cell_lod(s.rain_detail_range_m, &s).streak_share, 0.0);
        let mut last = near;
        let mut d = 0.0;
        while d < s.rain_detail_range_m {
            d += 0.5;
            let next = cell_lod(d, &s);
            assert!(
                (next.streak_share - last.streak_share).abs() < 0.02,
                "streaks pop at {d} m"
            );
            assert!(
                (next.streak_alpha - last.streak_alpha).abs() < 0.02,
                "alpha pops at {d} m"
            );
            last = next;
        }
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
