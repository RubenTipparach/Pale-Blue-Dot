//! `lighting-model` L4: the sun's bounce in the voxel field (the game's
//! answer to SVOGI, at a fraction of its cost).
//!
//! Every air cell beside a sunlit face is seeded with the light that face
//! throws back, its albedo times the sun on it, and the seed floods a few
//! cells through the air with falloff, as lamp light does. A wall by a sunlit
//! meadow is lit green from below, a doorway warm from the sand in front of
//! it, a cave mouth by the rock the sun reaches. Solids stop it, so it does
//! not leak through a one-cell wall.
//!
//! The field is stored sparsely: per column a window of [`SPAN`] layers round
//! its surface, each cell's colour 5:6:5 in sixteen bits, two to a word. That
//! is what the GPU reads (`planet_surface.wgsl`, `bounce_at`).
//!
//! Directions are in the planet's frame, and the columns are hex prisms: a
//! column's border with a neighbour is the plane midway between their two
//! unit directions, square to the step from one to the other, and a layer is
//! a shell between two spheres (`column::BASE_M` plus the layer's index, metres over `radius`).

use crate::column::{BASE_M, Column, LAYERS};
use crate::terrain::Material;
use glam::Vec3;

/// Layers per column the field holds, from [`window_base`] up.
pub const SPAN: usize = 32;
/// How far under a column's surface its window starts, layers.
pub const BELOW: usize = 12;
/// Words per column: a sixteen-bit cell, two to a word.
pub const WORDS: usize = SPAN / 2;
/// The brightest bounce a cell holds; brighter is clamped.
pub const MAX: f32 = 0.5;
/// What a face throws onto the air beside it, of its own albedo times the sun
/// on it: about the view factor of a wide floor seen from a wall's foot.
const SHARE: f32 = 0.5;
/// How much is left a cell further on, and how many cells it goes.
const FALLOFF: f32 = 0.62;
const REACH: usize = 5;
/// The longest walk toward the sun, cells.
const SUN_WALK: usize = 96;

/// The column tier the field is baked over: what is solid, who neighbours
/// whom (`u32::MAX` off the tier), and where each column stands.
pub struct Region<'a> {
    pub columns: &'a [Column],
    pub neighbors: &'a [[u32; 6]],
    pub centres: &'a [Vec3],
    /// The planet's radius, metres: the layers' shells are over it.
    pub radius: f32,
}

impl Region<'_> {
    pub(crate) fn solid(&self, slot: usize, layer: usize) -> bool {
        self.columns
            .get(slot)
            .is_none_or(|c| layer < LAYERS && c.solid(layer))
    }
}

/// The first layer of a column's window: [`BELOW`] under its surface.
pub fn window_base(column: &Column) -> usize {
    column
        .surface()
        .unwrap_or(BELOW)
        .saturating_sub(BELOW)
        .min(LAYERS - SPAN)
}

/// A face's colour as the bounce takes it: the mean of its tile.
pub fn albedo(material: Material) -> Vec3 {
    match material {
        Material::Grass => Vec3::new(0.12, 0.32, 0.075),
        Material::JungleGrass => Vec3::new(0.085, 0.24, 0.060),
        Material::DryGrass => Vec3::new(0.32, 0.36, 0.22),
        Material::Sand => Vec3::new(0.61, 0.48, 0.25),
        Material::Stone | Material::Rock | Material::Ore => Vec3::new(0.31, 0.34, 0.33),
        Material::Snow => Vec3::new(0.80, 0.90, 0.91),
        Material::Dirt | Material::Soil => Vec3::new(0.466, 0.342, 0.255),
        _ => Vec3::splat(0.35),
    }
}

/// Whether the sun reaches the point `from` (planet frame), in column `slot`,
/// along `toward`: the walk out of each hex prism through whichever of its
/// side planes or layer shells the ray meets first, until a solid cell, the
/// open sky over the tier, or the tier's edge (past which the land is taken
/// to be open).
pub fn sees(region: &Region, slot: usize, from: Vec3, toward: Vec3) -> bool {
    clear(region, slot, from, toward, f32::INFINITY)
}

