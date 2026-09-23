//! The atmosphere and the ocean: two fluids on the planet's cells.
//!
//! One deterministic simulation is the source of cloud, rain, snow, wind,
//! ocean currents and lightning. The sun heats the ground (the sea slowly,
//! the land quickly); warm air lowers the pressure and air flows from high to
//! low, turned by the spin; the sea evaporates, the wind carries the vapour,
//! and where air converges and rises it condenses, releasing heat that makes
//! it rise harder; thick cloud rains; storms build charge and strike, and the
//! cold outflow under a strike lifts the air around it into the next storm.
//! The ocean is the same fluid on the sea cells, driven by the wind and walled
//! by the coasts.
//!
//! It is stepped at a fixed step in a fixed order, with no hash map and no
//! randomness but a hash of the cell and the step, so the same seed and the
//! same steps give the same state on a build and a platform. Across platforms
//! the maths library's last bits may differ, which is why a future server owns
//! the weather and sends it; the state is saved with the world.
//! See `openspec/changes/atmospheric-circulation`.

pub mod grid;
mod numeric;
mod ocean;
pub mod settings;
mod step;
mod storms;
#[cfg(test)]
mod tests;

use crate::planet_gen::{self, Biome, TerrainConfig};
use glam::Vec3;
pub use grid::Grid;
use grid::{weighted, weighted_vec};
pub use numeric::smoothstep;
pub use settings::AtmosphereSettings;
use std::sync::Arc;

/// A place a player is forcing a storm, with the slider's strength in 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forcing {
    pub direction: Vec3,
    pub strength: f32,
}

/// A lightning strike the renderer can draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    /// Where it struck, a unit direction.
    pub direction: Vec3,
    /// The step it struck on.
    pub step: u64,
    /// How hard, 0..1: how far past the threshold the cell's charge was.
    pub strength: f32,
}

/// What the ground under each cell is. Fixed for a world: derived from the
/// terrain generator, which is a pure function of the seed.
#[derive(Clone, Debug)]
pub struct Surface {
    pub ocean: Vec<bool>,
    /// Height above sea level, metres (0 over the sea).
    pub elevation: Vec<f32>,
    /// Slope of that height, metres per metre, in the tangent plane.
    pub slope: Vec<Vec3>,
    /// How freely it gives up water: 1 over the sea, from the biome on land.
    pub wetness: Vec<f32>,
    pub albedo: Vec<f32>,
    pub heat_capacity: Vec<f32>,
}

/// Everything at one point, interpolated from the cells around it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sample {
    /// Cloud cover, 0..1.
    pub cover: f32,
    /// Condensed water in the column, kg/m^2.
    pub cloud: f32,
    /// How tall the cloud towers, 0..1 of the tallest.
    pub cloud_top: f32,
    /// Precipitation, kg/m^2/s (mm/s).
    pub rain_rate: f32,
    /// Whether that falls as snow.
    pub snow: bool,
    /// Surface wind and the cloud-level wind, m/s, tangent.
    pub wind: Vec3,
    pub upper: Vec3,
    /// Ocean current, m/s; zero over land.
    pub current: Vec3,
    /// Relative humidity at the surface, 0..1.
    pub humidity: f32,
    /// Sunlight reaching the ground, W/m^2.
    pub sunlight: f32,
    /// Surface temperature, deg C, at the ground's own height.
    pub temperature: f32,
    /// Optical depth of the column's cloud, for lighting it.
    pub optical_depth: f32,
}

