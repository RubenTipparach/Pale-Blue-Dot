//! Spherical terrain generator: Tenebris's `planet_gen.rs`, ported term for
//! term at its unit-sphere scales, with every height re-authored in metres
//! against this body's budget (`openspec/changes/tenebris-terrain`).
//!
//! The order of the altitude terms is the shape: a sharpened continent
//! field, ridged mountains multiplied by the land so they stand inland, hills,
//! detail, an ocean floor that deepens past the beach, then islands, rivers,
//! eased shorelines and rocky highlands. The biome is one classification of a
//! direction that the top block, the trees and the scatter all read.
//!
//! Directions are unit vectors on the body. Everything is `f32` in the same
//! order of operations as the reference, so a sample can be checked against
//! it bit for bit. Changing any of it changes every world: bump
//! [`crate::terrain::GENERATOR_VERSION`] with it.

use glam::Vec3;

use crate::terrain::Material;

/// Every tunable of the generator, with units. One source of defaults, so a
/// second body is a second value of this struct and never a second function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainConfig {
    /// World seed; folded into every noise field, each with its own salt.
    pub seed: u64,
    /// The body this config is authored for. A land-scale field's frequency
    /// is derived from it, so the same config on a bigger body makes MORE
    /// hills rather than bigger ones.
    pub radius_m: f32,
    /// PLANET-SCALE: a unit-sphere frequency, so the feature is an ANGLE and
    /// a bigger body has the same few of them. A world has a handful of
    /// continents whatever its radius.
    pub continent_scale: f32,
    /// LAND-SCALE: the coarsest feature in METRES, so the feature is a size a
    /// player walks over. The reference's own metres, except the mountain,
    /// which is matched by SLOPE instead: fully metric it would be 120 m, and
    /// our taller ridge across a 120 m gap is a wall rather than a mountain.
    pub mountain_m: f32,
    pub hill_m: f32,
    pub detail_m: f32,
    /// Weights of the planet-scale fields in the normalised height.
    pub continent_weight: f32,
    pub mountain_height: f32,
    /// The land-scale amplitudes, in METRES rather than as a share of the
    /// relief budget, so the summit and the roughness underfoot move apart.
    pub hill_amplitude_m: f32,
    pub detail_amplitude_m: f32,
    /// Pushes the continent field toward land; the reference's 0.20 less its
    /// seeded-world trim of 0.16.
    pub land_bias: f32,
    /// Exponent on the ocean side of the normalised height: under one, the
    /// floor drops away fast past the beach.
    pub ocean_depth_power: f32,
    /// Metres one unit of normalised height is worth above the sea. The
    /// fields sum to well under one, so the summit is about half of this;
    /// the reference's 40 reaches 34 with its uplift.
    pub land_scale_m: f32,
    /// Metres one unit of normalised depth is worth below the sea; the floor
    /// reaches about a quarter of it.
    pub ocean_scale_m: f32,
    /// Sea level, metres above the base radius.
    pub sea_level_m: f32,
    /// Half-width of the sand band about the waterline, metres.
    pub beach_band_m: f32,
    /// Islands: a low-frequency field lifts land out of shallow sea.
    pub island_scale: f32,
    pub island_threshold: f32,
    pub island_height_m: f32,
    /// Rivers: a ridged iso-line pulled to a bed below the sea, on lowland only.
    pub river_m: f32,
    pub river_threshold: f32,
    pub river_depth_m: f32,
    pub river_max_elev_m: f32,
    /// Shoreline easing: within the band the slope is cut by the flatten
    /// factor at the waterline, easing back to full at the band's edge.
    pub shore_band_m: f32,
    pub shore_flatten: f32,
    /// Rocky highlands: a region field lifts whole areas above the threshold.
    /// Planet-scale: a mountain COUNTRY is a region of the world.
    pub rocky_scale: f32,
    pub rocky_above: f32,
    pub rocky_uplift_m: f32,
    /// Moisture field and the biome thresholds on it.
    pub moisture_m: f32,
    pub desert_below: f32,
    pub wet_above: f32,
    pub swamp_max_elev_m: f32,
    /// Above this the biome is Mountains; above the snowcap its top is snow.
    pub mountain_elev_m: f32,
    pub mountain_snowcap_elev_m: f32,
    /// Temperate hills: bare stone above the snow line, snow above the stone
    /// line; the cold band halves its snow line.
    pub snow_line_m: f32,
    pub stone_line_m: f32,
    /// |latitude| (as |y|) above which the surface is Tundra, and polar.
    pub cold_latitude: f32,
    pub polar_latitude: f32,
    /// Fraction of desert tops that break into rock.
    pub desert_rock_frac: f32,
}

