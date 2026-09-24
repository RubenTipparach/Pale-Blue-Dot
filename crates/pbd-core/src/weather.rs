//! What the weather is at a place, read off the simulated atmosphere.
//!
//! The rain, the cloud and what falls come from `crate::atmosphere`, a
//! simulation stepped on the planet's cells. This module is its read side, the
//! one seam every consumer calls: the player's weather, the rain lattice, the
//! rain map the renderer marches, and the flicker of a strike. It used to be a
//! stateless field of drifting noise (Tenebris's `weather.rs`); what is left of
//! that is `solar`, the drifting noise the atmosphere uses for weather finer
//! than a cell.

use crate::atmosphere::Atmosphere;
use crate::planet_gen;
use glam::Vec3;

/// What falls under a raining column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precip {
    None,
    Rain,
    Snow,
}

/// The drifting noise's knobs: how big its pockets are and how fast they move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherField {
    /// Unit-sphere frequency: higher is smaller and more numerous pockets.
    pub solar_scale: f32,
    /// How fast the pockets drift and evolve, per second.
    pub solar_drift: f32,
}

impl WeatherField {
    pub const DEFAULT: WeatherField = WeatherField {
        solar_scale: 2.2,
        solar_drift: 0.015,
    };
}

impl Default for WeatherField {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Salt so the pockets are their own stream rather than the terrain's.
const SOLAR_SEED_SALT: u64 = 0x501a_125e_ed00_0001;

/// A drifting field of pockets in [0, 1], two octaves for shape and detail,
/// on the gradient noise the terrain is built on.
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

/// One column's weather.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloudCell {
    /// Cover in [0, 1]: nothing at zero, a full mass at one.
    pub cover: f32,
    /// Whether this column is precipitating.
    pub raining: bool,
    /// What falls, meaningful only when `raining`.
    pub precip: Precip,
}

/// Sample the whole of one column's weather.
pub fn cloud_cell(atmosphere: &Atmosphere, direction: Vec3) -> CloudCell {
    let sample = atmosphere.sample(direction);
    let raining = sample.rain_rate >= atmosphere.settings.raining_rate;
    CloudCell {
        cover: sample.cover,
        raining,
        precip: match (raining, sample.snow) {
            (false, _) => Precip::None,
            (true, true) => Precip::Snow,
            (true, false) => Precip::Rain,
        },
    }
}

