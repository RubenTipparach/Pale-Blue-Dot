//! The weather field: rain and cloud as a pure function of where and when.
//!
//! Ported from `tenebris-core/src/weather.rs` and its `weather.yaml`. Nothing
//! here is stored, saved or synchronised. A drifting field of warm pockets
//! condenses the terrain's own moisture map into cloud; where the cloud is
//! thick enough it rains, and the rain trails the cloud by sampling the same
//! field a little earlier rather than by remembering anything.
//!
//! It lives in the core because it is an engine-independent rule, and because a
//! field that is a pure function of direction and time is one a future
//! multiplayer agrees about without sending a byte.

use crate::planet_gen::{self, Biome, TerrainConfig};
use glam::Vec3;

/// What falls under a raining column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precip {
    None,
    Rain,
    Snow,
}

/// The knobs, named as `weather.yaml` names them so the two can be read side by
/// side. Values are its shipped values except where a doc comment says why.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherField {
    /// Unit-sphere frequency of the warm-pocket field: higher is smaller and
    /// more numerous systems.
    pub solar_scale: f32,
    /// How fast that field drifts and evolves, per second.
    pub solar_drift: f32,
    /// Warmth floor, so a moist region still clouds where the sun is weak.
    pub solar_floor: f32,
    /// Above one, clears the dry end harder: deserts cloud and rain rarely.
    pub arid_gamma: f32,
    /// Density below which the sky is clear.
    pub cloud_min: f32,
    /// Density above which the cover is full.
    pub cloud_full: f32,
    /// Opacity of the smallest just-forming cloud; it grows to one.
    pub min_alpha: f32,
    /// Cover a column needs before it rains, so a wisp does not.
    pub rain_cover_min: f32,
    /// How long rain trails a cloud that has drifted off, seconds.
    pub rain_min_s: f32,
    /// The frequency moisture is sampled at FOR WEATHER, on the unit sphere.
    ///
    /// The terrain samples moisture land-scale, at 188 m, which is the right
    /// size for deciding whether a hillside is jungle or fields. A weather
    /// system is not 188 m across. So the same fBm is asked a second question
    /// at a planet-scale frequency, and the biome's own moisture is left alone.
    pub weather_moisture_scale: f32,
    /// Lerps every cell toward saturation, 0..1. The reference's own way of
    /// forcing a storm from its weather menu without a second code path: at one
    /// the whole planet is past the rain threshold.
    pub moisture_boost: f32,
}

impl WeatherField {
    pub const DEFAULT: WeatherField = WeatherField {
        solar_scale: 2.2,
        solar_drift: 0.015,
        solar_floor: 0.55,
        arid_gamma: 1.35,
        cloud_min: 0.22,
        cloud_full: 0.52,
        min_alpha: 0.32,
        rain_cover_min: 0.62,
        rain_min_s: 15.0,
        weather_moisture_scale: 1.6,
        moisture_boost: 0.0,
    };
}