#[derive(Clone, Debug)]
pub struct Atmosphere {
    /// The cells, shared: they never change, so a copy of the weather taken
    /// to step elsewhere copies only the state.
    pub grid: Arc<Grid>,
    pub settings: AtmosphereSettings,
    pub surface: Arc<Surface>,
    seed: u64,
    /// Steps taken since the world began.
    pub step: u64,
    // --- State, one entry per cell ---
    /// Pressure anomaly as geopotential, m^2/s^2.
    pub phi: Vec<f32>,
    /// Surface wind, m/s, tangent.
    pub wind: Vec<Vec3>,
    /// Air temperature, deg C.
    pub air_k: Vec<f32>,
    /// Ground temperature at sea level, deg C; over the sea, the sea surface.
    pub ground_k: Vec<f32>,
    /// Water vapour in the column, kg/m^2.
    pub vapour: Vec<f32>,
    /// Condensed water in the column, kg/m^2.
    pub cloud: Vec<f32>,
    /// Electrification, in units of the strike threshold.
    pub charge: Vec<f32>,
    /// Sea-surface height as geopotential, m^2/s^2; zero on land.
    pub eta: Vec<f32>,
    /// Surface current, m/s; zero on land.
    pub current: Vec<Vec3>,
    // --- What the last step worked out, for sampling and drawing ---
    /// Precipitation, kg/m^2/s.
    pub rain_rate: Vec<f32>,
    /// Ascent out of the boundary layer, m/s.
    pub lift: Vec<f32>,
    /// Wind at cloud height, m/s.
    pub upper: Vec<Vec3>,
    /// Sunlight reaching the ground, W/m^2.
    pub sunlight: Vec<f32>,
    /// Strikes within the last `strike_keep_s`.
    pub strikes: Vec<Strike>,
    /// Variability finer than a cell, -1..1, refreshed now and then.
    mesoscale: Vec<f32>,
}

impl Atmosphere {
    /// A new world's atmosphere: at rest, at its latitude's climate, with the
    /// old stateless field's noise to break the symmetry. Not spun up; call
    /// `spin_up` for weather on the first frame.
    pub fn new(terrain: &TerrainConfig, settings: AtmosphereSettings, seed: u64) -> Self {
        let grid = Grid::new(settings.level, terrain.radius_m);
        let surface = surface_of(&grid, terrain, &settings);
        let n = grid.len();
        let mut atmosphere = Atmosphere {
            seed,
            step: 0,
            phi: vec![0.0; n],
            wind: vec![Vec3::ZERO; n],
            air_k: vec![0.0; n],
            ground_k: vec![0.0; n],
            vapour: vec![0.0; n],
            cloud: vec![0.0; n],
            charge: vec![0.0; n],
            eta: vec![0.0; n],
            current: vec![Vec3::ZERO; n],
            rain_rate: vec![0.0; n],
            lift: vec![0.0; n],
            upper: vec![Vec3::ZERO; n],
            sunlight: vec![0.0; n],
            strikes: Vec::new(),
            mesoscale: vec![0.0; n],
            grid: Arc::new(grid),
            settings,
            surface: Arc::new(surface),
        };
        let field = crate::weather::WeatherField::DEFAULT;
        for cell in 0..n {
            let c = atmosphere.grid.centre[cell];
            let climate = climate_k(c.y);
            // The old field's warm pockets, as a few kelvin of disturbance.
            let pocket = crate::weather::solar(&field, seed, c, 0.0) - 0.5;
            atmosphere.ground_k[cell] = climate + pocket * 4.0;
            atmosphere.air_k[cell] = atmosphere.ground_k[cell];
            let saturated = atmosphere.saturation(atmosphere.ground_k[cell]);
            atmosphere.vapour[cell] = saturated * 0.5 * atmosphere.surface.wetness[cell].max(0.3);
        }
        atmosphere.refresh_mesoscale();
        atmosphere
    }

    /// Run the weather for `spinup_s` so a new world opens on weather rather
    /// than on a planet at rest. `sun` gives the sun for a world time.
    pub fn spin_up(&mut self, mut sun: impl FnMut(f64) -> Vec3, start_seconds: f64) {
        let steps = (self.settings.spinup_s / self.settings.dt_s).ceil() as u64;
        for i in 0..steps {
            let t = start_seconds - (steps - i) as f64 * self.settings.dt_s as f64;
            self.step(sun(t), &[]);
        }
    }

