//! `lighting-model` L7: the lamp bake, with shadows and a bounce (owner:
//! "Can we increase the accuracy of lamps vertex color bake? Have some bounce
//! and shadow shading?").
//!
//! The flood fill lights a cell by how many steps it is from a lamp, round
//! corners: no direction, no shadow behind a pillar, no bounce. This bake
//! keeps, for every air cell within [`REACH_M`] of a lamp (in the same
//! per-column window as the sun's bounce, [`crate::bounce`]):
//! - which lamps it can SEE, by a walk over the columns from the cell to the
//!   lamp: up to [`SEEN`] of them, the brightest first. The shader lights a
//!   pixel from exactly those lamps, by N.L and the windowed inverse square,
//!   so the falloff is smooth and a lamp behind a wall adds nothing;
//! - one bounce: the lamp light on the faces round the cell, in their own
//!   colour, flooded a few cells, so a lantern by a plaster wall lights the
//!   room cream and one on grass lights its post green.
//!
//! The GPU reads it as `planet_surface.wgsl`'s `lamp_cell`.

use crate::bounce::{self, Region, SPAN};
use crate::column::{BASE_M, LAYERS};
use glam::Vec3;

/// How far a lamp reaches, metres (the mockup's 16).
pub const REACH_M: f32 = 16.0;
/// The most lamps a cell keeps.
pub const SEEN: usize = 4;
/// The most lamps a bake takes; an index of `NONE` is no lamp.
pub const MAX_LAMPS: usize = 255;
pub const NONE: u32 = 255;
/// Words per cell: the lamps it sees, then its bounce and flags.
pub const CELL_WORDS: usize = 2;
pub const WORDS: usize = SPAN * CELL_WORDS;
/// The brightest bounce a cell holds.
pub const BOUNCE_MAX: f32 = 1.0;
/// Set in a cell's second word's high half: some lamp of the bake is in
/// reach, so the bake, not the flood, lights it.
pub const IN_REACH: u32 = 1 << 16;
/// The mockup's lamp gain, and the bounce's share and spread.
pub const GAIN: f32 = 2.4;
const SHARE: f32 = 0.5;
const FALLOFF: f32 = 0.62;
const SPREAD: usize = 4;

/// One lamp of the bake: where it stands (planet frame) and its level over
/// a torch's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    pub slot: usize,
    pub at: Vec3,
    pub level: f32,
}

/// The light one lamp gives a point `d` metres off it whose face turns `nl`
/// toward it: the mockup's windowed inverse square.
pub fn falloff(d: f32, nl: f32) -> f32 {
    if d > REACH_M || nl <= 0.0 {
        return 0.0;
    }
    let w = (1.0 - (d / REACH_M).powi(4)).max(0.0);
    nl * w * w / (1.0 + (d / 2.2).powi(2)) * GAIN
}

/// The bake: the lamps as the GPU reads them, and per column its window's
/// base and its cells.
pub struct LampField {
    pub lamps: Vec<Lamp>,
    pub bases: Vec<u32>,
    pub words: Vec<u32>,
}

impl LampField {
    fn cell(&self, slot: usize, layer: usize) -> Option<usize> {
        let base = *self.bases.get(slot)? as usize;
        let k = layer.checked_sub(base).filter(|&k| k < SPAN)?;
        Some((slot * SPAN + k) * CELL_WORDS)
    }

    /// The lamps a cell sees, by index into `lamps`.
    pub fn seen(&self, slot: usize, layer: usize) -> Vec<usize> {
        let Some(i) = self.cell(slot, layer) else {
            return Vec::new();
        };
        (0..SEEN)
            .map(|k| (self.words[i] >> (8 * k)) & 0xff)
            .filter(|&l| l != NONE)
            .map(|l| l as usize)
            .collect()
    }

    /// A cell's bounce, and whether a lamp is in reach of it.
    pub fn bounce(&self, slot: usize, layer: usize) -> Option<(Vec3, bool)> {
        let i = self.cell(slot, layer)?;
        let w = self.words[i + 1];
        let v = (w & 0xffff) as u16;
        let c = Vec3::new(
            ((v >> 11) & 31) as f32 / 31.0,
            ((v >> 5) & 63) as f32 / 63.0,
            (v & 31) as f32 / 31.0,
        ) * BOUNCE_MAX;
        Some((c, w & IN_REACH != 0))
    }
}

