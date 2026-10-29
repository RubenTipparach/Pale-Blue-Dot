//! The puffs on the GPU (`cloud-puffs`): every frame, each site reads the
//! weather maps the view is showing (the shown pair mixed by `t`, as the GPU's
//! blend mixes them), `pbd_core::cloud_puffs` turns that into billboards, and
//! they are sorted far to near from the eye for one blended, instanced draw.
//!
//! The maps are read on the CPU from the same `WeatherMapsNow` the GPU blend
//! is uploaded from, so a puff stands where the march would have drawn cloud.

use crate::atmosphere::{MAP_SIZE, WeatherMaps};
use crate::config::WeatherSettings;
use crate::sky::{CLOUD_RADIUS, CLOUD_THICKNESS};
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use pbd_core::cloud_puffs::{self, Layer, Look, Puff, Site};

/// One puff as `PuffInstance` in `water.wgsl` reads it. 48 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub(super) struct PuffInstance {
    /// xyz the centre, body-local metres; w the noise seed.
    pub centre_seed: [f32; 4],
    /// x radius across, y radius up, z darkness, w opacity.
    pub shape: [f32; 4],
    /// x the column's base radius, y its top radius; zw spare.
    pub layer: [f32; 4],
}

impl From<&Puff> for PuffInstance {
    fn from(p: &Puff) -> Self {
        Self {
            centre_seed: [p.centre.x, p.centre.y, p.centre.z, p.seed],
            shape: [p.radius_across_m, p.radius_up_m, p.dark, p.opacity],
            layer: [p.base_radius_m, p.top_radius_m, 0.0, 0.0],
        }
    }
}

/// The four texels a direction reads on a cube map, and their bilinear
/// weights, within its face.
type Taps = [(u32, f32); 4];

/// The sites for one spacing, and where each reads the maps. Made once and
/// remade only if the spacing changes.
pub(super) struct PuffSites {
    spacing_m: f32,
    sites: Vec<Site>,
    taps: Vec<Taps>,
    puffs: Vec<Puff>,
}

impl PuffSites {
    pub fn new(spacing_m: f32) -> Self {
        let sites = cloud_puffs::sites(crate::planet::PLANET_RADIUS, spacing_m);
        let taps = sites
            .iter()
            .map(|s| cube_taps(s.direction, MAP_SIZE))
            .collect();
        Self {
            spacing_m,
            sites,
            taps,
            puffs: Vec::new(),
        }
    }

    pub fn spacing_m(&self) -> f32 {
        self.spacing_m
    }

    /// The puffs under the shown weather, sorted far to near from `eye`
    /// (body-local), into `out`.
    pub fn build(
        &mut self,
        from: &WeatherMaps,
        to: &WeatherMaps,
        t: f32,
        eye: Vec3,
        settings: &WeatherSettings,
        out: &mut Vec<PuffInstance>,
    ) {
        out.clear();
        self.puffs.clear();
        let texels = 6 * MAP_SIZE * MAP_SIZE;
        if to.cloud.len() != texels {
            return;
        }
        let from = if from.cloud.len() == texels { from } else { to };
        let layer = Layer {
            base_radius_m: CLOUD_RADIUS,
            thickness_m: CLOUD_THICKNESS,
            spacing_m: self.spacing_m,
            opacity: settings.cloud_puff_opacity,
            deck_floor: settings.cloud_deck_floor,
            optical_depth_scale: settings.cloud_puff_optical_depth,
        };
        let t = t.clamp(0.0, 1.0);
        for (site, taps) in self.sites.iter().zip(&self.taps) {
            let read = |maps: &WeatherMaps| {
                taps.iter().fold([0.0f32; 4], |mut acc, &(i, w)| {
                    let v = maps.cloud[i as usize];
                    for k in 0..4 {
                        acc[k] += v[k] * w;
                    }
                    acc
                })
            };
            let a = read(from);
            let b = read(to);
            let mix = |k: usize| a[k] + (b[k] - a[k]) * t;
            let look = Look {
                cover: mix(0),
                top: mix(1),
                optical_depth: mix(3),
            };
            cloud_puffs::puffs_at(site, look, &layer, &mut self.puffs);
        }
        // Past the horizon nothing of a puff shows (design 3b).
        let sea = crate::planet::PLANET_RADIUS;
        self.puffs
            .retain(|p| cloud_puffs::over_horizon(eye, p, sea));
        // Far to near: the blend is "over", so the nearest is drawn last.
        let mut order: Vec<(f32, u32)> = self
            .puffs
            .iter()
            .enumerate()
            .map(|(i, p)| (p.centre.distance_squared(eye), i as u32))
            .collect();
        order.sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        out.extend(
            order
                .iter()
                .map(|&(_, i)| PuffInstance::from(&self.puffs[i as usize])),
        );
    }
}

