//! The world map's raster: what the map draws at every pixel, at every level
//! of zoom (`world-map` decisions 3, 9 and 10).
//!
//! What a place IS comes from `pbd_core::map::base_texel`, the function the
//! mockup's rasters were made with, so the map cannot show a planet the ground
//! does not have. What it LOOKS like is decided here, from the same tileset
//! pictures the ground is drawn with: each top block in its biome's tile,
//! shaded by relief from the north-west, the sea by its depth. The mockup's
//! `tools/world_map.py` did this first; this is the authority now.
//!
//! The levels are counted down from the cell (decision 10): the finest is
//! 10,656 pixels round the equator, 2.830 m a pixel against a cell's 2.833 m,
//! and each coarser one halves it. Tiles are [`TILE`] pixels square, which
//! makes every level a whole number of them. The coarsest level is the BASE:
//! built whole, once a world, and cached in the save's folder under the seed
//! and the generator version. The finer two are built a tile at a time.
//!
//! The cache holds texels, not colours, so a repainted tile shows on the next
//! start without a key that has to know about it.

use crate::planet::{PLANET_RADIUS, terrain_config};
use crate::saves::format::{material_code, material_of};
use pbd_core::geo;
use pbd_core::map::{Texel, base_texel};
use pbd_core::planet_gen::{Biome, TerrainConfig};
use pbd_core::terrain::Material;
use std::path::{Path, PathBuf};

/// A tile's side, pixels.
pub const TILE: usize = 333;

/// The levels' widths, coarsest first. Each is twice the last, and a whole
/// number of tiles across and, at half the width, down.
pub const LEVELS: [usize; 3] = [8 * TILE, 16 * TILE, 32 * TILE];

/// The base level: built whole and cached.
pub const BASE: usize = 0;

/// A level's size, pixels: twice as wide as it is tall, as an equirectangular
/// map of a sphere is.
pub fn level_size(level: usize) -> (usize, usize) {
    let width = LEVELS[level];
    (width, width / 2)
}

/// A level's pixel at the equator, metres.
pub fn metres_per_pixel(level: usize) -> f32 {
    std::f32::consts::TAU * PLANET_RADIUS / LEVELS[level] as f32
}

/// Every texel of a block of a level, row by row, with a border of one pixel
/// round it: `(w + 2) x (h + 2)` texels starting at `(x0 - 1, y0 - 1)`. The
/// border is what relief shading needs at the block's edge, so two blocks
/// side by side shade their shared edge alike. Columns wrap round the
/// antimeridian; rows past a pole read the pole's row.
pub fn texel_block(
    cfg: &TerrainConfig,
    level: usize,
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
) -> Vec<Texel> {
    let (width, height) = level_size(level);
    let mut texels = Vec::with_capacity((w + 2) * (h + 2));
    for row in 0..h + 2 {
        let y = (y0 + row).saturating_sub(1).min(height - 1);
        for col in 0..w + 2 {
            let x = (x0 + width + col - 1) % width;
            texels.push(base_texel(cfg, geo::pixel_direction(x, y, width, height)));
        }
    }
    texels
}

/// The tileset colours the map is painted with: each tile's mean, from the
/// same pictures the ground is drawn with.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    /// Per biome, in `Biome` order: its sheet's ground (tile 0,0), earth
    /// (2,0) and stone (3,0).
    ground: [[f32; 3]; 8],
    earth: [[f32; 3]; 8],
    stone: [[f32; 3]; 8],
}

/// The biomes in their `repr` order, which is the order `Palette` keeps.
const BIOMES: [Biome; 8] = [
    Biome::Ocean,
    Biome::Beach,
    Biome::Fields,
    Biome::Desert,
    Biome::Jungle,
    Biome::Swamp,
    Biome::Mountains,
    Biome::Tundra,
];

/// A biome's sheet's file name, as the ground's `tileset_slot` names it.
fn sheet_name(biome: Biome) -> &'static str {
    match biome {
        Biome::Ocean => "ocean",
        Biome::Beach => "beach",
        Biome::Fields => "fields",
        Biome::Desert => "desert",
        Biome::Jungle => "jungle",
        Biome::Swamp => "swamp",
        Biome::Mountains => "mountains",
        Biome::Tundra => "tundra",
    }
}