impl TerrainConfig {
    /// The main body's generator: the reference's scales and thresholds, the
    /// heights re-authored for a 4,800 m body (summits near 150 m, the floor
    /// near 90 m, about half the sphere land, measured).
    pub const TENEBRIS: Self = Self {
        seed: 0x5eed_2026,
        radius_m: 4_800.0,
        continent_scale: 0.8,
        mountain_m: 686.0,
        hill_m: 60.0,
        detail_m: 25.0,
        continent_weight: 0.7,
        mountain_height: 0.8,
        hill_amplitude_m: 6.0,
        detail_amplitude_m: 2.0,
        land_bias: 0.0,
        ocean_depth_power: 1.1,
        land_scale_m: 210.0,
        ocean_scale_m: 320.0,
        sea_level_m: 0.0,
        beach_band_m: 2.0,
        island_scale: 4.0,
        island_threshold: 0.68,
        island_height_m: 12.0,
        river_m: 231.0,
        river_threshold: 0.86,
        river_depth_m: 3.0,
        river_max_elev_m: 40.0,
        shore_band_m: 6.0,
        shore_flatten: 0.5,
        rocky_scale: 2.0,
        rocky_above: 0.72,
        rocky_uplift_m: 60.0,
        moisture_m: 188.0,
        desert_below: 0.36,
        wet_above: 0.64,
        swamp_max_elev_m: 5.0,
        mountain_elev_m: 105.0,
        mountain_snowcap_elev_m: 150.0,
        snow_line_m: 100.0,
        stone_line_m: 140.0,
        cold_latitude: 0.875,
        polar_latitude: 0.95,
        desert_rock_frac: 0.14,
    };
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self::TENEBRIS
    }
}

impl TerrainConfig {
    /// The unit-sphere frequency a land-scale field runs at: a feature of
    /// `metres` on this body. This one line is the whole of the planet-scale
    /// / land-scale distinction.
    pub fn scale_of(&self, metres: f32) -> f32 {
        self.radius_m / metres.max(1.0)
    }
}

/// Salts that decorrelate the seeded fields from the elevation octaves. The
/// reference's values; part of the generator's identity, not tunables.
const MOISTURE_SEED_SALT: u64 = 0xB10E_5EED;
const ROCKY_SEED_SALT: u64 = 0x0C0C_5EED;
const RIVER_SEED_SALT: u64 = 0x217E_5EED;
const ISLAND_SEED_SALT: u64 = 0x151A_D5ED;

// ---- The noise primitive: 3D gradient noise, seeded, bit for bit ----------

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn hash(x: i32, y: i32, z: i32, seed: i32) -> i32 {
    let mut h =
        seed ^ x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ z.wrapping_mul(1274126177);
    h = (h ^ (((h as u32) >> 13) as i32)).wrapping_mul(1103515245);
    h ^= ((h as u32) >> 16) as i32;
    h
}

fn grad(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 != 0 { -u } else { u }) + (if h & 2 != 0 { -v } else { v })
}

