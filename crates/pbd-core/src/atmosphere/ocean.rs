//! The ocean: the same fluid, on the sea cells, walled by the coasts.
//!
//! The wind drags the surface water; the sea-surface height pushes back; the
//! same spin the air feels turns it. Where the Coriolis force grows toward the
//! poles, a basin's return flow crowds its western shore, which is the Gulf
//! Stream in the owner's reference. The current carries the sea's heat.

use super::Atmosphere;
use super::step::{coriolis, neighbour_mean, turn};
use glam::Vec3;

impl Atmosphere {
    pub(super) fn ocean(&mut self, dt: f32) {
        let s = self.settings;
        let n = self.grid.len();
        let ocean = &self.surface.ocean;
        let land = |k: usize| !ocean[k];
        let c2 = s.ocean_wave_mps * s.ocean_wave_mps;
        let before = self.eta.clone();
        for i in 0..n {
            if !ocean[i] {
                continue;
            }
            let divergence = self.grid.divergence(&self.current, i, land);
            let smooth = self.grid.neighbour_excess(&before, i, land) * s.smoothing;
            self.eta[i] += dt * (-c2 * divergence - before[i] / s.ocean_relax_s) + smooth;
        }
        let flow = self.current.clone();
        for i in 0..n {
            if !ocean[i] {
                continue;
            }
            let c = self.grid.centre[i];
            let driven = (self.wind[i] * s.current_per_wind - flow[i]) / s.ocean_drag_s;
            let mut u = flow[i] + (driven - self.grid.gradient(&self.eta, i, land)) * dt;
            u = turn(u, c, -coriolis(&s, c) * dt);
            u += neighbour_mean(&self.grid, &flow, i, land)
                .map_or(Vec3::ZERO, |m| (m - u) * s.smoothing);
            // No current runs into a coast: take off the part aimed at land.
            for side in 0..self.grid.sides[i] as usize {
                let k = self.grid.neighbour[i][side] as usize;
                if land(k) {
                    let into = u.dot(self.grid.normal[i][side]);
                    if into > 0.0 {
                        u -= self.grid.normal[i][side] * into;
                    }
                }
            }
            self.current[i] = u - c * c.dot(u);
        }
        self.carry_sea(dt);
    }

    /// The sea's heat and the current itself move with the current, only
    /// between sea cells: a coast is not a source of seawater.
    fn carry_sea(&mut self, dt: f32) {
        let ocean = &self.surface.ocean;
        let land = |k: usize| !ocean[k];
        let fluxes = self.grid.fluxes(&self.current, land);
        let heat = self.grid.upwind(&self.ground_k, &fluxes, dt, false);
        let flow = self.grid.upwind_vec(&self.current, &fluxes, dt);
        for i in 0..self.grid.len() {
            if ocean[i] {
                self.ground_k[i] = heat[i];
                self.current[i] = flow[i];
            } else {
                self.current[i] = Vec3::ZERO;
                self.eta[i] = 0.0;
            }
        }
    }
}
