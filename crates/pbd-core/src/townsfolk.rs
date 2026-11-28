//! The people of a settlement (`townsfolk` decisions 1, 19, 20 and 23):
//! derived from the town's record and its template, never saved.
//!
//! A template carries its people as the towns mockup's `person()` calls put
//! them ([`PersonDef`]); [`people_of`] lays each onto the town's chart as
//! the boats and the dressing are laid ([`crate::settlement::sea::point`]),
//! gives it a look from the generator ([`look::generate`]) seeded by the
//! site and its index, and says where it stands at a time ([`Person::at`]).
//! The figure, its face and its motion are [`figure`], [`face`] and
//! [`motion`]; the game draws them.

pub mod character;
pub mod face;
pub mod figure;
pub mod look;
pub mod motion;

#[cfg(test)]
mod tests;

use crate::settlement::Template;
use crate::settlement::chart::{Chart, Patch};
use crate::settlement::record::{Town, datum};
use crate::settlement::sea;
use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A person's solid: a circle this round, m, from the feet up
/// [`SOLID_HEIGHT_M`] (decision 3).
pub const SOLID_RADIUS_M: f32 = 0.28;
pub const SOLID_HEIGHT_M: f32 = 1.9;
/// How finely a path is sampled onto the planet's cells, m.
pub const PATH_STEP_M: f32 = 0.5;

/// A person as a template holds them: the mockup's `person()` call.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PersonDef {
    /// The mockup's role, lower case with underscores: an id.
    pub role: String,
    /// Where they stand, `[x, y, z]` in the mockup's metres, `y` their feet.
    pub at: [f32; 3],
    /// The way they face, radians from the mockup's east toward its south.
    pub facing: f32,
    /// Their tunic, sRGB.
    pub tunic: u32,
    /// What they do at their post: `hammer`, `wipe`, `sit` or `idle`.
    #[serde(default)]
    pub work: Option<String>,
    /// The two ends they walk between, `[x, z]`, and their pace, m/s.
    #[serde(default)]
    pub path: Option<Vec<[f32; 2]>>,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub spear: bool,
}

/// What a person does at their post.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Work {
    Hammer,
    Wipe,
    /// Sitting, drawn standing at the post as the mockup draws it.
    Sit,
    Idle,
}

impl Work {
    fn of(id: Option<&str>) -> Self {
        match id {
            Some("hammer") => Work::Hammer,
            Some("wipe") => Work::Wipe,
            Some("sit") => Work::Sit,
            _ => Work::Idle,
        }
    }
}

/// Who a person is, for every part of the game that asks: the site's id and
/// their place in its template's people.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonId {
    pub site: u32,
    pub index: u32,
}

/// A post or a path, on the planet: directions, the town's frame.
#[derive(Clone, Debug, PartialEq)]
pub enum Activity {
    /// Standing at `at`, facing `facing` (a tangent there), at work.
    Post { work: Work },
    /// Walking back and forth along `points` at `speed` m/s; `length_m` its
    /// length, `start_s` where in its cycle it is at time zero.
    Path {
        points: Vec<Vec3>,
        speed: f32,
        length_m: f32,
        start_s: f32,
    },
}

/// A townsperson, derived.
#[derive(Clone, Debug, PartialEq)]
pub struct Person {
    pub id: PersonId,
    pub role: String,
    pub spear: bool,
    /// Their post's (or path's first point's) direction from the planet's
    /// centre, and their feet's height over the radius, m.
    pub at: Vec3,
    pub altitude_m: f32,
    /// The way they face at their post, a unit tangent at `at`.
    pub facing: Vec3,
    pub activity: Activity,
    pub look: look::Look,
    /// Their own seed, for their face's and body's sequences.
    pub seed: u32,
}

/// Where a person is at a time: their feet in the planet's frame (f64),
/// the way they face (a tangent there) and whether they are walking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub feet: DVec3,
    pub forward: DVec3,
    pub walking: bool,
}