impl Default for WeatherField {
    fn default() -> Self {
        Self::DEFAULT
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge1 <= edge0 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Salt so the warm-pocket field is its own stream rather than the terrain's.
const SOLAR_SEED_SALT: u64 = 0x501a_125e_ed00_0001;
/// And so weather-scale moisture is not the terrain's moisture at a new scale
/// with the same values in a different place.
const WEATHER_MOISTURE_SALT: u64 = 0x501a_125e_ed00_0002;

/// Solar saturation in [0, 1]: a drifting field of warm pockets, two octaves
/// for shape and detail.
///
/// The reference uses simplex noise here and this uses the gradient noise the
/// whole terrain is built on, which is pinned bit for bit against Tenebris's.
/// A second noise basis for one caller would be a second thing to keep right;
/// what the field needs is a low-frequency drifting blob field, which both give.
/// The values could not have matched anyway, since the moisture they multiply is
/// sampled on a body sixteen times the reference's radius.
pub fn solar(field: &WeatherField, seed: u64, direction: Vec3, seconds: f32) -> f32 {
    let f = field.solar_scale;
    let d = seconds * field.solar_drift;
    let seed = seed ^ SOLAR_SEED_SALT;
    // The drift is a translation of the sample point, which is what makes the
    // field move across the world rather than flicker in place.
    let a = planet_gen::noise01(
        seed,
        direction * f + Vec3::new(d, d * 0.3, -d * 0.5),
        1.0,
        1,
    );
    let b = planet_gen::noise01(
        seed.rotate_left(1),
        direction * (f * 2.3) + Vec3::new(-d * 0.7, 0.0, d * 0.4),
        1.0,
        1,
    );
    (a * 0.68 + b * 0.32).clamp(0.0, 1.0)
}

/// The moisture the WEATHER reads: the same fBm the terrain uses, asked at a
/// weather system's own scale rather than a hillside's.
pub fn weather_moisture(field: &WeatherField, terrain: &TerrainConfig, direction: Vec3) -> f32 {
    planet_gen::noise01(
        terrain.seed ^ WEATHER_MOISTURE_SALT,
        direction,
        field.weather_moisture_scale,
        4,
    )
}

/// Cloud density in [0, 1]: warmth condensing the moisture source, with the dry
/// end cleared by the aridity gamma, then lerped toward saturation by the boost.
pub fn cloud_density(
    field: &WeatherField,
    terrain: &TerrainConfig,
    direction: Vec3,
    seconds: f32,
) -> f32 {
    let base = weather_moisture(field, terrain, direction);
    let heat = field.solar_floor
        + (1.0 - field.solar_floor) * solar(field, terrain.seed, direction, seconds);
    let density = (base * heat).clamp(0.0, 1.0).powf(field.arid_gamma);
    let boost = field.moisture_boost.clamp(0.0, 1.0);
    (density + boost * (1.0 - density)).clamp(0.0, 1.0)
}

/// What falls under a raining column: snow over the cold biomes, rain else.
pub fn precip_kind(terrain: &TerrainConfig, direction: Vec3) -> Precip {
    match planet_gen::biome(terrain, direction) {
        Biome::Tundra | Biome::Mountains => Precip::Snow,
        _ => Precip::Rain,
    }
}

/// One column's weather.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloudCell {
    /// Cover in [0, 1]: nothing at zero, a full mass at one.
    pub cover: f32,
    /// Opacity in [0, 1]: a small cloud is transparent, a big one is not.
    pub alpha: f32,
    /// Whether this column is precipitating.
    pub raining: bool,
    /// What falls, meaningful only when `raining`.
    pub precip: Precip,
}

/// Sample the whole of one column's weather.
pub fn cloud_cell(
    field: &WeatherField,
    terrain: &TerrainConfig,
    direction: Vec3,
    seconds: f32,
) -> CloudCell {
    let density = cloud_density(field, terrain, direction, seconds);
    let cover = smoothstep(field.cloud_min, field.cloud_full, density);
    let alpha = field.min_alpha + (1.0 - field.min_alpha) * cover;
    // Rain needs a solid cloud overhead, and it TRAILS one that has drifted
    // off: a column that was this cloudy `rain_min_s` ago still counts. Both
    // samples come out of the same deterministic field, so nothing is stored
    // and every client agrees without being told.
    let raining = if cover >= field.rain_cover_min {
        true
    } else if field.rain_min_s > 0.0 {
        let past = cloud_density(field, terrain, direction, seconds - field.rain_min_s);
        smoothstep(field.cloud_min, field.cloud_full, past) >= field.rain_cover_min
    } else {
        false
    };
    CloudCell {
        cover,
        alpha,
        raining,
        precip: if raining {
            precip_kind(terrain, direction)
        } else {
            Precip::None
        },
    }
}

/// How hard it is raining in [0, 1] at a column: the cover where it rains and
/// nothing where it does not. This is the ONE number every rain effect reads.
pub fn rain_at(
    field: &WeatherField,
    terrain: &TerrainConfig,
    direction: Vec3,
    seconds: f32,
) -> f32 {
    let cell = cloud_cell(field, terrain, direction, seconds);
    if cell.raining { cell.cover } else { 0.0 }
}

/// One cell of the rain lattice: a row from the north pole and a place along
/// it, and the direction of its centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainCell {
    pub row: u32,
    pub column: u32,
    pub direction: Vec3,
    /// What falls there.
    pub precip: Precip,
}