/// The shipped tilesets, resolved as the config directory is.
pub fn tileset_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tilesets"
    ))
}

/// The mean colour of tile `(col, row)` of a 4 x 4 sheet, 0..255 sRGB.
fn tile_mean(sheet: &image::RgbImage, col: u32, row: u32) -> [f32; 3] {
    let (w, h) = (sheet.width() / 4, sheet.height() / 4);
    let mut sum = [0.0f64; 3];
    for y in row * h..(row + 1) * h {
        for x in col * w..(col + 1) * w {
            let p = sheet.get_pixel(x, y).0;
            for c in 0..3 {
                sum[c] += f64::from(p[c]);
            }
        }
    }
    let n = f64::from(w * h);
    sum.map(|s| (s / n) as f32)
}

impl Palette {
    /// Read the eight biome sheets from `dir`.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut palette = Palette {
            ground: [[0.0; 3]; 8],
            earth: [[0.0; 3]; 8],
            stone: [[0.0; 3]; 8],
        };
        for (i, biome) in BIOMES.iter().enumerate() {
            let path = dir.join(format!("{}.png", sheet_name(*biome)));
            let sheet = image::open(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?
                .to_rgb8();
            palette.ground[i] = tile_mean(&sheet, 0, 0);
            palette.earth[i] = tile_mean(&sheet, 2, 0);
            palette.stone[i] = tile_mean(&sheet, 3, 0);
        }
        Ok(palette)
    }

    /// The colour of a place's top block, before relief: its biome's ground
    /// tile, or its earth for dirt, its stone for stone and rock and ore;
    /// sand from the beach's sheet, or the desert's in a desert; snow from
    /// the tundra's. As the mockup painted it.
    pub fn ground_colour(&self, texel: &Texel) -> [f32; 3] {
        let b = texel.biome as usize;
        match texel.top {
            Material::Dirt => self.earth[b],
            Material::Stone | Material::Rock | Material::Ore => self.stone[b],
            Material::Sand if texel.biome == Biome::Desert => self.ground[Biome::Desert as usize],
            Material::Sand => self.ground[Biome::Beach as usize],
            Material::Snow => self.ground[Biome::Tundra as usize],
            Material::Water => [70.0, 110.0, 120.0],
            _ => self.ground[b],
        }
    }
}

/// The sea's colour at a depth, metres: the shallows pale, the shelf mid
/// blue past the fish classes' 6 m, the deep dark past 40 m.
pub fn sea_colour(depth_m: f32) -> [f32; 3] {
    const STOPS: [(f32, [f32; 3]); 5] = [
        (0.0, [120.0, 190.0, 200.0]),
        (6.0, [70.0, 150.0, 185.0]),
        (40.0, [35.0, 90.0, 150.0]),
        (150.0, [18.0, 45.0, 95.0]),
        (400.0, [10.0, 25.0, 60.0]),
    ];
    for pair in STOPS.windows(2) {
        let ((d0, c0), (d1, c1)) = (pair[0], pair[1]);
        if depth_m < d1 {
            let t = ((depth_m - d0) / (d1 - d0)).clamp(0.0, 1.0);
            return std::array::from_fn(|c| c0[c] + (c1[c] - c0[c]) * t);
        }
    }
    STOPS[4].1
}

/// A hillshade factor from the north-west: the ground's slope from its four
/// neighbours' altitudes (east, west, north, south, metres, the sea floor
/// counted as the shore), exaggerated three times, since a planet 4.8 km
/// across has gentle hills. The east-west spacing narrows with the latitude,
/// so a slope reads the same at every latitude.
pub fn relief(level: usize, lat: f32, east: f32, west: f32, north: f32, south: f32) -> f32 {
    let spacing = metres_per_pixel(level);
    let across = lat.cos().max(0.05) * spacing;
    let dx = (east.max(0.0) - west.max(0.0)) / (2.0 * across);
    let dy = (north.max(0.0) - south.max(0.0)) / (2.0 * spacing);
    let n = bevy::math::Vec3::new(-3.0 * dx, -3.0 * dy, 1.0).normalize();
    let light = bevy::math::Vec3::new(-1.0, 1.0, 1.4).normalize();
    (0.55 + 0.6 * (n.dot(light) - light.z) + 0.45).clamp(0.45, 1.25)
}