    /// The world seed the strikes are hashed off.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// A column's saturated water, kg/m^2, at an air temperature.
    pub fn saturation(&self, k: f32) -> f32 {
        self.settings.saturation_kg * numeric::exp(self.settings.saturation_per_k * (k - 15.0))
    }

    /// Cover for a column's cloud water.
    pub fn cover_of(&self, cloud: f32) -> f32 {
        smoothstep(
            self.settings.cover_min_kg,
            self.settings.cover_full_kg,
            cloud,
        )
    }

    /// Everything at a direction.
    pub fn sample(&self, direction: Vec3) -> Sample {
        let (cells, w) = self.grid.locate(direction);
        let cloud = weighted(&self.cloud, cells, w).max(0.0);
        let lift = weighted(&self.lift, cells, w).max(0.0);
        let air = weighted(&self.air_k, cells, w);
        let elevation = weighted(&self.surface.elevation, cells, w);
        let ground = weighted(&self.ground_k, cells, w) - self.settings.lapse_k_per_m * elevation;
        let vapour = weighted(&self.vapour, cells, w).max(0.0);
        let ocean_share: f32 = cells
            .iter()
            .zip(w)
            .map(|(&c, w)| {
                if self.surface.ocean[c as usize] {
                    w
                } else {
                    0.0
                }
            })
            .sum();
        Sample {
            cover: self.cover_of(cloud),
            cloud,
            cloud_top: self.cloud_top(cloud, lift),
            rain_rate: weighted(&self.rain_rate, cells, w).max(0.0),
            snow: ground < 0.0,
            wind: weighted_vec(&self.wind, cells, w),
            upper: weighted_vec(&self.upper, cells, w),
            current: if ocean_share > 0.5 {
                weighted_vec(&self.current, cells, w)
            } else {
                Vec3::ZERO
            },
            humidity: (vapour / self.saturation(air - self.settings.lapse_k_per_m * elevation))
                .clamp(0.0, 1.0),
            sunlight: weighted(&self.sunlight, cells, w).max(0.0),
            temperature: ground,
            optical_depth: cloud * 20.0,
        }
    }

    /// How tall a cloud stands, 0..1: a thick column of rising air towers, a
    /// thin sheet over sinking air lies flat.
    pub fn cloud_top(&self, cloud: f32, lift: f32) -> f32 {
        let convective = smoothstep(0.0, 1.5, lift) * smoothstep(0.1, 0.8, cloud);
        0.3 + 0.7 * convective
    }

    /// Total water in the air (vapour and cloud), kg, area-weighted.
    pub fn water_kg(&self) -> f64 {
        (0..self.grid.len())
            .map(|i| ((self.vapour[i] + self.cloud[i]) * self.grid.area[i]) as f64)
            .sum()
    }

