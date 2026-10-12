//! Where storms are born, and the weather finer than a cell.
//!
//! A cyclone is seeded as a low with the swirl the spin gives it, where the
//! sea is warm enough away from the equator (a tropical storm) or where the
//! temperature changes steeply (a frontal storm in the storm tracks). After
//! that it is the physics' own: converging air rises and rains, the latent
//! heat deepens the low, the steering wind carries it, and land and cold water
//! starve it. The seeding is a hash of the cell and the step, so it is as
//! deterministic as the rest of the step.

use super::numeric::chance;
use super::step::coriolis;
use super::{Atmosphere, smoothstep};
use glam::Vec3;

/// Salt so a storm's hash is not the lightning's.
const STORM_SALT: u64 = 0x5701_2a5e_ed00_0003;

impl Atmosphere {
    pub(super) fn seed_storms(&mut self, dt: f32) {
        let s = self.settings;
        let n = self.grid.len();
        for i in 0..n {
            // The draw first: favour is at most one, so only a draw under the
            // bound can make a storm, and only those pay for the favour.
            let bound = s.storm_rate * self.grid.area[i] * 1e-6 * dt;
            let draw = chance(self.seed() ^ STORM_SALT, i, self.step);
            if draw < bound && draw < bound * self.storm_favour(i) {
                self.spawn_storm(i);
            }
        }
    }

    /// 0..1: how ready a cell is to make a storm.
    fn storm_favour(&self, i: usize) -> f32 {
        let s = self.settings;
        let c = self.grid.centre[i];
        // No spin at the equator, so nothing to turn a low into a vortex.
        let off_equator = smoothstep(0.08, 0.2, c.y.abs());
        let tropical = if self.surface.ocean[i] {
            smoothstep(s.storm_sea_c, s.storm_sea_c + 3.0, self.ground_k[i])
                * smoothstep(0.55, 0.3, c.y.abs())
        } else {
            0.0
        };
        let gradient = self.grid.gradient(&self.air_k, i, |_| false).length() * 1000.0;
        let frontal = smoothstep(
            s.storm_front_k_per_km * 0.5,
            s.storm_front_k_per_km,
            gradient,
        ) * smoothstep(0.35, 0.55, c.y.abs());
        off_equator * tropical.max(frontal)
    }

    /// A low at cell `at`, its air moistened and set turning.
    fn spawn_storm(&mut self, at: usize) {
        let s = self.settings;
        let centre = self.grid.centre[at];
        let reach = s.storm_radius_m / self.grid.radius;
        let cos_reach = reach.cos();
        for k in 0..self.grid.len() {
            let c = self.grid.centre[k];
            let along = c.dot(centre);
            if along < cos_reach {
                continue;
            }
            let r = along.clamp(-1.0, 1.0).acos() / reach;
            let bump = (1.0 - r * r).max(0.0).powi(2);
            self.phi[k] -= s.storm_depth * bump;
            let saturated = self.saturation(self.air_k[k]);
            self.vapour[k] = self.vapour[k].max(saturated * (0.6 + 0.35 * bump));
            // Gradient of the low's own pressure, pointing out from its eye,
            // and the balanced wind that goes round it: u = (1/f) up x grad.
            let outward = (c - centre * c.dot(centre)).normalize_or_zero();
            let outward = (outward - c * c.dot(outward)).normalize_or_zero();
            let slope =
                s.storm_depth * 4.0 * r * (1.0 - r * r).max(0.0) / (reach * self.grid.radius);
            let f = coriolis(&s, c);
            if f.abs() < 1e-6 {
                continue;
            }
            let mut swirl = c.cross(outward * slope) / f;
            let speed = swirl.length();
            if speed > 20.0 {
                swirl *= 20.0 / speed;
            }
            self.wind[k] += swirl;
        }
    }

    /// Refresh the finer-than-a-cell variability from the old field's drifting
    /// noise, in -1..1.
    pub(super) fn refresh_mesoscale(&mut self) {
        let field = crate::weather::WeatherField::DEFAULT;
        let seconds = self.step as f32 * self.settings.dt_s;
        for i in 0..self.grid.len() {
            let c: Vec3 = self.grid.centre[i];
            self.mesoscale[i] =
                (crate::weather::solar(&field, self.seed(), c * 4.0, seconds) - 0.5) * 2.0;
        }
    }
}