/// Colour a block of texels made by [`texel_block`]: `w x h` RGBA pixels,
/// sRGB, opaque, for rows starting at `y0` of `level`.
pub fn colour_block(
    texels: &[Texel],
    level: usize,
    y0: usize,
    w: usize,
    h: usize,
    palette: &Palette,
) -> Vec<u8> {
    let (_, height) = level_size(level);
    let stride = w + 2;
    let at = |col: usize, row: usize| &texels[row * stride + col];
    let mut out = Vec::with_capacity(w * h * 4);
    for row in 1..=h {
        let y = y0 + row - 1;
        let lat = std::f32::consts::PI * (0.5 - (y as f32 + 0.5) / height as f32);
        // The top and bottom rows have no row beyond the pole to slope to.
        let pole = y == 0 || y == height - 1;
        for col in 1..=w {
            let t = at(col, row);
            let colour = if t.sea {
                sea_colour(terrain_config().sea_level_m - t.altitude_m)
            } else {
                let (north, south) = if pole {
                    (t.altitude_m, t.altitude_m)
                } else {
                    (at(col, row - 1).altitude_m, at(col, row + 1).altitude_m)
                };
                let shade = relief(
                    level,
                    lat,
                    at(col + 1, row).altitude_m,
                    at(col - 1, row).altitude_m,
                    north,
                    south,
                );
                palette.ground_colour(t).map(|c| c * shade)
            };
            out.extend(colour.map(|c| c.round().clamp(0.0, 255.0) as u8));
            out.push(255);
        }
    }
    out
}

/// The base level's cache file for a seed and the generator that made it:
/// the version is in the name, so a new generator reads nothing of the old
/// one's. It is passed, not read at the moment of saving, since a switch of
/// generator can come between a build's start and its save.
pub fn cache_name(seed: u64, generator: u32) -> String {
    format!("map-base-g{generator}-{seed:016x}.png")
}

/// Pack a texel for the cache: its altitude in 16 bits (floored metres, the
/// layer the column is floored to, offset by 32,768), its top block's saved
/// code and its biome.
fn pack(t: &Texel) -> [u8; 4] {
    let altitude = (t.altitude_m.floor() + 32768.0).clamp(0.0, 65535.0) as u16;
    let [high, low] = altitude.to_be_bytes();
    [high, low, material_code(t.top), t.biome as u8]
}

fn unpack(p: [u8; 4], cfg: &TerrainConfig) -> Option<Texel> {
    let altitude_m = f32::from(u16::from_be_bytes([p[0], p[1]])) - 32768.0;
    Some(Texel {
        altitude_m,
        biome: *BIOMES.get(usize::from(p[3]))?,
        top: material_of(p[2])?,
        sea: altitude_m < cfg.sea_level_m,
    })
}

/// Write the base level's texels (without the border) to the world's folder.
pub fn save_base(dir: &Path, seed: u64, generator: u32, texels: &[Texel]) -> Result<(), String> {
    let (width, height) = level_size(BASE);
    if texels.len() != width * height {
        return Err(format!(
            "{} texels for a {width} x {height} base",
            texels.len()
        ));
    }
    let bytes: Vec<u8> = texels.iter().flat_map(pack).collect();
    let image = image::RgbaImage::from_raw(width as u32, height as u32, bytes)
        .ok_or("the base did not fit its image")?;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join(cache_name(seed, generator));
    image
        .save(&path)
        .map_err(|error| format!("{}: {error}", path.display()))
}

