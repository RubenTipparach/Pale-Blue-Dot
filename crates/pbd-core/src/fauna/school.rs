//! A school: one species' fish, flocking.
//!
//! The fish are packed arrays rather than entities, because there are dozens
//! of them to a school and a fish is a point with a heading, not an actor.
//! Positions are planet-local metres (the planet's centre at the origin), so
//! "up" at a fish is its own direction and the surface and the bed are radii.
//!
//! Each step is the three boid rules within the school (separation,
//! alignment, cohesion), a goal the school wanders between, the surface and
//! the bed as walls, and a splash to scatter from. One fish at a time may be
//! steered to a lure or held at it by the line; that is how the fish that
//! bites is a fish that was swimming there.

use super::{FlockSettings, Rng, Species, hash2};
use glam::Vec3;

/// The water a school swims in: the sea surface and the ground under a
/// point, as radii from the planet's centre, m.
pub trait Water {
    fn surface(&self, point: Vec3) -> f32;
    fn bed(&self, point: Vec3) -> f32;

    /// Water depth over a point, m; nought or less on dry land.
    fn depth(&self, point: Vec3) -> f32 {
        self.surface(point) - self.bed(point)
    }
}

/// Keeps a fish this far off the surface and the bed, m.
const SKIN_M: f32 = 0.25;

#[derive(Clone, Debug, PartialEq)]
pub struct School {
    /// The species' index in its body's roster.
    pub species: u16,
    pub seed: u64,
    /// Where it spawned: the water its goals are drawn around, and where a
    /// fish that strays over water too shallow for it is pushed back toward.
    pub anchor: Vec3,
    pub positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    /// Seconds of fright left per fish.
    pub spook: Vec<f32>,
    /// A stable number per fish, which is what its length is hashed from, so
    /// a fish measures the same whichever of its schoolmates are caught first.
    pub ids: Vec<u32>,
    pub goal: Vec3,
    goal_t: f32,
    /// The school has scented a lure and is swimming for it.
    pub lured: bool,
    /// One fish steered to a point: the lure it has left the school for.
    pub lure: Option<(usize, Vec3)>,
    /// One fish held at a point: nibbling, biting or on the line.
    pub held: Option<(usize, Vec3)>,
    rng: Rng,
}