/// Whether nothing solid stands within `reach` metres of `from` (in column
/// `slot`) along `toward`: [`sees`], stopped short (`lighting-model` L7's line
/// of sight to a lamp).
pub fn clear(region: &Region, slot: usize, from: Vec3, toward: Vec3, reach: f32) -> bool {
    let mut walked = 0.0;
    let mut slot = slot;
    let mut p = from;
    let Some(mut layer) = layer_of(region, p) else {
        return true;
    };
    for _ in 0..SUN_WALK {
        let c = region.centres[slot];
        let mut t_exit = f32::INFINITY;
        let mut next = (slot, layer as i64);
        for &n in &region.neighbors[slot] {
            let Some(nc) = region.centres.get(n as usize) else {
                continue;
            };
            // The border, through the point midway between the two columns
            // at this height: through the planet's centre it would hang on
            // the radial part of `plane`, which is below f32's resolution.
            let plane = *nc - c;
            let mid = (c + *nc) * (0.5 * p.length());
            let dn = toward.dot(plane);
            if dn > 1e-9 {
                let t = (-(p - mid).dot(plane) / dn).max(0.0);
                if t < t_exit {
                    t_exit = t;
                    next = (n as usize, layer as i64);
                }
            }
        }
        let b = p.dot(toward);
        let pp = p.dot(p);
        let r_hi = region.radius + BASE_M as f32 + layer as f32 + 1.0;
        let up = b * b - (pp - r_hi * r_hi);
        if up >= 0.0 {
            let t = -b + up.sqrt();
            if t >= 0.0 && t < t_exit {
                t_exit = t;
                next = (slot, layer as i64 + 1);
            }
        }
        let r_lo = r_hi - 1.0;
        let down = b * b - (pp - r_lo * r_lo);
        if down >= 0.0 && layer > 0 {
            let t = -b - down.sqrt();
            if t >= 0.0 && t < t_exit {
                t_exit = t;
                next = (slot, layer as i64 - 1);
            }
        }
        if !t_exit.is_finite() {
            return true;
        }
        walked += t_exit;
        if walked >= reach {
            return true;
        }
        p += toward * (t_exit + 1e-4);
        slot = next.0;
        if next.1 < 0 || next.1 as usize >= LAYERS {
            return true;
        }
        layer = next.1 as usize;
        if region.solid(slot, layer) {
            return false;
        }
    }
    true
}

pub(crate) fn layer_of(region: &Region, p: Vec3) -> Option<usize> {
    let index = (p.length() - region.radius - BASE_M as f32).floor();
    (index >= 0.0 && (index as usize) < LAYERS).then_some(index as usize)
}

/// The bounce over a tier for a sun toward `sun` (unit, planet frame) of
/// colour `light` (after the air): per column its window's base layer, and
/// its [`WORDS`] words.
pub struct Field {
    pub bases: Vec<u32>,
    pub words: Vec<u32>,
}

impl Field {
    /// One cell's bounce, or `None` outside its column's window.
    pub fn at(&self, slot: usize, layer: usize) -> Option<Vec3> {
        let base = *self.bases.get(slot)? as usize;
        let k = layer.checked_sub(base).filter(|&k| k < SPAN)?;
        let word = self.words[slot * WORDS + k / 2];
        Some(unpack((word >> ((k % 2) * 16)) as u16))
    }
}

fn pack(c: Vec3) -> u16 {
    let c = (c / MAX).clamp(Vec3::ZERO, Vec3::ONE);
    let r = (c.x * 31.0).round() as u16;
    let g = (c.y * 63.0).round() as u16;
    let b = (c.z * 31.0).round() as u16;
    (r << 11) | (g << 5) | b
}

fn unpack(v: u16) -> Vec3 {
    Vec3::new(
        ((v >> 11) & 31) as f32 / 31.0,
        ((v >> 5) & 63) as f32 / 63.0,
        (v & 31) as f32 / 31.0,
    ) * MAX
}