/// The texels `direction` reads on a cube map of `size`, the inverse of
/// `atmosphere::cube_direction`, bilinear within the face (clamped at its
/// edges).
fn cube_taps(direction: Vec3, size: usize) -> Taps {
    let d = direction;
    let a = d.abs();
    let (face, s, t) = if a.x >= a.y && a.x >= a.z {
        if d.x > 0.0 {
            (0, -d.z / a.x, -d.y / a.x)
        } else {
            (1, d.z / a.x, -d.y / a.x)
        }
    } else if a.y >= a.z {
        if d.y > 0.0 {
            (2, d.x / a.y, d.z / a.y)
        } else {
            (3, d.x / a.y, -d.z / a.y)
        }
    } else if d.z > 0.0 {
        (4, d.x / a.z, -d.y / a.z)
    } else {
        (5, -d.x / a.z, -d.y / a.z)
    };
    let n = size as f32;
    let column = ((s + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
    let row = ((t + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
    let (c0, r0) = (column.floor() as usize, row.floor() as usize);
    let (c1, r1) = ((c0 + 1).min(size - 1), (r0 + 1).min(size - 1));
    let (fc, fr) = (column - c0 as f32, row - r0 as f32);
    let at = |r: usize, c: usize| (face * size * size + r * size + c) as u32;
    [
        (at(r0, c0), (1.0 - fc) * (1.0 - fr)),
        (at(r0, c1), fc * (1.0 - fr)),
        (at(r1, c0), (1.0 - fc) * fr),
        (at(r1, c1), fc * fr),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atmosphere::cube_direction;

    #[test]
    fn a_texels_own_direction_reads_that_texel() {
        for face in 0..6 {
            for (row, column) in [(0, 0), (5, 40), (63, 63), (31, 2)] {
                let taps = cube_taps(cube_direction(face, row, column, MAP_SIZE), MAP_SIZE);
                let best = taps.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
                assert_eq!(
                    best.0 as usize,
                    face * MAP_SIZE * MAP_SIZE + row * MAP_SIZE + column,
                    "face {face} row {row} column {column}"
                );
                assert!(best.1 > 0.99);
            }
        }
    }

    #[test]
    fn puffs_follow_the_map_and_come_far_to_near() {
        let texels = 6 * MAP_SIZE * MAP_SIZE;
        let mut maps = WeatherMaps {
            cloud: vec![[0.0; 4]; texels],
            wind: vec![[0.0; 4]; texels],
        };
        let mut sites = PuffSites::new(250.0);
        // No lanes, so one puff a carrying site.
        let settings = WeatherSettings {
            cloud_deck_floor: 0.0,
            ..default()
        };
        let eye = Vec3::Y * (CLOUD_RADIUS + 50.0);
        let mut out = Vec::new();
        sites.build(&maps, &maps, 0.0, eye, &settings, &mut out);
        assert!(out.is_empty(), "clear sky drew {}", out.len());
        for v in &mut maps.cloud {
            *v = [0.6, 0.2, 0.0, 30.0];
        }
        sites.build(&maps, &maps, 0.0, eye, &settings, &mut out);
        let share = out.len() as f32 / sites.sites.len() as f32;
        assert!(share > 0.5 && share < 0.75, "{share}");
        let d: Vec<f32> = out
            .iter()
            .map(|p| Vec3::from_slice(&p.centre_seed[..3]).distance(eye))
            .collect();
        assert!(d.windows(2).all(|w| w[0] >= w[1]));
    }
}