impl School {
    /// A school of `species` spawned about `anchor`, which the spawner has
    /// already found to be water the species lives in.
    pub fn spawn(
        index: u16,
        species: &Species,
        anchor: Vec3,
        seed: u64,
        water: &impl Water,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let count = if species.school.1 > species.school.0 {
            species.school.0
                + (rng.next_u64() % u64::from(species.school.1 - species.school.0 + 1)) as u16
        } else {
            species.school.0
        };
        let up = anchor.normalize_or(Vec3::Y);
        let (east, north) = tangents(up);
        let mut school = Self {
            species: index,
            seed,
            anchor,
            positions: Vec::with_capacity(count as usize),
            velocities: Vec::with_capacity(count as usize),
            spook: vec![0.0; count as usize],
            ids: (0..count as u32).collect(),
            goal: anchor,
            goal_t: 0.0,
            lured: false,
            lure: None,
            held: None,
            rng,
        };
        for _ in 0..count {
            let spread = 1.5;
            let offset = east * school.rng.range(-spread, spread)
                + north * school.rng.range(-spread, spread);
            let at = (anchor + offset).normalize_or(up);
            let point = at * water.surface(at * anchor.length());
            let radius = school.depth_for(species, point, water);
            school.positions.push(at * radius);
            let heading = east * school.rng.range(-1.0, 1.0) + north * school.rng.range(-1.0, 1.0);
            school
                .velocities
                .push(heading.normalize_or(east) * species.speed_mps.0);
        }
        school.goal = school.pick_goal(species, 25.0, water);
        school
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// The school's middle.
    pub fn centre(&self) -> Vec3 {
        if self.is_empty() {
            return self.anchor;
        }
        self.positions.iter().copied().sum::<Vec3>() / self.len() as f32
    }

    /// The nearest fish to a point that is not spooked.
    pub fn nearest_calm(&self, point: Vec3) -> Option<usize> {
        (0..self.len())
            .filter(|&i| self.spook[i] <= 0.0)
            .min_by(|&a, &b| {
                self.positions[a]
                    .distance_squared(point)
                    .total_cmp(&self.positions[b].distance_squared(point))
            })
    }

    /// Frighten one fish.
    pub fn frighten(&mut self, fish: usize, seconds: f32) {
        if let Some(s) = self.spook.get_mut(fish) {
            *s = s.max(seconds);
        }
    }

    /// Take one fish out: the one that was caught.
    pub fn remove(&mut self, fish: usize) {
        if fish >= self.len() {
            return;
        }
        self.positions.swap_remove(fish);
        self.velocities.swap_remove(fish);
        self.spook.swap_remove(fish);
        self.ids.swap_remove(fish);
        self.lure = None;
        self.held = None;
    }

    /// How long this fish is, cm: its species' length, 85 to 125% of it,
    /// hashed from the school and the fish so it never changes.
    pub fn length_cm(&self, fish: usize, species: &Species) -> u32 {
        let id = self.ids.get(fish).copied().unwrap_or(0);
        let u = (hash2(self.seed, u64::from(id)) >> 40) as f32 / (1u64 << 24) as f32;
        (species.length_m * (0.85 + 0.40 * u) * 100.0)
            .round()
            .max(1.0) as u32
    }

    /// Point the school at a lure: its goal becomes the water under it.
    pub fn scent(&mut self, species: &Species, lure: Vec3, water: &impl Water, hold_s: f32) {
        let up = lure.normalize_or(Vec3::Y);
        let point = up * water.surface(lure);
        self.goal = up * self.depth_for(species, point, water);
        self.goal_t = hold_s;
        self.lured = true;
    }

    /// One step of the flock.
    pub fn step(
        &mut self,
        dt: f32,
        species: &Species,
        flock: &FlockSettings,
        water: &impl Water,
        splash: Option<Vec3>,
    ) {
        if dt.is_nan() || dt <= 0.0 || self.is_empty() {
            return;
        }
        self.goal_t -= dt;
        if self.goal_t <= 0.0 {
            self.goal = self.pick_goal(species, flock.goal_reach_m, water);
            self.goal_t = self.rng.range(flock.goal_every_s.0, flock.goal_every_s.1);
            self.lured = false;
        }
        let n = self.len();
        let accelerations: Vec<Vec3> = (0..n)
            .map(|i| self.acceleration(i, species, flock, water, splash))
            .collect();
        for (i, acceleration) in accelerations.into_iter().enumerate() {
            if let Some((held, point)) = self.held
                && held == i
            {
                self.velocities[i] = (point - self.positions[i]) / dt;
                self.positions[i] = point;
                continue;
            }
            if splash.is_some_and(|s| s.distance(self.positions[i]) < flock.splash_m) {
                self.spook[i] = self.spook[i].max(1.0);
            }
            self.spook[i] = (self.spook[i] - dt).max(0.0);
            let mut v = self.velocities[i] + acceleration * dt;
            let fright = if self.spook[i] > 0.0 { 2.0 } else { 1.0 };
            let (slow, fast) = (species.speed_mps.0 * 0.5, species.speed_mps.1 * fright);
            let speed = v.length();
            if speed > fast {
                v *= fast / speed;
            } else if speed < slow {
                v = v.normalize_or(tangents(self.positions[i].normalize_or(Vec3::Y)).0) * slow;
            }
            if !v.is_finite() {
                v = Vec3::ZERO;
            }
            let mut p = self.positions[i] + v * dt;
            // Never out of the water: between the bed and the surface, with a
            // skin off each, and in the middle of a column too thin for both.
            let (top, bottom) = (water.surface(p) - SKIN_M, water.bed(p) + SKIN_M);
            let r = p.length();
            let clamped = if top > bottom {
                r.clamp(bottom, top)
            } else {
                0.5 * (top + bottom)
            };
            if r > 0.0 && clamped.is_finite() {
                p *= clamped / r;
            }
            if p.is_finite() {
                self.positions[i] = p;
                self.velocities[i] = v;
            }
        }
    }

    fn acceleration(
        &self,
        i: usize,
        species: &Species,
        flock: &FlockSettings,
        water: &impl Water,
        splash: Option<Vec3>,
    ) -> Vec3 {
        let p = self.positions[i];
        let up = p.normalize_or(Vec3::Y);
        let (mut separation, mut heading, mut middle, mut neighbours) =
            (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, 0.0f32);
        for (j, q) in self.positions.iter().enumerate() {
            if j == i {
                continue;
            }
            let d = p.distance(*q);
            if d < flock.separation_m && d > 1e-4 {
                separation += (p - *q) / (d * d);
            }
            if d < flock.neighbour_m {
                heading += self.velocities[j];
                middle += *q;
                neighbours += 1.0;
            }
        }
        let mut acc = separation * flock.separation;
        if neighbours > 0.0 {
            acc += (heading / neighbours - self.velocities[i]) * flock.alignment;
            acc += (middle / neighbours - p) * flock.cohesion;
        }
        let lure = self
            .lure
            .filter(|(fish, _)| *fish == i)
            .map(|(_, point)| point);
        let (target, weight) = match lure {
            Some(point) => (point, flock.lure),
            None if self.lured => (self.goal, flock.lured),
            None => (self.goal, flock.goal),
        };
        let want = target - p;
        acc += want.normalize_or_zero() * weight * want.length().min(1.0);
        // The surface and the bed are walls.
        let r = p.length();
        let (top, bottom) = (water.surface(p) - SKIN_M, water.bed(p) + SKIN_M);
        if r > top - 0.3 {
            acc -= up * flock.wall * (r - (top - 0.3) + 0.1);
        }
        if r < bottom + 0.3 {
            acc += up * flock.wall * ((bottom + 0.3) - r + 0.1);
        }
        // Water too shallow for the species: back toward where it spawned.
        if water.depth(p) < species.depth_m.0 + 0.4 {
            let home = self.anchor - p;
            acc += (home - up * home.dot(up)).normalize_or_zero() * flock.wall;
        }
        if species.bed && lure.is_none() {
            acc += up * 3.0 * ((water.bed(p) + 0.35) - r);
        }
        if let Some(centre) = splash {
            let d = p.distance(centre);
            if d < flock.splash_m {
                acc += (p - centre).normalize_or_zero()
                    * flock.splash_push
                    * (1.0 - d / flock.splash_m);
            }
        }
        if acc.is_finite() { acc } else { Vec3::ZERO }
    }

    /// The radius a fish of this species keeps at a point: in its depth band
    /// under the surface, or just off the bed for a bed dweller.
    fn depth_for(&mut self, species: &Species, point: Vec3, water: &impl Water) -> f32 {
        let (surface, bed) = (water.surface(point), water.bed(point));
        if species.bed {
            return bed + SKIN_M + 0.1;
        }
        let deepest = (surface - bed - SKIN_M - 0.05).max(SKIN_M);
        let lo = species.depth_m.0.min(deepest);
        let hi = species.depth_m.1.min(deepest).max(lo);
        surface - self.rng.range(lo, hi)
    }

    /// A new place to swim to near where the school spawned, in water deep
    /// enough for it; the old goal when none is found.
    fn pick_goal(&mut self, species: &Species, reach_m: f32, water: &impl Water) -> Vec3 {
        let up = self.anchor.normalize_or(Vec3::Y);
        let (east, north) = tangents(up);
        for _ in 0..12 {
            let offset = east * self.rng.range(-reach_m, reach_m)
                + north * self.rng.range(-reach_m, reach_m);
            let at = (self.anchor + offset).normalize_or(up) * self.anchor.length();
            if species.fits(water.depth(at)) {
                let point = at.normalize_or(up) * water.surface(at);
                return at.normalize_or(up) * self.depth_for(species, point, water);
            }
        }
        if self.goal == Vec3::ZERO {
            self.anchor
        } else {
            self.goal
        }
    }
}

/// Two tangents at a direction, stable away from the poles' axis.
fn tangents(up: Vec3) -> (Vec3, Vec3) {
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    (east, up.cross(east))
}