impl Person {
    pub fn work(&self) -> Option<Work> {
        match self.activity {
            Activity::Post { work } => Some(work),
            Activity::Path { .. } => None,
        }
    }

    pub fn pace(&self) -> Option<f32> {
        match self.activity {
            Activity::Path { speed, .. } => Some(speed),
            Activity::Post { .. } => None,
        }
    }

    /// Where they are at `t` seconds on a planet of `radius_m`: at their
    /// post, or back and forth along their path as the mockup's
    /// `animateNpc` walks it.
    pub fn at(&self, t: f64, radius_m: f64) -> Placed {
        let r = radius_m + f64::from(self.altitude_m);
        match &self.activity {
            Activity::Post { .. } => Placed {
                feet: self.at.as_dvec3().normalize() * r,
                forward: self.facing.as_dvec3(),
                walking: false,
            },
            Activity::Path {
                points,
                speed,
                length_m,
                start_s,
            } => {
                let period = 2.0 * f64::from(*length_m) / f64::from(speed.max(0.05));
                let ph = ((t + f64::from(*start_s)) / period).rem_euclid(1.0);
                let (f, back) = if ph < 0.5 {
                    (ph * 2.0, false)
                } else {
                    (2.0 - ph * 2.0, true)
                };
                let along = f * (points.len() - 1) as f64;
                let i = (along.floor() as usize).min(points.len() - 2);
                let u = along - i as f64;
                let (a, b) = (points[i].as_dvec3(), points[i + 1].as_dvec3());
                let d = a.lerp(b, u).normalize();
                let ahead = if back { a - b } else { b - a };
                let tangent = ahead - d * ahead.dot(d);
                Placed {
                    feet: d * r,
                    forward: tangent.try_normalize().unwrap_or(self.facing.as_dvec3()),
                    walking: true,
                }
            }
        }
    }
}

/// What a person may not walk through, in the mockup's plane: its
/// buildings' cells and its masonry (decision 20).
pub struct Walls {
    cell_m: f32,
    building: BTreeMap<(i32, i32), usize>,
    masonry: BTreeMap<(i32, i32), (i32, i32)>,
}

impl Walls {
    pub fn of(template: &Template) -> Self {
        Self {
            cell_m: template.grid.cell_m,
            building: template
                .buildings
                .iter()
                .enumerate()
                .flat_map(|(i, b)| b.cells.iter().map(move |c| ((c[0], c[1]), i)))
                .collect(),
            masonry: template
                .masonry
                .iter()
                .map(|m| ((m.c, m.r), (m.from, m.to)))
                .collect(),
        }
    }

    /// The building whose cell holds `(x, z)`, if any.
    pub fn building_at(&self, x: f32, z: f32) -> Option<usize> {
        self.building.get(&sea::cell_at(x, z, self.cell_m)).copied()
    }

    /// Whether masonry stands across a body with its feet at `feet` at
    /// `(x, z)` (a body on the wall's walk stands over it).
    pub fn walled(&self, x: f32, z: f32, feet: f32) -> bool {
        self.masonry
            .get(&sea::cell_at(x, z, self.cell_m))
            .is_some_and(|&(from, to)| {
                (from as f32) < feet + SOLID_HEIGHT_M && (to as f32) > feet + 0.05
            })
    }

    /// A path's two ends, each brought back to half a metre short of the
    /// first wall met walking from the person's post `at` toward it: the
    /// mockup's people walk through what was built after their lanes (the
    /// village's lane walker through its hall), and the game's do not.
    pub fn clip(&self, at: [f32; 3], ends: [[f32; 2]; 2]) -> [[f32; 2]; 2] {
        let [x0, feet, z0] = at;
        let home = self.building_at(x0, z0);
        ends.map(|[ex, ez]| {
            let len = ((ex - x0).powi(2) + (ez - z0).powi(2)).sqrt();
            let n = (len / 0.1).ceil().max(1.0) as usize;
            let at_u = |u: f32| [x0 + (ex - x0) * u, z0 + (ez - z0) * u];
            for k in 1..=n {
                let [x, z] = at_u(k as f32 / n as f32);
                if self.building_at(x, z) != home || self.walled(x, z, feet) {
                    let back = ((k as f32 * 0.1 - 0.5) / len.max(1e-3)).max(0.0);
                    return at_u(back.min(1.0));
                }
            }
            [ex, ez]
        })
    }
}