    /// Bit-exact state for the save: the prognostic fields in a fixed order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let n = self.grid.len();
        let mut out = Vec::with_capacity(16 + n * 4 * 14);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.settings.level.to_le_bytes());
        out.extend_from_slice(&(n as u32).to_le_bytes());
        out.extend_from_slice(&self.step.to_le_bytes());
        for field in self.scalars() {
            for v in field {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        for field in [&self.wind, &self.current] {
            for v in field {
                for x in v.to_array() {
                    out.extend_from_slice(&x.to_le_bytes());
                }
            }
        }
        out
    }

    /// Restore what `to_bytes` wrote, onto an atmosphere built for the same
    /// world and level. Refuses anything else, whole, leaving `self` as it was.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), String> {
        let n = self.grid.len();
        let want = MAGIC.len() + 16 + n * 4 * (SCALARS + 6);
        if bytes.len() != want || &bytes[..MAGIC.len()] != MAGIC {
            return Err(format!(
                "weather state is {} bytes, expected {want}",
                bytes.len()
            ));
        }
        let mut at = MAGIC.len();
        let word = |at: &mut usize| {
            let v = u32::from_le_bytes(bytes[*at..*at + 4].try_into().expect("four bytes"));
            *at += 4;
            v
        };
        if word(&mut at) != self.settings.level || word(&mut at) as usize != n {
            return Err("weather state is for another grid".into());
        }
        let step = u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"));
        at += 8;
        let mut floats = bytes[at..]
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().expect("four bytes")));
        let mut read = |len: usize| -> Vec<f32> { (&mut floats).take(len).collect() };
        let scalars: Vec<Vec<f32>> = (0..SCALARS).map(|_| read(n)).collect();
        let vectors: Vec<Vec<Vec3>> = (0..2)
            .map(|_| read(n * 3).chunks_exact(3).map(Vec3::from_slice).collect())
            .collect();
        if scalars.iter().flatten().any(|v| !v.is_finite())
            || vectors.iter().flatten().any(|v| !v.is_finite())
        {
            return Err("weather state holds a non-finite value".into());
        }
        let mut scalars = scalars.into_iter();
        for field in [
            &mut self.phi,
            &mut self.air_k,
            &mut self.ground_k,
            &mut self.vapour,
            &mut self.cloud,
            &mut self.charge,
            &mut self.eta,
        ] {
            *field = scalars.next().expect("seven fields");
        }
        let mut vectors = vectors.into_iter();
        self.wind = vectors.next().expect("wind");
        self.current = vectors.next().expect("current");
        self.step = step;
        Ok(())
    }

    fn scalars(&self) -> [&Vec<f32>; SCALARS] {
        [
            &self.phi,
            &self.air_k,
            &self.ground_k,
            &self.vapour,
            &self.cloud,
            &self.charge,
            &self.eta,
        ]
    }
}

const MAGIC: &[u8; 8] = b"PBDATM01";
const SCALARS: usize = 7;

/// A latitude's rough year-round temperature at sea level, deg C, from the
/// sine of the latitude: where a new world's weather starts.
fn climate_k(sin_latitude: f32) -> f32 {
    28.0 - 45.0 * sin_latitude * sin_latitude
}

fn surface_of(grid: &Grid, terrain: &TerrainConfig, settings: &AtmosphereSettings) -> Surface {
    let n = grid.len();
    let mut surface = Surface {
        ocean: Vec::with_capacity(n),
        elevation: Vec::with_capacity(n),
        slope: vec![Vec3::ZERO; n],
        wetness: Vec::with_capacity(n),
        albedo: Vec::with_capacity(n),
        heat_capacity: Vec::with_capacity(n),
    };
    for &c in &grid.centre {
        let height = planet_gen::surface_altitude(terrain, c);
        let ocean = height < terrain.sea_level_m;
        let biome = planet_gen::biome_at(terrain, c, height);
        let (wetness, albedo) = match biome {
            Biome::Ocean => (1.0, settings.ocean_albedo),
            Biome::Swamp => (1.0, settings.land_albedo),
            Biome::Jungle => (0.9, settings.land_albedo * 0.8),
            Biome::Beach => (0.7, settings.land_albedo * 1.2),
            Biome::Fields => (0.6, settings.land_albedo),
            Biome::Mountains => (0.3, settings.land_albedo * 1.2),
            Biome::Tundra => (0.3, settings.snow_albedo),
            Biome::Desert => (0.1, settings.land_albedo * 1.4),
        };
        surface.ocean.push(ocean);
        surface
            .elevation
            .push((height - terrain.sea_level_m).max(0.0));
        surface.wetness.push(if ocean { 1.0 } else { wetness });
        surface.albedo.push(if ocean {
            settings.ocean_albedo
        } else {
            albedo.min(1.0)
        });
        surface.heat_capacity.push(if ocean {
            settings.ocean_heat_capacity
        } else {
            settings.land_heat_capacity
        });
    }
    for cell in 0..n {
        surface.slope[cell] = grid.gradient(&surface.elevation, cell, |_| false);
    }
    surface
}
