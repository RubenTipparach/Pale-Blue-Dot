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
use pbd_core::frustum::Planes;

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
    /// What each site reads off the shown pair's two maps (cover, top,
    /// optical depth), kept while the pair is the same (`lighting-perf`
    /// 2.4): the 8 map taps a site were most of a build, and the pair
    /// changes only when the weather steps, while `t` moves every frame.
    looks: Vec<([f32; 3], [f32; 3])>,
    /// The pair `looks` was read from: `WeatherMapsNow::generation`, which
    /// moves whenever the shown pair does.
    looks_of: Option<u64>,
    /// The far-to-near order's scratch, kept to save its allocation.
    order: Vec<(f32, u32)>,
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
            looks: Vec::new(),
            looks_of: None,
            order: Vec::new(),
        }
    }

    pub fn spacing_m(&self) -> f32 {
        self.spacing_m
    }

    /// The puffs under the shown weather that `view` can see (when given:
    /// the frustum in body-local space), sorted far to near from `eye`
    /// (body-local), into `out`. `pair` names the two maps
    /// (`WeatherMapsNow::generation`): what the sites read off them is kept
    /// until it changes.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        pair: u64,
        from: &WeatherMaps,
        to: &WeatherMaps,
        t: f32,
        eye: Vec3,
        view: Option<&Planes>,
        settings: &WeatherSettings,
        out: &mut Vec<PuffInstance>,
    ) -> PuffCounts {
        out.clear();
        self.puffs.clear();
        let mut counts = PuffCounts::default();
        let texels = 6 * MAP_SIZE * MAP_SIZE;
        if to.cloud.len() != texels {
            return counts;
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
        if self.looks_of != Some(pair) || self.looks.len() != self.sites.len() {
            self.looks = self
                .taps
                .iter()
                .map(|taps| {
                    let read = |maps: &WeatherMaps| {
                        let v = taps.iter().fold([0.0f32; 4], |mut acc, &(i, w)| {
                            let v = maps.cloud[i as usize];
                            for k in 0..4 {
                                acc[k] += v[k] * w;
                            }
                            acc
                        });
                        [v[0], v[1], v[3]]
                    };
                    (read(from), read(to))
                })
                .collect();
            self.looks_of = Some(pair);
        }
        for (site, (a, b)) in self.sites.iter().zip(&self.looks) {
            let mix = |k: usize| a[k] + (b[k] - a[k]) * t;
            let look = Look {
                cover: mix(0),
                top: mix(1),
                optical_depth: mix(2),
            };
            cloud_puffs::puffs_at(site, look, &layer, &mut self.puffs);
        }
        counts.built = self.puffs.len();
        // Past the horizon nothing of a puff shows (design 3b).
        let sea = crate::planet::PLANET_RADIUS;
        self.puffs
            .retain(|p| cloud_puffs::over_horizon(eye, p, sea));
        counts.over_horizon = self.puffs.len();
        // Nor outside the view: culled only when the box round every
        // orientation of its billboard misses the frustum
        // (`puff-frustum-cull`).
        if let Some(view) = view {
            self.puffs
                .retain(|p| view.touches_box(p.centre, Vec3::splat(cloud_puffs::bound_radius(p))));
        }
        counts.in_view = self.puffs.len();
        // Far to near: the blend is "over", so the nearest is drawn last.
        let order = &mut self.order;
        order.clear();
        order.extend(
            self.puffs
                .iter()
                .enumerate()
                .map(|(i, p)| (p.centre.distance_squared(eye), i as u32)),
        );
        order.sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        out.extend(
            order
                .iter()
                .map(|&(_, i)| PuffInstance::from(&self.puffs[i as usize])),
        );
        counts
    }
}

/// How many puffs one build made, kept past the horizon, and kept in view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PuffCounts {
    pub built: usize,
    pub over_horizon: usize,
    pub in_view: usize,
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

    /// `lighting-perf` 2.4: a build that reuses what the sites read off the
    /// pair is the build that reads it afresh, bit for bit, at any `t`, and
    /// a new pair is read anew.
    #[test]
    fn a_kept_reading_builds_the_same_puffs() {
        let texels = 6 * MAP_SIZE * MAP_SIZE;
        let maps = |k: f32| WeatherMaps {
            cloud: (0..texels)
                .map(|i| {
                    let x = (i as f32 * 0.37 + k).sin() * 0.5 + 0.5;
                    [x, 0.3 + 0.2 * x, 0.0, 10.0 + 20.0 * x]
                })
                .collect(),
            wind: vec![[0.0; 4]; texels],
        };
        let (a, b, c) = (maps(0.0), maps(1.3), maps(2.9));
        let settings = WeatherSettings::default();
        let eye = Vec3::Y * (CLOUD_RADIUS - 400.0);
        let mut kept = PuffSites::new(250.0);
        let mut out = Vec::new();
        kept.build(1, &a, &b, 0.9, eye, None, &settings, &mut out);
        for (pair, from, to, t) in [(1, &a, &b, 0.3), (1, &a, &b, 0.71), (2, &b, &c, 0.2)] {
            kept.build(pair, from, to, t, eye, None, &settings, &mut out);
            let mut fresh_out = Vec::new();
            PuffSites::new(250.0).build(pair, from, to, t, eye, None, &settings, &mut fresh_out);
            assert!(!out.is_empty());
            assert_eq!(
                bytemuck::cast_slice::<_, u8>(&out),
                bytemuck::cast_slice::<_, u8>(&fresh_out)
            );
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
        sites.build(0, &maps, &maps, 0.0, eye, None, &settings, &mut out);
        assert!(out.is_empty(), "clear sky drew {}", out.len());
        for v in &mut maps.cloud {
            *v = [0.6, 0.2, 0.0, 30.0];
        }
        let counts = sites.build(1, &maps, &maps, 0.0, eye, None, &settings, &mut out);
        // Counted as built: from the layer most of them lie past the horizon
        // (`space-clouds`; the cull came after this test).
        let share = counts.built as f32 / sites.sites.len() as f32;
        assert!(share > 0.5 && share < 0.75, "{share}");
        let d: Vec<f32> = out
            .iter()
            .map(|p| Vec3::from_slice(&p.centre_seed[..3]).distance(eye))
            .collect();
        assert!(d.windows(2).all(|w| w[0] >= w[1]));
    }
}