/// The base level's texels from the world's folder, if a cache for this seed
/// and this generator is there. Another generator's cache for the same seed
/// is deleted on the way: it can never be read again.
pub fn load_base(dir: &Path, seed: u64, generator: u32, cfg: &TerrainConfig) -> Option<Vec<Texel>> {
    let name = cache_name(seed, generator);
    let this_seed = format!("-{seed:016x}.png");
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let other = entry.file_name().to_string_lossy().into_owned();
            if other.starts_with("map-base-") && other.ends_with(&this_seed) && other != name {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    let image = image::open(dir.join(&name)).ok()?.to_rgba8();
    let (width, height) = level_size(BASE);
    if image.dimensions() != (width as u32, height as u32) {
        return None;
    }
    image.pixels().map(|p| unpack(p.0, cfg)).collect()
}

/// The whole base level's texels, row by row, without a border: the rows
/// split between the machine's threads, since it is 3.5 million texels.
pub fn build_base(cfg: &TerrainConfig) -> Vec<Texel> {
    let (width, height) = level_size(BASE);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let rows_each = height.div_ceil(threads);
    let mut texels = Vec::with_capacity(width * height);
    std::thread::scope(|scope| {
        let parts: Vec<_> = (0..height)
            .step_by(rows_each)
            .map(|first| {
                scope.spawn(move || {
                    let last = (first + rows_each).min(height);
                    (first..last)
                        .flat_map(|y| (0..width).map(move |x| (x, y)))
                        .map(|(x, y)| base_texel(cfg, geo::pixel_direction(x, y, width, height)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for part in parts {
            texels.extend(part.join().expect("a base row thread"));
        }
    });
    texels
}

/// A map layer drawn from the base's texels: a colour, with its alpha, for
/// each place, or clear (`world-map` decision 7). `city-sites` and
/// `climate-and-fish-maps` add theirs here, and the map's code does not
/// change for them.
#[derive(Clone, Copy, Debug)]
pub struct RasterLayer {
    /// Its button in the legend.
    pub name: &'static str,
    /// One line under the legend saying what it shows.
    pub note: &'static str,
    pub paint: fn(&Texel) -> [u8; 4],
    /// Its colour key, as the legend shows it: a name and a colour a row
    /// (`world-map` decision 11).
    pub key: &'static [(&'static str, [u8; 3])],
    /// Which row of the key a texel counts toward, or `None` where the layer
    /// draws nothing: what each row's share of the ground is counted from.
    pub class: fn(&Texel) -> Option<usize>,
}

/// Each row of a layer's key's share of the texels it counts, weighted by
/// the area a texel covers (the cosine of its latitude), as the mockup's
/// legend gives each biome's share of the land.
pub fn layer_shares(layer: &RasterLayer, texels: &[Texel], width: usize) -> Vec<f32> {
    let height = texels.len() / width.max(1);
    let mut weight = vec![0.0_f64; layer.key.len()];
    for (row, line) in texels.chunks(width).enumerate() {
        let latitude = (0.5 - (row as f64 + 0.5) / height as f64) * std::f64::consts::PI;
        let area = latitude.cos();
        for texel in line {
            if let Some(class) = (layer.class)(texel).filter(|&c| c < weight.len()) {
                weight[class] += area;
            }
        }
    }
    let total: f64 = weight.iter().sum();
    weight
        .iter()
        .map(|w| if total > 0.0 { (w / total) as f32 } else { 0.0 })
        .collect()
}

/// Every layer the legend lists, in the order they were added.
#[derive(bevy::prelude::Resource, Default)]
pub struct MapLayers {
    pub rasters: Vec<RasterLayer>,
}

impl MapLayers {
    /// Add a layer, and return its index in the legend's list.
    pub fn add(&mut self, layer: RasterLayer) -> usize {
        self.rasters.push(layer);
        self.rasters.len() - 1
    }
}

/// A layer's picture over the whole base level, RGBA, from its texels.
pub fn paint_layer(layer: &RasterLayer, texels: &[Texel]) -> Vec<u8> {
    texels.iter().flat_map(|t| (layer.paint)(t)).collect()
}

/// The biome layer's colours, as the mockup's overlay drew them (survey M3:
/// biomes are an overlay over the greyed base), at the mockup's 0.88
/// (`world-map` decision 11). The sea is left clear.
pub fn biome_colour(texel: &Texel) -> [u8; 4] {
    match biome_class(texel) {
        Some(class) => {
            let [r, g, b] = BIOME_KEY[class].1;
            [r, g, b, BIOME_ALPHA]
        }
        None => [0; 4],
    }
}

/// The biome key, in the mockup's order and colours.
pub const BIOME_KEY: &[(&str, [u8; 3])] = &[
    ("beach", [232, 214, 150]),
    ("fields", [150, 190, 80]),
    ("desert", [222, 150, 60]),
    ("jungle", [30, 120, 60]),
    ("swamp", [90, 130, 110]),
    ("mountains", [140, 130, 125]),
    ("tundra", [225, 235, 245]),
];

/// A texel's row in [`BIOME_KEY`]; the sea has none.
pub fn biome_class(texel: &Texel) -> Option<usize> {
    if texel.sea {
        return None;
    }
    match texel.biome {
        Biome::Ocean => None,
        Biome::Beach => Some(0),
        Biome::Fields => Some(1),
        Biome::Desert => Some(2),
        Biome::Jungle => Some(3),
        Biome::Swamp => Some(4),
        Biome::Mountains => Some(5),
        Biome::Tundra => Some(6),
    }
}

/// How opaque the biome layer is: the mockup blits it at 0.88.
pub const BIOME_ALPHA: u8 = 224;

/// The base level's texels with the one-pixel border `colour_block` reads,
/// made from the borderless ones by wrapping the columns and repeating the
/// polar rows, as `texel_block` would have computed them.
pub fn bordered(texels: &[Texel], width: usize, height: usize) -> Vec<Texel> {
    let mut out = Vec::with_capacity((width + 2) * (height + 2));
    for row in 0..height + 2 {
        let y = row.saturating_sub(1).min(height - 1);
        for col in 0..width + 2 {
            let x = (col + width - 1) % width;
            out.push(texels[y * width + x]);
        }
    }
    out
}

/// One tile of one level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileKey {
    pub level: u8,
    pub x: u16,
    pub y: u16,
}

impl TileKey {
    /// The tile of `level` holding map position `uv` (`geo::project`'s).
    pub fn under(level: usize, uv: bevy::math::Vec2) -> Self {
        let (width, height) = level_size(level);
        let x = ((uv.x.rem_euclid(1.0) * width as f32) as usize).min(width - 1) / TILE;
        let y = ((uv.y.clamp(0.0, 1.0) * height as f32) as usize).min(height - 1) / TILE;
        TileKey {
            level: level as u8,
            x: x as u16,
            y: y as u16,
        }
    }

    /// Its RGBA pixels, built from the generator.
    pub fn build(self, cfg: &TerrainConfig, palette: &Palette) -> Vec<u8> {
        let level = usize::from(self.level);
        let (x0, y0) = (usize::from(self.x) * TILE, usize::from(self.y) * TILE);
        let texels = texel_block(cfg, level, x0, y0, TILE, TILE);
        colour_block(&texels, level, y0, TILE, TILE, palette)
    }
}

/// The tiles built so far, the least recently drawn dropped first once there
/// are more than `capacity`. Generic over what a tile is kept as, so the
/// order can be tested without a GPU.
pub struct TileLru<T> {
    capacity: usize,
    clock: u64,
    tiles: std::collections::HashMap<TileKey, (T, u64)>,
}

impl<T> TileLru<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            clock: 0,
            tiles: std::collections::HashMap::new(),
        }
    }

    /// A tile if it is built, marked as just used.
    pub fn get(&mut self, key: TileKey) -> Option<&T> {
        self.clock += 1;
        let clock = self.clock;
        self.tiles.get_mut(&key).map(|(tile, used)| {
            *used = clock;
            &*tile
        })
    }

    pub fn contains(&self, key: TileKey) -> bool {
        self.tiles.contains_key(&key)
    }

    /// Keep a built tile, and return whatever the capacity pushed out.
    pub fn insert(&mut self, key: TileKey, tile: T) -> Vec<(TileKey, T)> {
        self.clock += 1;
        self.tiles.insert(key, (tile, self.clock));
        let mut dropped = Vec::new();
        while self.tiles.len() > self.capacity {
            let oldest = self
                .tiles
                .iter()
                .min_by_key(|(key, (_, used))| (*used, **key))
                .map(|(key, _)| *key)
                .expect("over capacity, so not empty");
            if let Some((tile, _)) = self.tiles.remove(&oldest) {
                dropped.push((oldest, tile));
            }
        }
        dropped
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every level is a whole number of tiles, twice as wide as tall, each
    /// twice the last; and the finest lands on the cell: 2.833 m a pixel at
    /// the equator within CLAUDE.md's geodesic spread, 2.595 to 3.101 m
    /// (task 3.3).
    #[test]
    fn the_finest_level_is_a_cell_a_pixel() {
        for (level, &width) in LEVELS.iter().enumerate() {
            let (w, h) = level_size(level);
            assert_eq!((w % TILE, h % TILE), (0, 0), "level {level}");
            assert_eq!(w, 2 * h);
            if level > 0 {
                assert_eq!(width, 2 * LEVELS[level - 1]);
            }
        }
        let finest = metres_per_pixel(LEVELS.len() - 1);
        assert!((2.595..=3.101).contains(&finest), "{finest} m a pixel");
        assert!((finest - 2.833).abs() / 2.833 < 0.01, "{finest} m a pixel");
    }

    /// The tile under the player at the finest zoom is the finest level's,
    /// and holds the player's pixel.
    #[test]
    fn the_tile_under_a_place_holds_it() {
        let finest = LEVELS.len() - 1;
        for direction in [
            bevy::math::Vec3::X,
            bevy::math::Vec3::new(0.3, 0.8, -0.5),
            -bevy::math::Vec3::Z,
        ] {
            let uv = geo::project(direction);
            let key = TileKey::under(finest, uv);
            let (width, height) = level_size(finest);
            let (px, py) = (uv.x * width as f32, uv.y * height as f32);
            let x0 = f32::from(key.x) * TILE as f32;
            let y0 = f32::from(key.y) * TILE as f32;
            assert!((x0..x0 + TILE as f32).contains(&px), "{direction}");
            assert!((y0..y0 + TILE as f32).contains(&py), "{direction}");
        }
        // The antimeridian's far edge and the south pole stay on the map, and
        // u = 1 is u = 0, the same meridian.
        let corner = TileKey::under(finest, bevy::math::Vec2::new(0.999_99, 1.0));
        assert_eq!((corner.x, corner.y), (31, 15));
        let wrapped = TileKey::under(finest, bevy::math::Vec2::new(1.0, 1.0));
        assert_eq!((wrapped.x, wrapped.y), (0, 15));
    }

    /// A tile's shaded edge matches the next tile's, because each is built
    /// with the other's first column as its border: the same pixels either
    /// side of a seam, and the antimeridian included.
    #[test]
    fn tiles_meet_without_a_seam() {
        let cfg = *terrain_config();
        let palette = Palette::load(&tileset_dir()).expect("the tilesets");
        let level = 1;
        let (width, _) = level_size(level);
        let tiles = width / TILE;
        for (left, right) in [(3usize, 4usize), (tiles - 1, 0)] {
            let y0 = 3 * TILE;
            // A strip two pixels wide across the seam, built as its own block.
            let x = (right * TILE + width - 1) % width;
            let strip = colour_block(
                &texel_block(&cfg, level, x, y0, 2, TILE),
                level,
                y0,
                2,
                TILE,
                &palette,
            );
            let a = colour_block(
                &texel_block(&cfg, level, left * TILE, y0, TILE, TILE),
                level,
                y0,
                TILE,
                TILE,
                &palette,
            );
            let b = colour_block(
                &texel_block(&cfg, level, right * TILE, y0, TILE, TILE),
                level,
                y0,
                TILE,
                TILE,
                &palette,
            );
            for row in 0..TILE {
                let last_of_a = &a[(row * TILE + TILE - 1) * 4..][..4];
                let first_of_b = &b[(row * TILE) * 4..][..4];
                assert_eq!(last_of_a, &strip[row * 2 * 4..][..4], "row {row}");
                assert_eq!(first_of_b, &strip[(row * 2 + 1) * 4..][..4], "row {row}");
            }
        }
    }

    /// The cache gives back what was written, for the same seed and
    /// generator; another seed reads nothing; and another generator's file is
    /// deleted rather than read (task 3.2).
    #[test]
    fn the_cache_is_keyed_by_seed_and_generator() {
        let dir = std::env::temp_dir().join(format!("pbd-map-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cfg = *terrain_config();
        let (width, height) = level_size(BASE);
        // A cheap stand-in for the base: every row the same line of places.
        let line: Vec<Texel> = (0..width)
            .map(|x| base_texel(&cfg, geo::pixel_direction(x, height / 3, width, height)))
            .collect();
        let texels: Vec<Texel> = (0..height).flat_map(|_| line.iter().copied()).collect();
        let generator = crate::planet::generator_version();
        save_base(&dir, 7, generator, &texels).expect("written");
        let back = load_base(&dir, 7, generator, &cfg).expect("read back");
        for (a, b) in texels.iter().zip(&back) {
            assert_eq!((a.top, a.biome, a.sea), (b.top, b.biome, b.sea));
            assert_eq!(a.altitude_m.floor(), b.altitude_m);
        }
        assert!(
            load_base(&dir, 8, generator, &cfg).is_none(),
            "another seed"
        );
        // A cache from another generator is not read, and goes.
        let stale = dir.join(cache_name(7, generator + 1));
        std::fs::copy(dir.join(cache_name(7, generator)), &stale).expect("copied");
        std::fs::remove_file(dir.join(cache_name(7, generator))).expect("removed");
        assert!(
            load_base(&dir, 7, generator, &cfg).is_none(),
            "another generator's cache"
        );
        assert!(!stale.exists(), "and it is deleted");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The least recently used tile goes first, and a tile just drawn stays.
    #[test]
    fn the_lru_drops_the_least_recently_drawn() {
        let key = |x| TileKey { level: 2, x, y: 0 };
        let mut lru = TileLru::new(2);
        assert!(lru.insert(key(0), "a").is_empty());
        assert!(lru.insert(key(1), "b").is_empty());
        assert_eq!(lru.get(key(0)), Some(&"a"));
        let dropped = lru.insert(key(2), "c");
        assert_eq!(dropped, vec![(key(1), "b")]);
        assert!(lru.contains(key(0)) && lru.contains(key(2)) && lru.len() == 2);
    }

    /// The colours come from the tiles: meadow ground is green, the deep sea
    /// dark blue, and shading only scales a colour.
    #[test]
    fn the_palette_paints_as_the_ground_does() {
        let palette = Palette::load(&tileset_dir()).expect("the tilesets");
        let meadow = Texel {
            altitude_m: 20.0,
            biome: Biome::Fields,
            top: Material::Grass,
            sea: false,
        };
        let [r, g, b] = palette.ground_colour(&meadow);
        assert!(g > r && g > b, "meadow is green: {r} {g} {b}");
        let [r, g, b] = sea_colour(300.0);
        assert!(b > g && g > r && b < 120.0, "the deep is dark blue");
        assert_eq!(sea_colour(0.0), [120.0, 190.0, 200.0]);
        let flat = relief(1, 0.0, 5.0, 5.0, 5.0, 5.0);
        assert!((flat - 1.0).abs() < 1e-5, "flat ground is unshaded: {flat}");
        // Ground rising to the south-east faces the north-west light.
        assert!(
            relief(1, 0.0, 10.0, 0.0, 0.0, 10.0) > flat,
            "a slope facing north-west is lit"
        );
        assert!(
            relief(1, 0.0, 0.0, 10.0, 10.0, 0.0) < flat,
            "one facing south-east is not"
        );
    }
}