/// Seeded 3D gradient noise in [-1, 1]: the reference's `gnoise3d_seed`.
pub fn gradient_noise(seed: u64, x: f32, y: f32, z: f32) -> f32 {
    let s = (((seed & 0xFFFF_FFFF) as u32) ^ (((seed >> 32) & 0xFFFF_FFFF) as u32)) as i32;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let fz = z - iz as f32;
    let u = fade(fx);
    let v = fade(fy);
    let w = fade(fz);
    let g000 = grad(hash(ix, iy, iz, s), fx, fy, fz);
    let g100 = grad(hash(ix + 1, iy, iz, s), fx - 1.0, fy, fz);
    let g010 = grad(hash(ix, iy + 1, iz, s), fx, fy - 1.0, fz);
    let g110 = grad(hash(ix + 1, iy + 1, iz, s), fx - 1.0, fy - 1.0, fz);
    let g001 = grad(hash(ix, iy, iz + 1, s), fx, fy, fz - 1.0);
    let g101 = grad(hash(ix + 1, iy, iz + 1, s), fx - 1.0, fy, fz - 1.0);
    let g011 = grad(hash(ix, iy + 1, iz + 1, s), fx, fy - 1.0, fz - 1.0);
    let g111 = grad(
        hash(ix + 1, iy + 1, iz + 1, s),
        fx - 1.0,
        fy - 1.0,
        fz - 1.0,
    );
    let a = lerp(lerp(g000, g100, u), lerp(g010, g110, u), v);
    let b = lerp(lerp(g001, g101, u), lerp(g011, g111, u), v);
    lerp(a, b, w)
}

