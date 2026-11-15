//! The lights of the night side seen from afar (`night-lights-from-space`).
//!
//! From altitude a town is not its lanterns and windows, which stand only
//! within a kilometre or so, but a warm cluster of points on the dark side of
//! the planet. Each settlement on the world's saved site list is one such far
//! light, sized and brightened by its kind; so is each group of the player's
//! own lights that passes a threshold. Everything here is derived from what
//! the save holds (the site list, the edits) and nothing here is saved.
//!
//! The terrain pass draws them (`planet_surface.wgsl`, `far_night_lights`);
//! this module decides where they are, how big, and how bright they are now.

use crate::sites::{Site, SiteKind};
use crate::sky_light;
use glam::Vec3;
use serde::Deserialize;
use std::collections::BTreeMap;

/// How a kind of settlement shows from afar.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct KindLight {
    /// How far from its centre its points reach, metres.
    pub radius_m: f32,
    /// Its brightness at full night, 0..; a walled town is one.
    pub brightness: f32,
}

/// `assets/config/night_lights.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct NightLightsConfig {
    /// Every kind of site, and how it shows. A kind with no brightness (the
    /// cave town, under its rock) shows nothing.
    pub kinds: BTreeMap<SiteKind, KindLight>,
    /// The capital's radius and brightness, times its kind's.
    pub capital_radius: f32,
    pub capital_brightness: f32,
    /// The grid the points of a town are scattered on, metres: about one lit
    /// point per this many metres square at its centre.
    pub town_spacing_m: f32,
    /// Within this distance of the camera a far light is gone, and beyond
    /// `near_in_m` it is whole, metres. A town stands within
    /// `towns::STAND_M` (1200 m), so its real lamps are drawn where its far
    /// light is not.
    pub near_out_m: f32,
    pub near_in_m: f32,
    /// The least number of the player's lights in one group that shows from
    /// afar.
    pub player_threshold: u32,
    /// The cube of the grid the player's lights are grouped by, metres: lights
    /// in touching cubes are one group.
    pub player_link_m: f32,
    /// A group's least radius, metres.
    pub player_min_radius_m: f32,
    /// A group's brightness when it has `player_full_count` lights or more;
    /// fewer is a share of it.
    pub player_brightness: f32,
    pub player_full_count: u32,
    /// The points' grid in a group of the player's lights, metres.
    pub player_spacing_m: f32,
}

impl NightLightsConfig {
    pub fn validate(&self) -> Result<(), String> {
        let positive = |name: &str, v: f32| {
            if v.is_finite() && v > 0.0 {
                Ok(())
            } else {
                Err(format!("{name} must be finite and above zero, not {v}"))
            }
        };
        let not_negative = |name: &str, v: f32| {
            if v.is_finite() && v >= 0.0 {
                Ok(())
            } else {
                Err(format!("{name} must be finite and not below zero, not {v}"))
            }
        };
        for kind in SiteKind::ALL {
            let Some(light) = self.kinds.get(&kind) else {
                return Err(format!("kinds has no {kind:?}"));
            };
            not_negative(&format!("kinds.{kind:?}.radius_m"), light.radius_m)?;
            not_negative(&format!("kinds.{kind:?}.brightness"), light.brightness)?;
        }
        positive("capital_radius", self.capital_radius)?;
        not_negative("capital_brightness", self.capital_brightness)?;
        positive("town_spacing_m", self.town_spacing_m)?;
        not_negative("near_out_m", self.near_out_m)?;
        if !(self.near_in_m.is_finite() && self.near_in_m > self.near_out_m) {
            return Err(format!(
                "near_in_m ({}) must be beyond near_out_m ({})",
                self.near_in_m, self.near_out_m
            ));
        }
        if self.player_threshold == 0 {
            return Err("player_threshold must be at least one".into());
        }
        positive("player_link_m", self.player_link_m)?;
        positive("player_min_radius_m", self.player_min_radius_m)?;
        not_negative("player_brightness", self.player_brightness)?;
        if self.player_full_count == 0 {
            return Err("player_full_count must be at least one".into());
        }
        positive("player_spacing_m", self.player_spacing_m)?;
        Ok(())
    }
}

/// One far light: a cluster of points of light round a place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FarLight {
    /// Its centre, a unit vector in the planet's frame.
    pub direction: Vec3,
    /// How far its points reach, metres.
    pub radius_m: f32,
    /// Its brightness at full night, far off.
    pub brightness: f32,
    /// What scatters its points, so two towns are not the same pattern.
    pub seed: u32,
    /// Its points' grid, metres.
    pub spacing_m: f32,
}