/// Bake the field. Seeds every air cell in a window beside a face the sun
/// reaches, then floods [`REACH`] cells through the air at [`FALLOFF`] a cell,
/// keeping the brightest that arrives.
#[allow(clippy::needless_range_loop)]
pub fn bake(region: &Region, sun: Vec3, light: Vec3) -> Field {
    let n = region.columns.len();
    let bases: Vec<u32> = region
        .columns
        .iter()
        .map(|c| window_base(c) as u32)
        .collect();
    let mut cells = vec![Vec3::ZERO; n * SPAN];
    let index = |slot: usize, layer: usize| -> Option<usize> {
        let base = *bases.get(slot)? as usize;
        let k = layer.checked_sub(base).filter(|&k| k < SPAN)?;
        Some(slot * SPAN + k)
    };
    if light.max_element() > 0.0 {
        for slot in 0..n {
            let base = bases[slot] as usize;
            let up = region.centres[slot];
            for layer in base..base + SPAN {
                if !region.solid(slot, layer) {
                    continue;
                }
                let albedo = albedo(region.columns[slot].material(layer));
                let shell = region.radius + BASE_M as f32 + layer as f32;
                // The top face: air over it.
                if layer + 1 < LAYERS && !region.solid(slot, layer + 1) {
                    let facing = up.dot(sun);
                    let at = up * (shell + 1.0 + 0.05);
                    if facing > 0.0
                        && sees(region, slot, at, sun)
                        && let Some(i) = index(slot, layer + 1)
                    {
                        cells[i] += albedo * light * facing * SHARE;
                    }
                }
                // The side faces: air beside it, at the same layer.
                for &neighbor in &region.neighbors[slot] {
                    let neighbor = neighbor as usize;
                    if neighbor >= n || region.solid(neighbor, layer) {
                        continue;
                    }
                    let across = region.centres[neighbor] - up;
                    let normal = (across - up * across.dot(up)).normalize_or_zero();
                    let facing = normal.dot(sun);
                    let mid = (up + region.centres[neighbor]).normalize() * (shell + 0.5);
                    if facing > 0.0
                        && sees(region, neighbor, mid + normal * 0.05, sun)
                        && let Some(i) = index(neighbor, layer)
                    {
                        cells[i] += albedo * light * facing * SHARE;
                    }
                }
            }
        }
    }
    // The flood: a few relaxation sweeps through the air, each cell taking the
    // brightest of its own and its neighbours' times the falloff.
    for _ in 0..REACH {
        let before = cells.clone();
        for slot in 0..n {
            let base = bases[slot] as usize;
            for k in 0..SPAN {
                let layer = base + k;
                if region.solid(slot, layer) {
                    continue;
                }
                let mut best = before[slot * SPAN + k];
                let mut take = |s: usize, l: usize| {
                    if let Some(i) = index(s, l)
                        && !region.solid(s, l)
                    {
                        best = best.max(before[i] * FALLOFF);
                    }
                };
                if layer > 0 {
                    take(slot, layer - 1);
                }
                take(slot, layer + 1);
                for &neighbor in &region.neighbors[slot] {
                    if (neighbor as usize) < n {
                        take(neighbor as usize, layer);
                    }
                }
                cells[slot * SPAN + k] = best;
            }
        }
    }
    let mut words = vec![0u32; n * WORDS];
    for (i, c) in cells.iter().enumerate() {
        let slot = i / SPAN;
        let k = i % SPAN;
        if region.solid(slot, bases[slot] as usize + k) {
            continue;
        }
        words[slot * WORDS + k / 2] |= (pack(*c) as u32) << ((k % 2) * 16);
    }
    Field { bases, words }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Column;

    /// A strip of columns along x, a metre apart at the north pole of a big
    /// planet: each neighbours the next.
    fn strip(columns: Vec<Column>) -> (Vec<Column>, Vec<[u32; 6]>, Vec<Vec3>) {
        let n = columns.len();
        let radius = 100_000.0f32;
        let centres: Vec<Vec3> = (0..n)
            .map(|i| Vec3::new(i as f32 * 2.0 / radius, 1.0, 0.0).normalize())
            .collect();
        let neighbors = (0..n)
            .map(|i| {
                let mut t = [u32::MAX; 6];
                if i > 0 {
                    t[0] = i as u32 - 1;
                }
                if i + 1 < n {
                    t[1] = i as u32 + 1;
                }
                t
            })
            .collect();
        (columns, neighbors, centres)
    }

    fn ground(top: usize, material: Material) -> Column {
        let mut c = Column::bedrock();
        for layer in 0..=top {
            c.set(layer, material);
        }
        c
    }

    const TOP: usize = 150;

    #[test]
    fn a_sunlit_floor_lights_the_air_over_it_in_its_own_colour() {
        let (columns, neighbors, centres) = strip(vec![ground(TOP, Material::Sand); 3]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
            centres: &centres,
            radius: 100_000.0,
        };
        let field = bake(&region, Vec3::Y, Vec3::ONE);
        let over = field.at(1, TOP + 1).unwrap();
        assert!(over.x > over.z && over.x > 0.2, "{over:?}");
        // Falls off with height, and the rock under it holds none.
        assert!(field.at(1, TOP + 4).unwrap().x < over.x);
        assert_eq!(field.at(1, TOP).unwrap(), Vec3::ZERO);
        // At night there is none.
        assert_eq!(
            bake(&region, Vec3::Y, Vec3::ZERO).at(1, TOP + 1).unwrap(),
            Vec3::ZERO
        );
    }

    #[test]
    fn it_stops_at_a_solid_wall() {
        // A sunlit floor at 0, a wall the full height of the window at 1, and
        // a sealed room at 2 whose roof keeps the sun off its own floor.
        let mut wall = ground(TOP, Material::Stone);
        for layer in TOP..TOP + 8 {
            wall.set(layer, Material::Stone);
        }
        let mut room = wall.clone();
        for layer in TOP + 1..TOP + 4 {
            room.set(layer, Material::Air);
        }
        let (columns, neighbors, centres) = strip(vec![ground(TOP, Material::Sand), wall, room]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
            centres: &centres,
            radius: 100_000.0,
        };
        let field = bake(&region, Vec3::Y, Vec3::ONE);
        assert!(field.at(0, TOP + 1).unwrap().x > 0.1);
        assert_eq!(field.at(2, TOP + 1).unwrap(), Vec3::ZERO);
    }

    #[test]
    fn a_slab_shades_the_floor_under_it_from_the_sun() {
        let mut roofed = ground(TOP, Material::Sand);
        roofed.set(TOP + 4, Material::Stone);
        let (columns, neighbors, centres) = strip(vec![roofed]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
            centres: &centres,
            radius: 100_000.0,
        };
        let shell = region.radius + BASE_M as f32 + TOP as f32 + 1.05;
        assert!(!sees(&region, 0, centres[0] * shell, Vec3::Y));
        let open = ground(TOP, Material::Sand);
        let (columns, ..) = strip(vec![open]);
        let region = Region {
            columns: &columns,
            ..region
        };
        assert!(sees(&region, 0, centres[0] * shell, Vec3::Y));
    }
}