/// How hard it is raining in [0, 1] at a column: the cover where it rains and
/// nothing where it does not. This is the ONE number every rain effect reads.
pub fn rain_at(atmosphere: &Atmosphere, direction: Vec3) -> f32 {
    let cell = cloud_cell(atmosphere, direction);
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
/// shape Tenebris's distant shafts use, so a shaft belongs to a place and does
/// not slide as the camera moves; which cells rain is the weather's answer and
/// changes with time, the cells themselves never do.
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
/// now, which is where a shaft is drawn: the weather's own answer, sampled at
/// each cell's centre, so a shaft stands exactly where it rains.
pub fn raining_cells(
    atmosphere: &Atmosphere,
    eye: Vec3,
    cell_angle: f32,
    range_angle: f32,
) -> Vec<RainCell> {
    let mut cells = Vec::new();
    lattice_near(eye, cell_angle, range_angle, |row, column, direction| {
        let cell = cloud_cell(atmosphere, direction);
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
/// rain intensity at its centre, NEGATIVE where it snows, zero where it is
/// dry. `u` and `v` are the plane's unit axes.
///
/// A point is carried onto the plane gnomonically (`anchor * radius + u * x +
/// v * y`, normalised), which is also how a shader reads the map back.
pub fn precipitation_map(
    atmosphere: &Atmosphere,
    anchor: Vec3,
    u: Vec3,
    v: Vec3,
    radius: f32,
    size: usize,
    cell_m: f32,
) -> Vec<f32> {
    let mut map = Vec::with_capacity(size * size);
    let half = size as f32 * 0.5;
    for row in 0..size {
        for column in 0..size {
            let x = (column as f32 + 0.5 - half) * cell_m;
            let y = (row as f32 + 0.5 - half) * cell_m;
            let direction = (anchor * radius + u * x + v * y).normalize();
            let cell = cloud_cell(atmosphere, direction);
            map.push(match (cell.raining, cell.precip) {
                (true, Precip::Snow) => -cell.cover,
                (true, _) => cell.cover,
                _ => 0.0,
            });
        }
    }
    map
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
    use crate::atmosphere::{AtmosphereSettings, Forcing};
    use crate::planet_gen::TerrainConfig;

    /// A small atmosphere with a storm brewed at `storm`.
    fn stormy(storm: Vec3) -> Atmosphere {
        let settings = AtmosphereSettings {
            level: 3,
            forcing_radius_m: 1500.0,
            ..Default::default()
        };
        let mut air = Atmosphere::new(&TerrainConfig::TENEBRIS, settings, 7);
        let sun = Vec3::new(0.8, 0.4, 0.45);
        for _ in 0..60 {
            air.step(
                sun,
                &[Forcing {
                    direction: storm,
                    strength: 1.0,
                }],
            );
        }
        air
    }

    #[test]
    fn rain_is_drawn_on_every_raining_cell_in_view_and_no_dry_one() {
        let eye = Vec3::new(0.3, 0.8, 0.2).normalize();
        let air = stormy(eye);
        let (cell, range) = (60.0 / 4800.0, 1400.0 / 4800.0);
        let drawn = raining_cells(&air, eye, cell, range);
        let mut expected = Vec::new();
        lattice_near(eye, cell, std::f32::consts::PI, |row, column, direction| {
            if direction.dot(eye) >= range.cos() && cloud_cell(&air, direction).raining {
                expected.push((row, column));
            }
        });
        let got: Vec<_> = drawn.iter().map(|c| (c.row, c.column)).collect();
        assert_eq!(
            got, expected,
            "the shafts must be exactly the raining cells"
        );
        assert!(
            expected.len() > 20,
            "a brewed storm rains in view: {}",
            expected.len()
        );
    }

    #[test]
    fn the_rain_lattice_is_fixed_to_the_body() {
        // The same cell seen from two eyes is the same place, so a shaft
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
    }

    #[test]
    fn the_precipitation_map_is_the_weather_at_each_cell() {
        let anchor = Vec3::new(0.8772, 0.4801, 0.0).normalize();
        let air = stormy(anchor);
        let u = anchor.any_orthonormal_vector();
        let v = anchor.cross(u);
        let (size, cell, radius) = (16, 50.0, 4800.0);
        let map = precipitation_map(&air, anchor, u, v, radius, size, cell);
        assert_eq!(map.len(), size * size);
        let mut wet = 0;
        for row in 0..size {
            for column in 0..size {
                let x = (column as f32 + 0.5 - 8.0) * cell;
                let y = (row as f32 + 0.5 - 8.0) * cell;
                let d = (anchor * radius + u * x + v * y).normalize();
                let c = cloud_cell(&air, d);
                let want = match (c.raining, c.precip) {
                    (false, _) => 0.0,
                    (true, Precip::Snow) => -c.cover,
                    (true, _) => c.cover,
                };
                assert_eq!(map[row * size + column], want);
                wet += usize::from(want != 0.0);
            }
        }
        assert!(wet > 0, "the storm shows on the map");
    }

    #[test]
    fn walking_far_enough_leaves_the_storm() {
        let here = Vec3::new(-0.2, 0.3, 0.9).normalize();
        let air = stormy(here);
        assert!(
            rain_at(&air, here) > 0.0,
            "it rains where the storm was brewed"
        );
        assert_eq!(
            rain_at(&air, -here),
            0.0,
            "and not on the far side of the planet"
        );
    }

    #[test]
    fn a_flash_has_three_strokes_and_ends_dark() {
        assert!(flicker(0.08) > 0.95);
        assert!(flicker(0.25) < 0.2, "dark between strokes");
        assert!(flicker(0.62) > 0.7);
        assert!(flicker(1.0) < 0.01);
    }
}