/// A seed from a site and a person's index.
pub fn seed_of(id: PersonId) -> u32 {
    let mut z =
        (u64::from(id.site) << 32 | u64::from(id.index)).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((z ^ (z >> 31)) & 0xffff_ffff) as u32
}

/// A town's people, in its template's order (decision 20): each laid on the
/// town's chart, given its look, its post or its path. A person whose post
/// is off the chart is left out (a harbour's boat that was not laid).
pub fn people_of(town: &Town, template: &Template, chart: &Chart, patch: &Patch) -> Vec<Person> {
    let cell_m = template.grid.cell_m;
    let datum = datum(template) as f32;
    let people = look::people_of_scene(&template.scene);
    let point = |x: f32, z: f32| sea::point(chart, patch, x, z, cell_m);
    let walls = Walls::of(template);
    template
        .people
        .iter()
        .enumerate()
        .filter_map(|(i, def)| {
            let id = PersonId {
                site: town.site,
                index: i as u32,
            };
            let seed = seed_of(id);
            let [x, y, z] = def.at;
            let at = point(x, z)?;
            let ahead = point(x + def.facing.cos(), z + def.facing.sin())?;
            let f = ahead - at;
            let facing = (f - at * f.dot(at)).normalize_or_zero();
            let activity = match &def.path {
                Some(ends) if ends.len() >= 2 => {
                    let [[ax, az], [bx, bz]] = walls.clip([x, y, z], [ends[0], ends[1]]);
                    let len = ((bx - ax).powi(2) + (bz - az).powi(2)).sqrt();
                    let steps = (len / PATH_STEP_M).ceil().max(1.0) as usize;
                    let points: Option<Vec<Vec3>> = (0..=steps)
                        .map(|k| {
                            let u = k as f32 / steps as f32;
                            point(ax + (bx - ax) * u, az + (bz - az) * u)
                        })
                        .collect();
                    let speed = def.speed.unwrap_or(1.2);
                    match points {
                        Some(points) => Activity::Path {
                            points,
                            speed,
                            length_m: len,
                            start_s: (seed % 10_000) as f32 / 1000.0,
                        },
                        None => Activity::Post { work: Work::Idle },
                    }
                }
                _ => Activity::Post {
                    work: Work::of(def.work.as_deref()),
                },
            };
            Some(Person {
                id,
                role: def.role.clone(),
                spear: def.spear,
                at,
                altitude_m: town.terrace as f32 + (y - datum),
                facing,
                activity,
                look: look::generate(people, seed, i as u32, def.tunic),
                seed,
            })
        })
        .collect()
}

/// Whether a body at `centre` (a planet point, f64), `half_height` above and
/// below and `radius` round, overlaps a person whose feet are at `feet`; and
/// if so the way out along the ground, a unit tangent (decision 25).
pub fn overlaps(centre: DVec3, half_height: f32, radius: f32, feet: DVec3) -> Option<DVec3> {
    let up = feet.try_normalize()?;
    let d = centre - feet;
    let rise = d.dot(up);
    let (lo, hi) = (rise - f64::from(half_height), rise + f64::from(half_height));
    if hi <= 0.0 || lo >= f64::from(SOLID_HEIGHT_M) {
        return None;
    }
    let flat = d - up * rise;
    let reach = f64::from(radius + SOLID_RADIUS_M);
    let dist = flat.length();
    if dist >= reach {
        return None;
    }
    Some(if dist > 1e-6 {
        flat / dist
    } else {
        // Dead centre: out along any tangent.
        let side = if up.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
        (side - up * side.dot(up)).normalize()
    })
}
