//! Wild herds, derived from the world seed and never saved (S10).
//!
//! The surface is cut into squares of `square_m` by latitude and longitude
//! (a row of squares narrows its longitude span toward the poles so they
//! stay near square). A square holds a herd if its hash says so; the hash
//! also picks the species, the herd's size, its centre in the square, and
//! each member's seed and sex. Whether the centre is dry, grass-topped
//! land is the world's to say, so the app asks that.

use super::{HerdSettings, LandSpecies, Sex};
use crate::fauna::hash2;
use glam::Vec3;

/// A square of the surface, by row (latitude) and column (longitude).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square {
    pub row: i32,
    pub col: i32,
}

/// One wild animal of a herd.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    /// Its seed: its genome, and the key a tamed record keeps.
    pub seed: u64,
    pub sex: Sex,
    /// Where it stands about the centre, east and north, m.
    pub offset: (f32, f32),
}

/// A herd a square holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Herd {
    pub square: Square,
    pub species: u16,
    /// Its centre, a unit direction in the body's frame.
    pub centre: Vec3,
    pub members: Vec<Member>,
}

fn rows(radius_m: f32, s: &HerdSettings) -> i32 {
    ((std::f32::consts::PI * radius_m / s.square_m).round() as i32).max(1)
}

fn cols(row: i32, rows: i32) -> i32 {
    let lat = ((row as f32 + 0.5) / rows as f32 - 0.5) * std::f32::consts::PI;
    ((2 * rows) as f32 * lat.cos()).round().max(1.0) as i32
}

/// Latitude (radians, +Y north) and longitude of a unit direction.
fn lat_lon(d: Vec3) -> (f32, f32) {
    (d.y.clamp(-1.0, 1.0).asin(), d.z.atan2(d.x))
}

fn direction(lat: f32, lon: f32) -> Vec3 {
    Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
}

/// The square a direction is in.
pub fn square_of(d: Vec3, radius_m: f32, s: &HerdSettings) -> Square {
    let n = rows(radius_m, s);
    let (lat, lon) = lat_lon(d.normalize_or(Vec3::Y));
    let row = (((lat / std::f32::consts::PI + 0.5) * n as f32) as i32).clamp(0, n - 1);
    let m = cols(row, n);
    let u = (lon / std::f32::consts::TAU).rem_euclid(1.0);
    let col = ((u * m as f32) as i32).clamp(0, m - 1);
    Square { row, col }
}

/// The squares within about `reach_m` of a direction, in a fixed order:
/// found by sampling the tangent plane at half a square's spacing.
pub fn squares_near(d: Vec3, reach_m: f32, radius_m: f32, s: &HerdSettings) -> Vec<Square> {
    let up = d.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let step = s.square_m * 0.5;
    let span = (reach_m / step).ceil() as i32;
    let mut out = Vec::new();
    for i in -span..=span {
        for j in -span..=span {
            let at = up * radius_m + east * (i as f32 * step) + north * (j as f32 * step);
            out.push(square_of(at, radius_m, s));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The herd a square holds in the first roll, if its hash gives one: not
/// yet checked against the ground.
pub fn herd_in(
    seed: u64,
    square: Square,
    radius_m: f32,
    s: &HerdSettings,
    roster: &[LandSpecies],
) -> Option<Herd> {
    roll(seed, square, radius_m, s, roster, 0)
}

/// The herd a square holds in the game's roll (`survival-mode` design 17):
/// the hare, the ridge goat, the rootling, the red fox and the dust hound,
/// over the same squares with a hash of their own, so the first roll's
/// herds are what they were before any of them. Not yet checked against
/// the ground.
pub fn game_herd_in(
    seed: u64,
    square: Square,
    radius_m: f32,
    s: &HerdSettings,
    roster: &[LandSpecies],
) -> Option<Herd> {
    roll(seed, square, radius_m, s, roster, 1)
}

/// Every herd a square holds, the first roll's first.
pub fn herds_in(
    seed: u64,
    square: Square,
    radius_m: f32,
    s: &HerdSettings,
    roster: &[LandSpecies],
) -> impl Iterator<Item = Herd> {
    herd_in(seed, square, radius_m, s, roster)
        .into_iter()
        .chain(game_herd_in(seed, square, radius_m, s, roster))
}

fn roll(
    seed: u64,
    square: Square,
    radius_m: f32,
    s: &HerdSettings,
    roster: &[LandSpecies],
    layer: u8,
) -> Option<Herd> {
    // Every species of the layer but a bought one (`land-transport` design
    // 10): a horse is sold in town, never rolled wild, so the herds of
    // every world made before it are the same animal for animal.
    let candidates: Vec<usize> = (0..roster.len())
        .filter(|&i| roster[i].status != crate::fauna::Status::Bought)
        .filter(|&i| roster[i].herd_layer == layer)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let (salt, chance) = match layer {
        0 => (0x4e2d_0000, s.chance),
        _ => (0x4e2d_0001, s.game_chance),
    };
    let h = hash2(
        hash2(seed, salt),
        (u64::from(square.row as u32) << 32) | u64::from(square.col as u32),
    );
    if (h >> 40) as f32 / (1u64 << 24) as f32 >= chance {
        return None;
    }
    let species = candidates[(hash2(h, 1) % candidates.len() as u64) as usize];
    let sp = &roster[species];
    let n = rows(radius_m, s);
    let m = cols(square.row, n);
    // The centre, kept off the square's edges.
    let fu = 0.2 + 0.6 * ((hash2(h, 2) >> 40) as f32 / (1u64 << 24) as f32);
    let fv = 0.2 + 0.6 * ((hash2(h, 3) >> 40) as f32 / (1u64 << 24) as f32);
    let lat = ((square.row as f32 + fv) / n as f32 - 0.5) * std::f32::consts::PI;
    let lon = (square.col as f32 + fu) / m as f32 * std::f32::consts::TAU;
    let (lo, hi) = (u64::from(sp.herd.0), u64::from(sp.herd.1));
    let count = lo + hash2(h, 4) % (hi - lo + 1);
    let male_one_in = sp.keep.as_ref().map_or(2, |k| u64::from(k.male_one_in));
    let members = (0..count)
        .map(|i| {
            let seed = hash2(h, 100 + i);
            // The first of a herd is its male; the rest by chance.
            let sex = if i == 0 || hash2(seed, 9).is_multiple_of(male_one_in) {
                Sex::M
            } else {
                Sex::F
            };
            let a = (hash2(seed, 7) >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
            let r = s.spread_m * ((hash2(seed, 8) >> 40) as f32 / (1u64 << 24) as f32).sqrt();
            Member {
                seed,
                sex,
                offset: (r * a.cos(), r * a.sin()),
            }
        })
        .collect();
    Some(Herd {
        square,
        species: species as u16,
        centre: direction(lat, lon),
        members,
    })
}