/// The lattice rain is drawn on: rows of constant latitude `cell_angle` apart,
/// each cut into as many cells as fit around it. It is FIXED TO THE BODY, the
/// shape Tenebris's distant shafts use, so a curtain belongs to a place and
/// does not slide as the camera moves; which cells rain is the field's answer
/// and changes with time, the cells themselves never do.
///
/// Calls `visit` with every lattice cell whose centre is within `range_angle`
/// of `eye`, in row then column order.
pub fn lattice_near(
    eye: Vec3,
    cell_angle: f32,
    range_angle: f32,
    mut visit: impl FnMut(u32, u32, Vec3),
) {
    let Some(eye) = eye.try_normalize() else {
        return;
    };
    let cell_angle = cell_angle.max(1e-5);
    let cos_range = range_angle.min(std::f32::consts::PI).cos();
    let rows = (std::f32::consts::PI / cell_angle).ceil() as u32;
    let eye_theta = eye.y.clamp(-1.0, 1.0).acos();
    for row in 0..rows {
        let theta = (row as f32 + 0.5) * cell_angle;
        if theta >= std::f32::consts::PI || (theta - eye_theta).abs() > range_angle + cell_angle {
            continue;
        }
        let (sin_t, cos_t) = theta.sin_cos();
        let columns = (std::f32::consts::TAU * sin_t.max(1e-4) / cell_angle)
            .ceil()
            .max(1.0) as u32;
        let step = std::f32::consts::TAU / columns as f32;
        for column in 0..columns {
            let phi = (column as f32 + 0.5) * step;
            let direction = Vec3::new(sin_t * phi.cos(), cos_t, sin_t * phi.sin());
            if direction.dot(eye) >= cos_range {
                visit(row, column, direction);
            }
        }
    }
}

/// Every cell of the rain lattice within `range_angle` of `eye` that is raining
/// now, which is where a curtain or a shaft is drawn: the field's own answer,
/// sampled at each cell's centre, so a curtain stands exactly where the field
/// says it rains and nowhere it does not.
pub fn raining_cells(
    field: &WeatherField,
    terrain: &TerrainConfig,
    eye: Vec3,
    seconds: f32,
    cell_angle: f32,
    range_angle: f32,
) -> Vec<RainCell> {
    let mut cells = Vec::new();
    lattice_near(eye, cell_angle, range_angle, |row, column, direction| {
        let cell = cloud_cell(field, terrain, direction, seconds);
        if cell.raining {
            cells.push(RainCell {
                row,
                column,
                direction,
                precip: cell.precip,
            });
        }
    });
    cells
}

/// A square map of precipitation around a point, for a renderer to draw rain
/// as a volume: `size` x `size` cells of `cell_m` metres on the plane tangent
/// to `anchor` (a unit direction) at `radius`, row by row along `v`, each the
/// field's rain intensity at its centre, NEGATIVE where it snows, zero where
/// it is dry. `u` and `v` are the plane's unit axes.
///
/// A point is carried onto the plane gnomonically (`anchor * radius + u * x +
/// v * y`, normalised), which is also how a shader reads the map back.
#[allow(clippy::too_many_arguments)]
pub fn precipitation_map(
    field: &WeatherField,
    terrain: &TerrainConfig,
    anchor: Vec3,
    u: Vec3,
    v: Vec3,
    radius: f32,
    size: usize,
    cell_m: f32,
    seconds: f32,
) -> Vec<f32> {
    let mut map = Vec::with_capacity(size * size);
    let half = size as f32 * 0.5;
    for row in 0..size {
        for column in 0..size {
            let x = (column as f32 + 0.5 - half) * cell_m;
            let y = (row as f32 + 0.5 - half) * cell_m;
            let direction = (anchor * radius + u * x + v * y).normalize();
            let cell = cloud_cell(field, terrain, direction, seconds);
            map.push(match (cell.raining, cell.precip) {
                (true, Precip::Snow) => -cell.cover,
                (true, _) => cell.cover,
                _ => 0.0,
            });
        }
    }
    map
}

/// When and where lightning strikes. Time is cut into `slot_s` slots and each
/// slot's hash decides whether it strikes, when inside the slot, and which of
/// the heavily raining places it picks: a pure function of the weather time and
/// the rain, so every client sees the same strike without being told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lightning {
    /// Length of one slot, seconds.
    pub slot_s: f32,
    /// Chance a slot strikes when the heaviest rain is at full.
    pub chance: f32,
    /// Rain intensity below which nothing strikes.
    pub storm_min: f32,
    /// How long one strike's flashes last, seconds.
    pub flash_s: f32,
}