/// fBm over the gradient noise, the seed stirred per octave, in [-1, 1].
pub fn fractal(
    seed: u64,
    p: Vec3,
    period: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut value = 0.0f32;
    let mut max_value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = period;
    for i in 0..octaves {
        let n = gradient_noise(
            seed.wrapping_add(i as u64 * 1117),
            p.x * frequency,
            p.y * frequency,
            p.z * frequency,
        );
        value += n * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    value / max_value
}

/// Ridged fBm: `(1 - |n|)^2` per octave, so every octave adds a crease and the
/// crests connect, in [0, 1].
pub fn ridged(
    seed: u64,
    p: Vec3,
    period: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut value = 0.0f32;
    let mut max_value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = period;
    for i in 0..octaves {
        let n = gradient_noise(
            seed.wrapping_add(i as u64 * 1117),
            p.x * frequency,
            p.y * frequency,
            p.z * frequency,
        );
        let ridge = 1.0f32 - n.abs();
        value += ridge * ridge * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    value / max_value
}

/// fBm remapped to [0, 1].
pub fn noise01(seed: u64, p: Vec3, scale: f32, octaves: u32) -> f32 {
    (fractal(seed, p, scale, octaves, 0.5, 2.0) * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// ---- The altitude ----------------------------------------------------------

/// Surface altitude in metres above the base radius at a unit direction,
/// unquantised. Negative is sea floor.
pub fn surface_altitude(cfg: &TerrainConfig, direction: Vec3) -> f32 {
    let d = direction.normalize_or(Vec3::Y);
    let seed = cfg.seed;

    // Continental base: six octaves, biased toward land, then sign * |n|^0.8
    // so the land/ocean mask has crisp coasts instead of a gradient of shallows.
    let continent = fractal(seed, d, cfg.continent_scale, 6, 0.5, 2.0) + cfg.land_bias;
    let continent_value = continent.signum() * ((continent as f64).abs().powf(0.8) as f32);

    // Mountain ridges, multiplied by the land so ranges stand inland.
    let mountain_noise = ridged(seed, d, cfg.scale_of(cfg.mountain_m), 4, 0.5, 2.2);
    let land_factor = continent_value.max(0.0);
    let mountain_height = mountain_noise * land_factor * cfg.mountain_height;

    let hill_noise = fractal(seed, d, cfg.scale_of(cfg.hill_m), 3, 0.5, 2.0);
    let detail_noise = fractal(seed, d, cfg.scale_of(cfg.detail_m), 2, 0.5, 2.0);

    let mut height = continent_value * cfg.continent_weight;
    height += mountain_height;
    // The land-scale terms carry metres, so they are divided back out of the
    // budget the normalised height is about to be multiplied by. Raising the
    // summit therefore does not also roughen the ground underfoot.
    height += hill_noise
        * (cfg.hill_amplitude_m / cfg.land_scale_m)
        * if land_factor > 0.1 { 1.0 } else { 0.3 };
    height += detail_noise * (cfg.detail_amplitude_m / cfg.land_scale_m);

    // Normalised height to metres: a power curve on the ocean side so the
    // floor drops away past the beach.
    let mut base = if height >= 0.0 {
        height * cfg.land_scale_m
    } else {
        let ocean_factor = (height as f64).abs().powf(cfg.ocean_depth_power as f64) as f32;
        -ocean_factor * cfg.ocean_scale_m
    };

    let sea = cfg.sea_level_m;

    // Islands rise out of shallow sea; deep basins stay open water.
    if base < sea {
        let island = noise01(seed ^ ISLAND_SEED_SALT, d, cfg.island_scale, 3);
        if island > cfg.island_threshold {
            let t = smooth((island - cfg.island_threshold) / (1.0 - cfg.island_threshold));
            base += t * cfg.island_height_m;
        }
    }

    // Rivers: a ridged iso-line of a mid-frequency field, pulled to a bed
    // under the sea, on lowland only so channels never climb the peaks.
    if base > sea && base < sea + cfg.river_max_elev_m {
        let channel = river_channel(cfg, d);
        if channel > cfg.river_threshold {
            let t = smooth((channel - cfg.river_threshold) / (1.0 - cfg.river_threshold));
            let bed = sea - cfg.river_depth_m;
            base += (bed - base) * t;
        }
    }

    // Gentle shorelines: the slope is compressed toward the sea within the
    // band, so a beach wades in instead of stepping off.
    let offset = base - sea;
    if offset.abs() < cfg.shore_band_m {
        let t = (offset.abs() / cfg.shore_band_m).clamp(0.0, 1.0);
        let keep = 1.0 - cfg.shore_flatten * (1.0 - t);
        base = sea + offset * keep;
    }

    // Rocky highlands lift whole regions, above the beach only, so the
    // Mountains biome has country under it and the shoreline stays put.
    if base > sea + cfg.beach_band_m {
        let r = rockiness(cfg, d);
        if r > cfg.rocky_above {
            let t = smooth(((r - cfg.rocky_above) / (1.0 - cfg.rocky_above)).clamp(0.0, 1.0));
            return base + t * cfg.rocky_uplift_m;
        }
    }
    base
}

/// How strongly the river field carves at a direction, in [0, 1]: the ridged
/// iso-line of a mid-frequency field, which peaks where the raw noise sits
/// mid-range. Above `river_threshold` the carve fires. Public because finding
/// a watercourse by guessing at heights cannot tell one from a shallow bay,
/// and because the altitude and anything looking for a river must agree.
pub fn river_channel(cfg: &TerrainConfig, direction: Vec3) -> f32 {
    let n = noise01(
        cfg.seed ^ RIVER_SEED_SALT,
        direction,
        cfg.scale_of(cfg.river_m),
        4,
    );
    1.0 - (2.0 * n - 1.0).abs()
}

/// The region field that drives the rocky uplift and the Mountains biome, in [0, 1].
pub fn rockiness(cfg: &TerrainConfig, direction: Vec3) -> f32 {
    noise01(cfg.seed ^ ROCKY_SEED_SALT, direction, cfg.rocky_scale, 3)
}

/// Moisture in [0, 1] at a unit direction; 0 is bone dry.
pub fn moisture(cfg: &TerrainConfig, direction: Vec3) -> f32 {
    noise01(
        cfg.seed ^ MOISTURE_SEED_SALT,
        direction,
        cfg.scale_of(cfg.moisture_m),
        4,
    )
}

// ---- The biome ---------------------------------------------------------------

/// One classification of a direction that every layer reads. The
/// discriminants are part of the interface: the record hands them to the
/// shaders, which key the foliage density and the tree's height off them.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Biome {
    Ocean,
    Beach,
    /// Temperate grass.
    Fields,
    /// Dry: sand broken by rock, no trees.
    Desert,
    /// Wet and warm: dense tall trees.
    Jungle,
    /// Wet lowland: groves, mud and pools.
    Swamp,
    /// High rock above the tree line.
    Mountains,
    /// The cold high-latitude band.
    Tundra,
}

/// Classify a direction. Elevation and latitude bands first (ocean, beach,
/// tundra, mountains), then the moisture field splits the temperate middle.
pub fn biome(cfg: &TerrainConfig, direction: Vec3) -> Biome {
    biome_at(cfg, direction, surface_altitude(cfg, direction))
}

/// [`biome`] with the altitude already in hand, which every column has.
pub fn biome_at(cfg: &TerrainConfig, direction: Vec3, surface_m: f32) -> Biome {
    let sea = cfg.sea_level_m;
    if surface_m < sea - 1.0 {
        return Biome::Ocean;
    }
    if surface_m < sea + cfg.beach_band_m {
        return Biome::Beach;
    }
    if direction.y.abs() > cfg.cold_latitude {
        return Biome::Tundra;
    }
    if surface_m > cfg.mountain_elev_m {
        return Biome::Mountains;
    }
    let m = moisture(cfg, direction);
    if m < cfg.desert_below {
        return Biome::Desert;
    }
    if m > cfg.wet_above {
        return if surface_m < cfg.swamp_max_elev_m {
            Biome::Swamp
        } else {
            Biome::Jungle
        };
    }
    Biome::Fields
}

// ---- The top block -----------------------------------------------------------

/// The material of a column's top cell: the reference's top-block rule, which
/// is where a biome becomes something a player can see.
pub fn top_material(cfg: &TerrainConfig, direction: Vec3, surface_m: f32) -> Material {
    let sea = cfg.sea_level_m;
    let latitude = direction.y.abs();
    let polar = latitude > cfg.polar_latitude;
    let cold = latitude > cfg.cold_latitude;
    let snow_line = if cold {
        cfg.snow_line_m * 0.5
    } else {
        cfg.snow_line_m
    };
    if surface_m > sea - cfg.beach_band_m && surface_m < sea + cfg.beach_band_m {
        return Material::Sand;
    }
    if surface_m < sea {
        return if surface_m > sea - 4.0 {
            Material::Sand
        } else {
            Material::Stone
        };
    }
    let biome = biome_at(cfg, direction, surface_m);
    if biome == Biome::Mountains {
        return if surface_m > cfg.mountain_snowcap_elev_m {
            Material::Snow
        } else {
            Material::Stone
        };
    }
    // A rocky tree-line band first, snow on top of it: the natural order.
    // Tundra stays snowy at every elevation.
    if surface_m > cfg.stone_line_m {
        return Material::Snow;
    }
    if surface_m > snow_line && biome != Biome::Tundra {
        return Material::Stone;
    }
    if polar || (biome == Biome::Tundra && surface_m > snow_line) {
        return Material::Snow;
    }
    // A cheap latitude dither, the reference's own, for the broken tops.
    let dither = (latitude * 977.0).fract();
    match biome {
        Biome::Desert => {
            if dither < cfg.desert_rock_frac {
                Material::Rock
            } else {
                Material::Sand
            }
        }
        Biome::Swamp => {
            if dither < 0.35 {
                Material::Dirt
            } else if dither > 0.80 {
                Material::Water
            } else {
                Material::Grass
            }
        }
        Biome::Jungle => Material::JungleGrass,
        Biome::Tundra => Material::Snow,
        Biome::Fields => Material::DryGrass,
        Biome::Ocean | Biome::Beach | Biome::Mountains => Material::Grass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(n: usize) -> impl Iterator<Item = Vec3> {
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..n).map(move |i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let t = golden * i as f32;
            Vec3::new(r * t.cos(), y, r * t.sin())
        })
    }

    /// The primitive is the reference's `gnoise3d_seed` bit for bit: these
    /// values were computed by that function on these inputs.
    #[test]
    fn gradient_noise_is_the_references_bit_for_bit() {
        for (seed, p, want) in REFERENCE_NOISE {
            let got = gradient_noise(seed, p[0], p[1], p[2]);
            assert_eq!(
                got.to_bits(),
                want,
                "seed {seed:#x} at {p:?}: got {got} ({:#010x}) want {} ({want:#010x})",
                got.to_bits(),
                f32::from_bits(want)
            );
        }
    }

    /// The generator holds this body's budget: summits near 150 m, the floor
    /// near 80 m, and roughly half the sphere is land, as the reference's
    /// seeded world is.
    #[test]
    fn relief_holds_the_budget_and_the_land_fraction() {
        let cfg = TerrainConfig::default();
        let n = 100_000;
        let (mut peak, mut floor, mut land) = (f32::MIN, f32::MAX, 0usize);
        for d in sphere(n) {
            let h = surface_altitude(&cfg, d);
            assert!(h.is_finite());
            peak = peak.max(h);
            floor = floor.min(h);
            if h >= cfg.sea_level_m {
                land += 1;
            }
        }
        let land = land as f32 / n as f32;
        let measured = format!(
            "summit {peak:.1} m, floor {floor:.1} m, land {:.1}%",
            land * 100.0
        );
        assert!((130.0..=180.0).contains(&peak), "{measured}");
        assert!((-110.0..=-60.0).contains(&floor), "{measured}");
        assert!((0.40..=0.60).contains(&land), "{measured}");
    }

    /// The ground a walker crosses is as stepped as the reference's. Measured
    /// the same way on both generators: over land, the share of adjacent
    /// cells whose one-metre caps differ. The reference is 44.0% at a mean
    /// step of 0.56 m; before the land-scale split this was 14.2% at 0.17 m,
    /// because the finest feature in the whole height field was 188 m, which
    /// is sixty-six cells.
    #[test]
    fn the_ground_is_as_rough_as_the_reference() {
        let cfg = TerrainConfig::default();
        const TILE: f32 = 2.833;
        let step = TILE / cfg.radius_m;
        let (mut pairs, mut stepped, mut total) = (0usize, 0usize, 0.0f64);
        for d in sphere(20_000) {
            let h = surface_altitude(&cfg, d).floor();
            if h < cfg.sea_level_m {
                continue;
            }
            let (a, b) = d.any_orthonormal_pair();
            for axis in [a, b] {
                let n = (d + axis * step).normalize();
                let nh = surface_altitude(&cfg, n);
                if nh < cfg.sea_level_m {
                    continue;
                }
                pairs += 1;
                total += (nh.floor() - h).abs() as f64;
                if (nh.floor() - h).abs() >= 1.0 {
                    stepped += 1;
                }
            }
        }
        let share = stepped as f64 / pairs as f64;
        let mean = total / pairs as f64;
        assert!(
            share >= 0.33,
            "only {:.1}% of neighbours step, mean {mean:.2} m",
            share * 100.0
        );
        assert!(mean >= 0.4, "mean step {mean:.2} m");
        // And nothing in the field may be coarser than a few cells, which is
        // the cause the share above is only a symptom of.
        let finest = cfg.radius_m / (cfg.scale_of(cfg.detail_m) * 2.0);
        assert!(finest < 20.0, "finest feature {finest:.0} m");
        assert!(
            finest / TILE < 7.0,
            "finest feature {:.0} cells",
            finest / TILE
        );
    }

    /// A biome is a place a walker passes THROUGH, not a hemisphere they are
    /// stuck in: the moisture field is land-scale, so a kilometre of land
    /// crosses more than one of them.
    #[test]
    fn a_kilometre_of_land_crosses_more_than_one_biome() {
        let cfg = TerrainConfig::default();
        const TILE: f32 = 2.833;
        let step = TILE / cfg.radius_m;
        let mut walks = 0;
        let mut single = 0;
        for start in sphere(400) {
            if surface_altitude(&cfg, start) < cfg.sea_level_m + 4.0 {
                continue;
            }
            let (heading, _) = start.any_orthonormal_pair();
            let mut seen = std::collections::BTreeSet::new();
            let mut here = start;
            let mut dry = true;
            for _ in 0..(1_000.0 / TILE) as usize {
                here = (here + heading * step).normalize();
                let h = surface_altitude(&cfg, here);
                if h < cfg.sea_level_m {
                    dry = false;
                    break;
                }
                seen.insert(biome_at(&cfg, here, h));
            }
            if !dry {
                continue;
            }
            walks += 1;
            if seen.len() < 2 {
                single += 1;
            }
        }
        assert!(walks > 20, "only {walks} inland walks to judge");
        assert!(
            single * 4 < walks,
            "{single} of {walks} kilometre walks stayed inside one biome"
        );
    }

    /// Mountains stand on land: the ridges are multiplied by the continent,
    /// so nothing above the mountain elevation is within a beach of the sea.
    #[test]
    fn mountains_stand_on_land() {
        let cfg = TerrainConfig::default();
        let step = 2.833 / 4_800.0;
        for d in sphere(60_000) {
            if surface_altitude(&cfg, d) <= cfg.mountain_elev_m {
                continue;
            }
            let (a, b) = d.any_orthonormal_pair();
            for k in 0..8 {
                let angle = k as f32 * std::f32::consts::TAU / 8.0;
                let n = (d + (a * angle.cos() + b * angle.sin()) * step * 3.0).normalize();
                assert!(
                    surface_altitude(&cfg, n) >= cfg.sea_level_m,
                    "a mountain at {d:?} has sea three cells away"
                );
            }
        }
    }

    /// A river channel drains: walking downhill from a channel cell ends at
    /// or below the sea, within a bounded number of steps.
    #[test]
    fn rivers_reach_the_sea() {
        let cfg = TerrainConfig::default();
        let step = 2.833 / 4_800.0;
        let mut channels = 0;
        for d in sphere(40_000) {
            let h = surface_altitude(&cfg, d);
            if !(cfg.sea_level_m - cfg.river_depth_m + 0.5..cfg.sea_level_m).contains(&h) {
                continue;
            }
            if river_channel(&cfg, d) <= cfg.river_threshold {
                continue;
            }
            channels += 1;
            let (mut here, mut height) = (d, h);
            for _ in 0..4000 {
                if height < cfg.sea_level_m - cfg.river_depth_m + 0.25 {
                    break;
                }
                let (a, b) = here.any_orthonormal_pair();
                let mut best = (here, height);
                for k in 0..8 {
                    let angle = k as f32 * std::f32::consts::TAU / 8.0;
                    let next = (here + (a * angle.cos() + b * angle.sin()) * step).normalize();
                    let next_height = surface_altitude(&cfg, next);
                    if next_height < best.1 {
                        best = (next, next_height);
                    }
                }
                if best.0 == here {
                    break;
                }
                here = best.0;
                height = best.1;
            }
            assert!(
                height < cfg.sea_level_m,
                "a channel at {d:?} pooled at {height} m"
            );
        }
        assert!(
            channels > 20,
            "only {channels} channel samples on the sphere"
        );
    }

    /// Every biome and every top material occurs, and the classification is
    /// the same one the top block reads.
    #[test]
    fn every_biome_and_material_occurs_and_the_beach_is_sand() {
        let cfg = TerrainConfig::default();
        let mut biomes = std::collections::BTreeSet::new();
        let mut materials = std::collections::BTreeSet::new();
        for d in sphere(50_000) {
            let h = surface_altitude(&cfg, d);
            let b = biome_at(&cfg, d, h);
            biomes.insert(b);
            materials.insert(top_material(&cfg, d, h) as u16);
            if b == Biome::Beach {
                assert_eq!(top_material(&cfg, d, h), Material::Sand);
            }
        }
        for b in [
            Biome::Ocean,
            Biome::Beach,
            Biome::Fields,
            Biome::Desert,
            Biome::Jungle,
            Biome::Swamp,
            Biome::Mountains,
            Biome::Tundra,
        ] {
            assert!(biomes.contains(&b), "no {b:?} on the sphere");
        }
        assert!(materials.len() >= 6, "only {} materials", materials.len());
    }

    /// The measurement instrument behind the thresholds: what share of the
    /// sphere each biome and each top material takes, and the relief. Run
    /// with `--ignored --nocapture`; the reference's seeded world measured
    /// Ocean 45.4, Beach 15.7, Fields 27.8, Tundra 7.3, Desert 1.7, Jungle
    /// 1.0, Mountains 0.9, Swamp 0.1 percent.
    #[test]
    #[ignore]
    fn distribution_report() {
        let cfg = TerrainConfig::default();
        let n = 200_000;
        let mut biomes = std::collections::BTreeMap::new();
        let mut materials = std::collections::BTreeMap::new();
        let mut hist = [0usize; 16];
        for d in sphere(n) {
            let h = surface_altitude(&cfg, d);
            *biomes.entry(biome_at(&cfg, d, h)).or_insert(0usize) += 1;
            *materials
                .entry(top_material(&cfg, d, h) as u16)
                .or_insert(0usize) += 1;
            hist[(((h + 100.0) / 20.0).floor() as isize).clamp(0, 15) as usize] += 1;
        }
        let pct = |c: &usize| format!("{:.1}%", 100.0 * *c as f64 / n as f64);
        println!(
            "biomes: {:?}",
            biomes
                .iter()
                .map(|(k, v)| format!("{k:?} {}", pct(v)))
                .collect::<Vec<_>>()
        );
        println!(
            "materials: {:?}",
            materials
                .iter()
                .map(|(k, v)| format!("{k} {}", pct(v)))
                .collect::<Vec<_>>()
        );
        println!(
            "height, 20 m bins from -100: {:?}",
            hist.iter().map(pct).collect::<Vec<_>>()
        );
    }

    /// How BUMPY the ground is, which is the thing a walker feels and a
    /// picture shows: over land, the share of adjacent cells whose one-metre
    /// quantised caps differ at all, and by how much. A surface that reads as
    /// smooth is one where most neighbours share a height. Also the finest
    /// wavelength each term carries, in metres, since a height field has
    /// nothing to show below the shortest one in it. Run with `--ignored
    /// --nocapture`; the same numbers off `tenebris-core` are the reference.
    #[test]
    #[ignore]
    fn roughness_report() {
        let cfg = TerrainConfig::default();
        let radius = cfg.radius_m;
        const TILE: f32 = 2.833;
        let step = TILE / radius;
        let (mut pairs, mut stepped, mut total) = (0usize, 0usize, 0.0f64);
        for d in sphere(40_000) {
            let h = surface_altitude(&cfg, d).floor();
            if h < cfg.sea_level_m {
                continue;
            }
            let (a, b) = d.any_orthonormal_pair();
            for axis in [a, b] {
                let n = (d + axis * step).normalize();
                if surface_altitude(&cfg, n) < cfg.sea_level_m {
                    continue;
                }
                let delta = (surface_altitude(&cfg, n).floor() - h).abs();
                pairs += 1;
                total += delta as f64;
                if delta >= 1.0 {
                    stepped += 1;
                }
            }
        }
        println!(
            "neighbours {pairs}: {:.1}% differ by a block or more, mean step {:.2} m",
            100.0 * stepped as f64 / pairs as f64,
            total / pairs as f64
        );
        for (name, scale, octaves, lacunarity) in [
            ("continent", cfg.continent_scale, 6u32, 2.0f32),
            ("mountain", cfg.scale_of(cfg.mountain_m), 4, 2.2),
            ("hill", cfg.scale_of(cfg.hill_m), 3, 2.0),
            ("detail", cfg.scale_of(cfg.detail_m), 2, 2.0),
            ("river", cfg.scale_of(cfg.river_m), 4, 2.0),
            ("moisture", cfg.scale_of(cfg.moisture_m), 4, 2.0),
        ] {
            let finest = radius / (scale * lacunarity.powi(octaves as i32 - 1));
            println!(
                "  {name:9} coarsest {:7.0} m, finest {finest:6.0} m = {:5.0} cells",
                radius / scale,
                finest / TILE
            );
        }
    }

    /// Computed by `tenebris_core::rng::gnoise3d_seed` on these inputs.
    const REFERENCE_NOISE: [(u64, [f32; 3], u32); 4] = [
        (0x5eed_2026, [0.3, 0.7, 0.2], 0xbebfa87c),
        (0x561cb73764fea0f2, [1.9, -2.3, 0.4], 0x3eacb970),
        (1, [12.5, 3.25, -7.75], 0xbc87d600),
        (0xB10E_5EED, [0.0, 0.0, 0.0], 0x00000000),
    ];
}