fn shell(region: &Region, layer: usize) -> f32 {
    region.radius + BASE_M as f32 + layer as f32
}

/// The middle of a cell, planet frame.
pub fn cell_centre(region: &Region, slot: usize, layer: usize) -> Vec3 {
    region.centres[slot] * (shell(region, layer) + 0.5)
}

pub fn bake(region: &Region, lamps: &[Lamp]) -> LampField {
    let lamps: Vec<Lamp> = lamps.iter().copied().take(MAX_LAMPS).collect();
    let n = region.columns.len();
    let bases: Vec<u32> = region
        .columns
        .iter()
        .map(|c| bounce::window_base(c) as u32)
        .collect();
    // Per cell, the lamps it sees with the light each gives it (facing it).
    let mut seen: Vec<Vec<(f32, usize)>> = vec![Vec::new(); n * SPAN];
    let mut reached = vec![false; n * SPAN];
    let reach_angle = (REACH_M + 4.0) / region.radius;
    for (index, lamp) in lamps.iter().enumerate() {
        let up = lamp.at.normalize_or_zero();
        for slot in 0..n {
            let c = region.centres[slot];
            if c.dot(up) < reach_angle.cos() {
                continue;
            }
            let base = bases[slot] as usize;
            for k in 0..SPAN {
                let layer = base + k;
                if layer >= LAYERS || region.solid(slot, layer) {
                    continue;
                }
                let p = cell_centre(region, slot, layer);
                let to = lamp.at - p;
                let d = to.length();
                if d > REACH_M {
                    continue;
                }
                reached[slot * SPAN + k] = true;
                let light = falloff(d, 1.0) * lamp.level;
                if d < 0.5 || bounce::clear(region, slot, p, to / d, d - 0.45) {
                    seen[slot * SPAN + k].push((light, index));
                }
            }
        }
    }
    // The bounce: the lamp light on the faces round each cell (a floor under
    // it, a ceiling over it, walls beside it), in their colour, then spread.
    let mut cells = vec![Vec3::ZERO; n * SPAN];
    let index = |slot: usize, layer: usize| -> Option<usize> {
        let base = *bases.get(slot)? as usize;
        let k = layer.checked_sub(base).filter(|&k| k < SPAN)?;
        Some(slot * SPAN + k)
    };
    for slot in 0..n {
        let base = bases[slot] as usize;
        let up = region.centres[slot];
        for k in 0..SPAN {
            let layer = base + k;
            let here = &seen[slot * SPAN + k];
            if here.is_empty() || region.solid(slot, layer) {
                continue;
            }
            let p = cell_centre(region, slot, layer);
            let mut faces = Vec::with_capacity(8);
            if layer > 0 && region.solid(slot, layer - 1) {
                faces.push((up, region.columns[slot].material(layer - 1)));
            }
            if layer + 1 < LAYERS && region.solid(slot, layer + 1) {
                faces.push((-up, region.columns[slot].material(layer + 1)));
            }
            for &nb in &region.neighbors[slot] {
                let nb = nb as usize;
                if nb < n && region.solid(nb, layer) {
                    let across = up - region.centres[nb];
                    let normal = (across - up * across.dot(up)).normalize_or_zero();
                    faces.push((normal, region.columns[nb].material(layer)));
                }
            }
            let mut sum = Vec3::ZERO;
            for (normal, material) in &faces {
                let mut on_face = 0.0;
                // From the face itself, half a cell behind the cell's middle.
                let on = p - *normal * 0.5;
                for &(_, l) in here {
                    let to = lamps[l].at - on;
                    let d = to.length().max(1e-3);
                    on_face += falloff(d, normal.dot(to / d).max(0.0)) * lamps[l].level;
                }
                sum += bounce::albedo(*material) * on_face;
            }
            cells[slot * SPAN + k] = sum * (SHARE / 6.0) * 2.0;
        }
    }
    for _ in 0..SPREAD {
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
                for &nb in &region.neighbors[slot] {
                    if (nb as usize) < n {
                        take(nb as usize, layer);
                    }
                }
                cells[slot * SPAN + k] = best;
            }
        }
    }
    let mut words = vec![0u32; n * WORDS];
    for i in 0..n * SPAN {
        let mut list = std::mem::take(&mut seen[i]);
        list.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut packed = 0u32;
        for k in 0..SEEN {
            let l = list.get(k).map_or(NONE, |&(_, l)| l as u32);
            packed |= l << (8 * k);
        }
        let c = (cells[i] / BOUNCE_MAX).clamp(Vec3::ZERO, Vec3::ONE);
        let rgb = ((c.x * 31.0).round() as u32) << 11
            | ((c.y * 63.0).round() as u32) << 5
            | (c.z * 31.0).round() as u32;
        words[i * CELL_WORDS] = packed;
        words[i * CELL_WORDS + 1] = rgb | if reached[i] { IN_REACH } else { 0 };
    }
    LampField {
        lamps,
        bases,
        words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Column;
    use crate::terrain::Material;

    const TOP: usize = 150;
    const RADIUS: f32 = 100_000.0;

    fn ground(top: usize, material: Material) -> Column {
        let mut c = Column::bedrock();
        for layer in 1..=top {
            c.set(layer, material);
        }
        c
    }

    fn strip(columns: &[Column]) -> (Vec<[u32; 6]>, Vec<Vec3>) {
        let n = columns.len();
        let centres = (0..n)
            .map(|i| Vec3::new(i as f32 * 2.0 / RADIUS, 1.0, 0.0).normalize())
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
        (neighbors, centres)
    }

    fn lamp_at(region: &Region, slot: usize) -> Lamp {
        Lamp {
            slot,
            at: cell_centre(region, slot, TOP + 1),
            level: 1.0,
        }
    }

    #[test]
    fn a_lamp_behind_a_wall_adds_nothing() {
        let mut wall = ground(TOP, Material::Stone);
        for layer in TOP + 1..TOP + 6 {
            wall.set(layer, Material::Stone);
        }
        let columns = vec![
            ground(TOP, Material::Sand),
            wall,
            ground(TOP, Material::Sand),
            ground(TOP, Material::Sand),
        ];
        let (neighbors, centres) = strip(&columns);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
            centres: &centres,
            radius: RADIUS,
        };
        let field = bake(&region, &[lamp_at(&region, 0)]);
        // Beside it, it is seen; past the wall it is not, though in reach.
        assert_eq!(field.seen(0, TOP + 2), vec![0]);
        assert!(field.seen(2, TOP + 1).is_empty());
        assert!(
            field.bounce(2, TOP + 1).unwrap().1,
            "in reach, so baked dark"
        );
        // High over the wall it is seen again; just over it, the wall's
        // own edge is in the way.
        assert_eq!(field.seen(1, TOP + 12), vec![0]);
        assert!(field.seen(1, TOP + 6).is_empty());
    }

    #[test]
    fn a_face_turned_to_a_lamp_is_brighter_than_one_beside_it() {
        let facing = falloff(2.0, 1.0);
        let grazing = falloff(2.0, 0.2);
        assert!(facing > grazing && grazing > 0.0);
        assert_eq!(falloff(2.0, -0.5), 0.0);
        assert_eq!(falloff(REACH_M + 0.1, 1.0), 0.0);
        // Nearer is brighter.
        assert!(falloff(1.0, 1.0) > falloff(4.0, 1.0));
    }

    #[test]
    fn a_lamp_on_sand_bounces_sand_coloured() {
        let columns = vec![ground(TOP, Material::Sand); 3];
        let (neighbors, centres) = strip(&columns);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
            centres: &centres,
            radius: RADIUS,
        };
        let field = bake(&region, &[lamp_at(&region, 1)]);
        let (c, reached) = field.bounce(0, TOP + 2).unwrap();
        assert!(reached && c.x > c.z && c.x > 0.0, "{c:?}");
    }
}