/// A strike under way: where (a unit direction) and how bright right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub direction: Vec3,
    pub brightness: f32,
}

fn slot_hash(slot: i64, salt: u32) -> f32 {
    let mut n = (slot as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(salt).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    n ^= n >> 31;
    n = n.wrapping_mul(0x94d0_49bb_1331_11eb);
    n ^= n >> 29;
    (n >> 40) as f32 / (1u64 << 24) as f32
}

impl Lightning {
    /// The strike under way at `seconds`, if any, among `places`: unit
    /// directions and their rain intensity (negative for snow, which never
    /// strikes here). Silent where nothing rains past `storm_min`.
    pub fn strike(&self, seconds: f32, places: &[(Vec3, f32)]) -> Option<Strike> {
        let slot_s = self.slot_s.max(self.flash_s + 0.01);
        let heavy: Vec<Vec3> = places
            .iter()
            .filter(|(_, rain)| *rain >= self.storm_min)
            .map(|(direction, _)| *direction)
            .collect();
        if heavy.is_empty() {
            return None;
        }
        let strongest = places.iter().map(|(_, rain)| *rain).fold(0.0f32, f32::max);
        let storm =
            ((strongest - self.storm_min) / (1.0 - self.storm_min).max(1e-3)).clamp(0.0, 1.0);
        let slot = (seconds / slot_s).floor() as i64;
        if slot_hash(slot, 1) >= self.chance * (0.35 + 0.65 * storm) {
            return None;
        }
        let start = slot as f32 * slot_s + slot_hash(slot, 2) * (slot_s - self.flash_s);
        let t = seconds - start;
        if !(0.0..=self.flash_s).contains(&t) {
            return None;
        }
        let pick = ((slot_hash(slot, 3) * heavy.len() as f32) as usize).min(heavy.len() - 1);
        Some(Strike {
            direction: heavy[pick],
            brightness: flicker(t / self.flash_s),
        })
    }
}

/// A strike's brightness over its life, 0..1 of it: a bright first stroke and
/// two weaker return strokes, which is what makes a flash read as lightning
/// rather than a lamp switched on.
pub fn flicker(t: f32) -> f32 {
    let pulse = |at: f32, width: f32| (-((t - at) / width).powi(2)).exp();
    pulse(0.08, 0.06)
        .max(0.55 * pulse(0.38, 0.07))
        .max(0.8 * pulse(0.62, 0.08))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;

    fn dirs(count: usize) -> Vec<Vec3> {
        // A Fibonacci sphere, so the samples below cover the whole body rather
        // than one latitude.
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..count)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let a = golden * i as f32;
                Vec3::new(a.cos() * r, y, a.sin() * r).normalize()
            })
            .collect()
    }

    #[test]
    fn rain_is_drawn_on_every_raining_cell_in_view_and_no_dry_one() {
        // 60 m cells within 1400 m on the 4,800 m body, the shipped shape.
        let f = WeatherField {
            moisture_boost: 0.35,
            ..WeatherField::DEFAULT
        };
        let (cell, range) = (60.0 / 4800.0, 1400.0 / 4800.0);
        let mut checked = 0;
        for (i, eye) in dirs(12).into_iter().enumerate() {
            let t = 311.0 * i as f32;
            let drawn = raining_cells(&f, &TERRAIN, eye, t, cell, range);
            // The brute force: every cell of the WHOLE lattice, asked directly.
            let mut expected = Vec::new();
            lattice_near(eye, cell, std::f32::consts::PI, |row, column, direction| {
                if direction.dot(eye) >= range.cos()
                    && cloud_cell(&f, &TERRAIN, direction, t).raining
                {
                    expected.push((row, column));
                }
            });
            let got: Vec<_> = drawn.iter().map(|c| (c.row, c.column)).collect();
            assert_eq!(
                got, expected,
                "eye {eye}: the curtains must be exactly the raining cells"
            );
            for c in &drawn {
                assert!(cloud_cell(&f, &TERRAIN, c.direction, t).raining);
                assert!(c.direction.dot(eye) >= range.cos() - 1e-6);
            }
            checked += expected.len();
        }
        assert!(
            checked > 50,
            "the forcing should put some rain in view; got {checked} cells"
        );
    }

    #[test]
    fn the_rain_lattice_is_fixed_to_the_body() {
        // The same cell seen from two eyes is the same place, so a curtain
        // does not slide as the camera moves.
        let (cell, range) = (60.0 / 4800.0, 1400.0 / 4800.0);
        let a = Vec3::new(0.3, 0.8, 0.2).normalize();
        let b = Vec3::new(0.31, 0.79, 0.21).normalize();
        let mut from_a = std::collections::BTreeMap::new();
        lattice_near(a, cell, range, |r, c, d| {
            from_a.insert((r, c), d);
        });
        let mut shared = 0;
        lattice_near(b, cell, range, |r, c, d| {
            if let Some(&other) = from_a.get(&(r, c)) {
                assert_eq!(other, d);
                shared += 1;
            }
        });
        assert!(shared > 100);
        // Neighbouring centres are about a cell apart, not a planet apart.
        let mut first = None;
        lattice_near(a, cell, cell * 1.5, |_, _, d| {
            if let Some(f) = first {
                assert!(d.angle_between(f) < cell * 3.0);
            } else {
                first = Some(d);
            }
        });
    }

    #[test]
    fn the_precipitation_map_is_the_field_at_each_cell() {
        let f = WeatherField {
            moisture_boost: 0.6,
            ..WeatherField::DEFAULT
        };
        let anchor = Vec3::new(0.8772, 0.4801, 0.0).normalize();
        let u = anchor.any_orthonormal_vector();
        let v = anchor.cross(u);
        let (size, cell, radius) = (16, 50.0, 4800.0);
        let map = precipitation_map(&f, &TERRAIN, anchor, u, v, radius, size, cell, 120.0);
        assert_eq!(map.len(), size * size);
        for row in 0..size {
            for column in 0..size {
                let x = (column as f32 + 0.5 - 8.0) * cell;
                let y = (row as f32 + 0.5 - 8.0) * cell;
                let d = (anchor * radius + u * x + v * y).normalize();
                let c = cloud_cell(&f, &TERRAIN, d, 120.0);
                let want = if !c.raining {
                    0.0
                } else if c.precip == Precip::Snow {
                    -c.cover
                } else {
                    c.cover
                };
                assert_eq!(map[row * size + column], want);
            }
        }
    }

    #[test]
    fn lightning_is_deterministic_and_silent_without_a_storm() {
        let bolt = Lightning {
            slot_s: 7.0,
            chance: 0.6,
            storm_min: 0.75,
            flash_s: 0.7,
        };
        let d = Vec3::Y;
        // Nothing heavy enough: never a strike.
        let light = [(d, 0.5), (Vec3::X, 0.7)];
        for i in 0..2000 {
            assert_eq!(bolt.strike(i as f32 * 0.37, &light), None);
        }
        // A storm: strikes happen, the same ones every time, where it pours.
        let storm = [(d, 1.0), (Vec3::X, 0.2)];
        let mut struck = 0;
        for i in 0..20_000 {
            let t = i as f32 * 0.05;
            let a = bolt.strike(t, &storm);
            assert_eq!(a, bolt.strike(t, &storm));
            if let Some(s) = a {
                assert_eq!(s.direction, d, "only the heavy place is struck");
                assert!((0.0..=1.0).contains(&s.brightness));
                struck += 1;
            }
        }
        assert!(
            struck > 100,
            "a full storm over 1000 s strikes: {struck} samples lit"
        );
        assert!(flicker(0.08) > 0.99 && flicker(1.0) < 0.05);
    }

    #[test]
    fn the_field_is_a_field_and_keeps_nothing() {
        let f = WeatherField::DEFAULT;
        for d in dirs(64) {
            for t in [0.0, 137.5, 9_000.0] {
                assert_eq!(
                    cloud_cell(&f, &TERRAIN, d, t),
                    cloud_cell(&f, &TERRAIN, d, t),
                    "the same column at the same time must give the same weather"
                );
            }
        }
    }

    #[test]
    fn cover_and_alpha_stay_in_range_and_alpha_never_reaches_zero() {
        let f = WeatherField::DEFAULT;
        for d in dirs(400) {
            for t in [0.0, 600.0, 4_321.0] {
                let cell = cloud_cell(&f, &TERRAIN, d, t);
                assert!((0.0..=1.0).contains(&cell.cover), "cover {}", cell.cover);
                assert!((0.0..=1.0).contains(&cell.alpha));
                // The faintest cloud is still visible; a cloud at zero opacity
                // is a cloud nobody can tell from clear sky.
                assert!(cell.alpha >= f.min_alpha - 1e-6);
            }
        }
    }

    #[test]
    fn rain_needs_cover_now_or_a_trail_time_ago() {
        let f = WeatherField::DEFAULT;
        let mut trailed = 0;
        for d in dirs(600) {
            for t in [200.0, 5_000.0] {
                let cell = cloud_cell(&f, &TERRAIN, d, t);
                if !cell.raining {
                    continue;
                }
                if cell.cover >= f.rain_cover_min {
                    continue;
                }
                // Raining under a thin sky is only allowed on the trail, and
                // the trail has to be real: the field a trail-time ago was over
                // the threshold.
                let past = cloud_density(&f, &TERRAIN, d, t - f.rain_min_s);
                assert!(
                    smoothstep(f.cloud_min, f.cloud_full, past) >= f.rain_cover_min,
                    "rain under thin cover must be a trail"
                );
                trailed += 1;
            }
        }
        assert!(trailed > 0, "the trail must actually happen somewhere");
    }

    #[test]
    fn no_trail_means_rain_stops_with_the_cover() {
        let f = WeatherField {
            rain_min_s: 0.0,
            ..WeatherField::DEFAULT
        };
        for d in dirs(400) {
            let cell = cloud_cell(&f, &TERRAIN, d, 1_000.0);
            assert_eq!(cell.raining, cell.cover >= f.rain_cover_min);
        }
    }

    #[test]
    fn a_boost_of_one_rains_everywhere_and_zero_is_the_natural_field() {
        let natural = WeatherField::DEFAULT;
        let forced = WeatherField {
            moisture_boost: 1.0,
            ..natural
        };
        let mut natural_dry = 0;
        for d in dirs(200) {
            assert!(
                cloud_cell(&forced, &TERRAIN, d, 300.0).raining,
                "a full boost must storm the whole planet"
            );
            if !cloud_cell(&natural, &TERRAIN, d, 300.0).raining {
                natural_dry += 1;
            }
        }
        assert!(
            natural_dry > 0,
            "and the natural field must leave somewhere dry"
        );
    }

    #[test]
    fn the_weather_moves_so_a_column_is_not_always_raining() {
        let f = WeatherField::DEFAULT;
        // Columns watched for an hour: the warmth drifts over them, so the
        // weather at a place must CHANGE. A field that never arrives or never
        // leaves is a global switch with extra steps.
        //
        // The claim is over the sphere rather than about one column, and that
        // is the field being honest rather than the test being tuned. Only the
        // warmth moves; moisture is fixed. So an arid column never rains
        // however long you watch, which is what the aridity gamma is for, and
        // the wettest column on the body rains for all six hundred samples,
        // which is what a rainforest is. Measured, the whole body averages 0.18
        // cover with 5.9% of it precipitating at any moment. What must be true
        // is that plenty of places in between see a front come and go.
        let hour: Vec<f32> = (0..120).map(|step| step as f32 * 30.0).collect();
        let mut changing = 0;
        let columns = dirs(400);
        for column in &columns {
            let mut wet = 0;
            for t in &hour {
                if cloud_cell(&f, &TERRAIN, *column, *t).raining {
                    wet += 1;
                }
            }
            if wet > 0 && wet < hour.len() {
                changing += 1;
            }
        }
        assert!(
            changing * 20 > columns.len(),
            "only {changing} of {} columns saw the weather change in an hour",
            columns.len()
        );
    }

    #[test]
    fn walking_far_enough_leaves_the_storm() {
        let f = WeatherField::DEFAULT;
        // Hold time still and walk: somewhere on the body there must be a pair
        // of columns, one raining and one not, within a few kilometres. If the
        // field had no spatial structure this could not happen.
        let t = 2_400.0;
        let raining: Vec<Vec3> = dirs(3_000)
            .into_iter()
            .filter(|d| cloud_cell(&f, &TERRAIN, *d, t).raining)
            .collect();
        assert!(!raining.is_empty(), "somewhere must be raining");
        let start = raining[0];
        // 4 km on a 4,800 m body is a long walk but well short of a hemisphere.
        let arc = 4_000.0 / 4_800.0;
        let mut escaped = false;
        for step in 1..=40 {
            let angle = arc * step as f32 / 40.0;
            let axis = start.cross(Vec3::Y).normalize_or_zero();
            let moved = (start * angle.cos() + axis * angle.sin()).normalize();
            if !cloud_cell(&f, &TERRAIN, moved, t).raining {
                escaped = true;
                break;
            }
        }
        assert!(escaped, "a storm must have an edge you can walk out of");
    }

    #[test]
    fn snow_falls_over_the_cold_biomes_and_nowhere_else() {
        for d in dirs(500) {
            let cold = matches!(
                planet_gen::biome(&TERRAIN, d),
                Biome::Tundra | Biome::Mountains
            );
            assert_eq!(precip_kind(&TERRAIN, d) == Precip::Snow, cold);
        }
    }

    #[test]
    fn arid_ground_carries_less_cloud_than_wet_ground() {
        let f = WeatherField::DEFAULT;
        // Averaged over the sphere and over time, the drier half of the
        // weather-moisture field must cloud less than the wetter half. This is
        // the aridity gamma's whole job, and a per-column assertion would only
        // measure the warmth field that multiplies it.
        let mut dry = (0.0, 0);
        let mut wet = (0.0, 0);
        for d in dirs(2_000) {
            let m = weather_moisture(&f, &TERRAIN, d);
            for t in [0.0, 1_500.0, 3_000.0] {
                let density = cloud_density(&f, &TERRAIN, d, t);
                if m < 0.45 {
                    dry = (dry.0 + density, dry.1 + 1);
                } else if m > 0.55 {
                    wet = (wet.0 + density, wet.1 + 1);
                }
            }
        }
        assert!(dry.1 > 100 && wet.1 > 100, "need both halves sampled");
        let (dry, wet) = (dry.0 / dry.1 as f32, wet.0 / wet.1 as f32);
        assert!(dry < wet * 0.8, "dry {dry:.3} against wet {wet:.3}");
    }

    #[test]
    #[ignore = "a probe: cargo test -p pbd-core spawn_weather -- --ignored --nocapture"]
    fn spawn_weather() {
        // The desktop spawn direction, and a desert one for contrast.
        let f = WeatherField::DEFAULT;
        let spawn = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        for (name, d) in [("spawn", spawn)] {
            for t in [0.0, 300.0, 900.0] {
                let cell = cloud_cell(&f, &TERRAIN, d, t);
                println!(
                    "{name} t={t:>5.0}  moisture {:.3}  cover {:.3}  raining {}  biome {:?}",
                    weather_moisture(&f, &TERRAIN, d),
                    cell.cover,
                    cell.raining,
                    planet_gen::biome(&TERRAIN, d)
                );
            }
        }
        // And the spread over the sphere, so a single column is in context.
        let mut buckets = [0usize; 5];
        for d in dirs(2_000) {
            let c = cloud_cell(&f, &TERRAIN, d, 300.0).cover;
            buckets[((c * 4.999) as usize).min(4)] += 1;
        }
        println!("cover histogram (0-.2,.2-.4,.4-.6,.6-.8,.8-1): {buckets:?}");
    }

    #[test]
    #[ignore = "a report, not a check: cargo test -p pbd-core weather_report -- --ignored --nocapture"]
    fn weather_report() {
        let f = WeatherField::DEFAULT;
        for t in [0.0, 1_800.0, 3_600.0] {
            let (mut cover, mut raining, mut snowing) = (0.0f32, 0, 0);
            let sample = dirs(4_000);
            for d in &sample {
                let cell = cloud_cell(&f, &TERRAIN, *d, t);
                cover += cell.cover;
                if cell.raining {
                    raining += 1;
                    if cell.precip == Precip::Snow {
                        snowing += 1;
                    }
                }
            }
            let n = sample.len() as f32;
            println!(
                "t={t:>6.0}s  mean cover {:.3}  precipitating {:.1}%  of which snow {:.1}%",
                cover / n,
                100.0 * raining as f32 / n,
                if raining > 0 {
                    100.0 * snowing as f32 / raining as f32
                } else {
                    0.0
                }
            );
        }
    }
}