/// A settlement's far light, or `None` for a kind that shows nothing.
pub fn site_light(site: &Site, config: &NightLightsConfig) -> Option<FarLight> {
    let kind = config.kinds.get(&site.kind)?;
    let (mut radius_m, mut brightness) = (kind.radius_m, kind.brightness);
    if site.capital {
        radius_m *= config.capital_radius;
        brightness *= config.capital_brightness;
    }
    (radius_m > 0.0 && brightness > 0.0 && site.direction.length_squared() > 0.5).then_some(
        FarLight {
            direction: site.direction.normalize(),
            radius_m,
            brightness,
            seed: mix(site.id ^ 0x7369_7465),
            spacing_m: config.town_spacing_m,
        },
    )
}

/// How bright a far light is now: its brightness, times the sky's night term
/// where it stands (`sky_light::night`, so it comes on as the sky darkens
/// there), times how far the camera is from it (gone within `near_out_m`,
/// whole past `near_in_m`). `sun` is a unit vector to the sun and `camera`
/// the eye, both in the planet's frame; `radius_m` is the planet's.
pub fn strength(
    light: &FarLight,
    sun: Vec3,
    camera: Vec3,
    radius_m: f32,
    config: &NightLightsConfig,
) -> f32 {
    let night = sky_light::night(light.direction.dot(sun));
    if night <= 0.0 {
        return 0.0;
    }
    let distance = camera.distance(light.direction * radius_m);
    light.brightness * night * smooth(config.near_out_m, config.near_in_m, distance)
}

/// The player's lights grouped into far lights: each group of at least
/// `player_threshold` lights is one, at its lights' mean direction, as wide
/// as its furthest light (at least `player_min_radius_m`) and as bright as
/// its count allows.
///
/// `lamps` are the lights' directions (unit vectors, the planet's frame).
/// A light falls in a cube of a grid of `player_link_m` over the planet's
/// frame, and occupied cubes that touch (the 26 round each) are one group.
/// The cubes are kept in a `BTreeMap` and the groups made in the order of
/// their least cube, so the answer is the same whatever order the lights came
/// in.
pub fn player_clusters(lamps: &[Vec3], radius_m: f32, config: &NightLightsConfig) -> Vec<FarLight> {
    let link = config.player_link_m;
    let mut cubes: BTreeMap<(i32, i32, i32), Vec<Vec3>> = BTreeMap::new();
    for &d in lamps {
        let d = d.normalize_or_zero();
        if d == Vec3::ZERO {
            continue;
        }
        let p = d * radius_m / link;
        let cube = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
        cubes.entry(cube).or_default().push(d);
    }
    let keys: Vec<(i32, i32, i32)> = cubes.keys().copied().collect();
    let index: BTreeMap<(i32, i32, i32), usize> =
        keys.iter().enumerate().map(|(i, &k)| (k, i)).collect();
    let mut parent: Vec<usize> = (0..keys.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for (i, &(x, y, z)) in keys.iter().enumerate() {
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(&j) = index.get(&(x + dx, y + dy, z + dz)) {
                        let (a, b) = (root(&mut parent, i), root(&mut parent, j));
                        // The smaller index is the root: the group is named by
                        // its least cube.
                        if a != b {
                            parent[a.max(b)] = a.min(b);
                        }
                    }
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<Vec3>> = BTreeMap::new();
    for (i, key) in keys.iter().enumerate() {
        let r = root(&mut parent, i);
        groups.entry(r).or_default().extend(&cubes[key]);
    }
    groups
        .into_iter()
        .filter(|(_, lights)| lights.len() as u32 >= config.player_threshold)
        .map(|(r, mut lights)| {
            // Summed in a fixed order, so the centre is the same float
            // whatever order the edits came in.
            lights.sort_by(|a, b| {
                a.x.total_cmp(&b.x)
                    .then(a.y.total_cmp(&b.y))
                    .then(a.z.total_cmp(&b.z))
            });
            let centre = lights.iter().copied().sum::<Vec3>().normalize_or(lights[0]);
            let reach = lights
                .iter()
                .map(|d| d.distance(centre) * radius_m)
                .fold(0.0_f32, f32::max);
            let (x, y, z) = keys[r];
            FarLight {
                direction: centre,
                radius_m: reach.max(config.player_min_radius_m),
                brightness: config.player_brightness
                    * (lights.len() as f32 / config.player_full_count as f32).min(1.0),
                seed: mix((x as u32) ^ mix(y as u32 ^ mix(z as u32 ^ 0x706c_6179))),
                spacing_m: config.player_spacing_m,
            }
        })
        .collect()
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Murmur3's finaliser: a seed that owes nothing to its neighbour's.
fn mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

#[cfg(test)]
mod tests;
