//! One step of the atmosphere, in the order the design gives: sun and heat,
//! pressure, wind, transport, water, lightning, forcing, then the ocean.

use super::numeric::chance;
use super::{Atmosphere, Forcing, Strike, smoothstep};
use crate::daylight::DAY_S;
use glam::Vec3;
use std::f32::consts::{FRAC_PI_2, TAU};

impl Atmosphere {
    /// Advance one fixed step with the sun at `sun` (a unit direction in the
    /// planet's frame) and whatever forcing the players are applying.
    pub fn step(&mut self, sun: Vec3, forcing: &[Forcing]) {
        let sun = sun.normalize_or(Vec3::X);
        let dt = self.settings.dt_s;
        let n = self.grid.len();
        let divergence: Vec<f32> = (0..n)
            .map(|i| self.grid.divergence(&self.wind, i, |_| false))
            .collect();
        self.heat(sun, dt);
        self.pressure(sun, &divergence, dt);
        self.accelerate(dt);
        self.aloft();
        self.carry(dt);
        // Lift reads the divergence averaged over each cell and its ring: the
        // raw value carries the grid's own cell-to-cell noise, which grows as
        // the cells shrink and would make the weather depend on the level.
        let broad: Vec<f32> = (0..n)
            .map(|i| divergence[i] + self.grid.neighbour_excess(&divergence, i, |_| false) * 0.75)
            .collect();
        // The slider brews its storm before the water step, so the updraft it
        // drives condenses and rains in the same step it is driven.
        self.forced.fill(0.0);
        for f in forcing {
            self.force(*f, dt);
        }
        self.water(&broad, dt);
        self.lightning(dt);
        self.seed_storms(dt);
        let every = (self.settings.mesoscale_every_s / dt).max(1.0) as u64;
        if self.step.is_multiple_of(every) {
            self.refresh_mesoscale();
        }
        self.ocean(dt);
        self.guard();
        self.step += 1;
    }

    /// Stages 1 and 2: sunlight heats the ground, the ground radiates, spreads
    /// its heat and warms the air.
    fn heat(&mut self, sun: Vec3, dt: f32) {
        let s = self.settings;
        let n = self.grid.len();
        let before = self.ground_k.clone();
        for i in 0..n {
            let c = self.grid.centre[i];
            let cover = self.cover(i);
            let sunlight = s.solar_wm2 * c.dot(sun).max(0.0) * (1.0 - s.cloud_albedo * cover);
            self.sunlight[i] = sunlight;
            let absorbed = sunlight * (1.0 - self.surface.albedo[i]);
            let outgoing = s.olr_a + s.olr_b * before[i] - s.cloud_greenhouse * cover;
            let sensible = s.sensible_wm2k * (before[i] - self.air_k[i]);
            let spread = self.grid.neighbour_excess(&before, i, |_| false) * s.heat_spread;
            self.ground_k[i] +=
                dt * ((absorbed - outgoing - sensible) / self.surface.heat_capacity[i] + spread);
            self.air_k[i] += dt * (self.ground_k[i] - self.air_k[i]) / s.air_relax_s;
        }
    }

    /// Stage 3: pressure falls where the air is warm and in the belts' lows,
    /// rises where the air converges, and spreads as gravity waves.
    fn pressure(&mut self, sun: Vec3, divergence: &[f32], dt: f32) {
        let s = self.settings;
        let n = self.grid.len();
        let (mut sum, mut area) = (0.0f64, 0.0f64);
        for i in 0..n {
            sum += (self.air_k[i] * self.grid.area[i]) as f64;
            area += self.grid.area[i] as f64;
        }
        let mean = (sum / area) as f32;
        let shift = sun.y.clamp(-1.0, 1.0).asin() * s.belt_follow_sun;
        let c2 = s.gravity_wave_mps * s.gravity_wave_mps;
        let before = self.phi.clone();
        for i in 0..n {
            let latitude = (self.grid.centre[i].y.clamp(-1.0, 1.0).asin() - shift)
                .clamp(-FRAC_PI_2, FRAC_PI_2);
            // Lows at the thermal equator and near 60 deg, highs near 30 deg
            // and at the poles: the three cells' surface pressure.
            let belts = -s.belt_pressure * (6.0 * latitude).cos();
            let wanted = belts - s.thermal_pressure * (self.air_k[i] - mean);
            let smooth = self.grid.neighbour_excess(&before, i, |_| false) * s.smoothing;
            self.phi[i] +=
                dt * (-c2 * divergence[i] - (before[i] - wanted) / s.pressure_relax_s) + smooth;
        }
    }

    /// Stage 4: the pressure gradient pushes, the spin turns, the ground drags.
    fn accelerate(&mut self, dt: f32) {
        let s = self.settings;
        let n = self.grid.len();
        let before = self.wind.clone();
        for i in 0..n {
            let c = self.grid.centre[i];
            let mut u = before[i] - self.grid.gradient(&self.phi, i, |_| false) * dt;
            u = turn(u, c, -coriolis(&s, c) * dt);
            let drag = if self.surface.ocean[i] {
                s.drag_sea_s
            } else {
                s.drag_land_s
            };
            u /= 1.0 + dt / drag;
            u += neighbour_mean(&self.grid, &before, i, |_| false)
                .map_or(Vec3::ZERO, |m| (m - u) * s.smoothing);
            self.wind[i] = u - c * c.dot(u);
        }
    }

    /// The wind at cloud height: the surface wind plus the thermal wind, which
    /// blows along the lines of equal temperature, fastest where they crowd.
    /// That band of fast wind is the jet.
    fn aloft(&mut self) {
        let s = self.settings;
        let n = self.grid.len();
        for i in 0..n {
            let c = self.grid.centre[i];
            // Floored away from the equator, where f passes through nought.
            let f = coriolis(&s, Vec3::Y * (c.y.signum() * c.y.abs().max(0.25)));
            let gradient = self.grid.gradient(&self.air_k, i, |_| false);
            let taper = smoothstep(0.1, 0.35, c.y.abs());
            let mut thermal = c.cross(gradient) * (s.thermal_wind / f) * taper;
            let speed = thermal.length();
            if speed > s.jet_max_mps {
                thermal *= s.jet_max_mps / speed;
            }
            self.upper[i] = self.wind[i] + thermal;
        }
    }

    /// Stage 5: everything the air carries moves with it. Vapour and cloud
    /// move in the conserving form, so moving water neither makes nor loses
    /// any; the cloud rides the steering wind, the rest the surface wind.
    fn carry(&mut self, dt: f32) {
        let s = self.settings;
        let steering: Vec<Vec3> = self
            .wind
            .iter()
            .zip(&self.upper)
            .map(|(w, u)| w.lerp(*u, s.cloud_steering))
            .collect();
        let surface = self.grid.fluxes(&self.wind, |_| false);
        let aloft = self.grid.fluxes(&steering, |_| false);
        self.air_k = self.grid.upwind(&self.air_k, &surface, dt, false);
        self.charge = self.grid.upwind(&self.charge, &surface, dt, false);
        self.vapour = self.grid.upwind(&self.vapour, &surface, dt, true);
        self.cloud = self.grid.upwind(&self.cloud, &aloft, dt, true);
        self.wind = self.grid.upwind_vec(&self.wind, &surface, dt);
    }

    /// Stage 6: the sea and wet ground evaporate, rising air condenses its
    /// vapour into cloud and is warmed by it, thick cloud rains.
    fn water(&mut self, divergence: &[f32], dt: f32) {
        let s = self.settings;
        for (i, &divergence) in divergence.iter().enumerate() {
            let wind = self.wind[i];
            let elevation = self.surface.elevation[i];
            // Signed: converging air and air driven up a slope rise and cool;
            // diverging air and air running downhill sink and dry, which is
            // what clears the sky under a high.
            let ground = self.ground_k[i] - s.lapse_k_per_m * elevation;
            let air = self.air_k[i] - s.lapse_k_per_m * elevation;
            // And sunlit land lifts the air over it: its heat capacity is small,
            // so what it absorbs goes into the air by day. This is the
            // afternoon's cumulus and its thunderstorm; the rain's own delay
            // (condensing, then raining out) puts the peak after noon.
            let buoyant = if self.surface.ocean[i] {
                0.0
            } else {
                let absorbed = self.sunlight[i] * (1.0 - self.surface.albedo[i]);
                (absorbed - s.convection_threshold_wm2).max(0.0) * s.convection_mps_per_wm2
            };
            let rising = -divergence * s.lift_depth_m
                + wind.dot(self.surface.slope[i])
                + buoyant
                + self.forced[i];
            let lift = rising.max(0.0);
            self.lift[i] = lift;
            let deficit = (self.saturation(ground) - self.vapour[i]).max(0.0);
            let evaporated = s.evaporation
                * deficit
                * (1.0 + wind.length() / s.evaporation_wind_mps)
                * self.surface.wetness[i]
                * dt;
            self.vapour[i] += evaporated;
            self.ground_k[i] -= evaporated * s.evaporation_cooling / self.surface.heat_capacity[i];
            let saturated = self.saturation(air)
                * (-s.lift_saturation * rising).clamp(-3.0, 2.0).exp()
                * (1.0 + s.mesoscale * self.mesoscale[i]);
            let condensed =
                ((self.vapour[i] - saturated).max(0.0) * dt / s.condense_s).min(self.vapour[i]);
            self.vapour[i] -= condensed;
            self.cloud[i] += condensed;
            self.air_k[i] += s.latent_k_per_kg * condensed;
            if self.vapour[i] < saturated {
                let dried = (self.cloud[i].min(saturated - self.vapour[i]) * dt
                    / s.cloud_evaporate_s)
                    .min(self.cloud[i]);
                self.cloud[i] -= dried;
                self.vapour[i] += dried;
            }
            // Cold cloud rains (snows) out of less water: ice grows at the
            // droplets' expense. So the threshold falls with what the air can
            // hold below 15 C; without it the polar cloud never reached the
            // threshold and never dried, and piled up day on day.
            let cold = (self.saturation(air) / s.saturation_kg).min(1.0);
            let threshold = s.rain_threshold_kg * cold / (1.0 + lift / s.convective_rain_mps);
            let rained = ((self.cloud[i] - threshold).max(0.0) * dt / s.rain_s).min(self.cloud[i]);
            self.cloud[i] -= rained;
            self.rain_rate[i] = rained / dt;
            self.charge[i] += s.charge_rate * (condensed / dt) * lift * dt;
            self.charge[i] /= 1.0 + dt / s.charge_decay_s;
        }
    }

    /// Stage 7: a charged storm strikes. The strike rains out much of its
    /// cloud and drops a cold pool whose outflow lifts the air round it.
    fn lightning(&mut self, dt: f32) {
        let s = self.settings;
        let keep = (s.strike_keep_s / dt).ceil() as u64;
        let now = self.step;
        self.strikes
            .retain(|strike| now.saturating_sub(strike.step) <= keep);
        for i in 0..self.grid.len() {
            if self.charge[i] < s.strike_charge || chance(self.seed(), i, now) >= s.strike_chance {
                continue;
            }
            let strength = ((self.charge[i] - s.strike_charge) / s.strike_charge.max(1e-6) + 0.5)
                .clamp(0.0, 1.0);
            self.charge[i] = 0.0;
            let rained = self.cloud[i] * s.strike_rain_share;
            self.cloud[i] -= rained;
            self.rain_rate[i] += rained / dt;
            self.phi[i] += s.pool_pressure;
            self.air_k[i] -= s.pool_k;
            self.strikes.push(Strike {
                direction: self.grid.centre[i],
                step: now,
                strength,
            });
        }
    }

    /// The weather slider: brew a storm round a place, toward a saturated,
    /// cloudy, warm column, fading out to the forcing's radius.
    fn force(&mut self, forcing: Forcing, dt: f32) {
        let s = self.settings;
        let strength = forcing.strength.clamp(0.0, 1.0);
        let Some(at) = forcing.direction.try_normalize() else {
            return;
        };
        if strength <= 0.0 {
            return;
        }
        let reach = s.forcing_radius_m / self.grid.radius;
        let rate = (dt / s.forcing_s).min(1.0);
        for i in 0..self.grid.len() {
            let angle = self.grid.centre[i].dot(at).clamp(-1.0, 1.0).acos();
            if angle > reach {
                continue;
            }
            let weight = strength * (1.0 - smoothstep(reach * 0.5, reach, angle));
            let target = s.forcing_cloud_kg * weight;
            if self.cloud[i] < target {
                self.cloud[i] += (target - self.cloud[i]) * rate;
            }
            let saturated = self.saturation(self.air_k[i]);
            if self.vapour[i] < saturated * weight {
                self.vapour[i] += (saturated * weight - self.vapour[i]) * rate;
            }
            self.charge[i] += weight * rate * 0.2;
            self.forced[i] = self.forced[i].max(weight * s.forcing_lift_mps);
        }
    }

    /// Any cell a bad value reached goes back to its climate, whole, so a NaN
    /// cannot spread across the planet from one cell.
    fn guard(&mut self) {
        for i in 0..self.grid.len() {
            let fine = self.phi[i].is_finite()
                && self.wind[i].is_finite()
                && self.air_k[i].is_finite()
                && self.ground_k[i].is_finite()
                && self.vapour[i].is_finite()
                && self.cloud[i].is_finite()
                && self.charge[i].is_finite()
                && self.eta[i].is_finite()
                && self.current[i].is_finite();
            if !fine {
                let climate = super::climate_k(self.grid.centre[i].y);
                self.phi[i] = 0.0;
                self.wind[i] = Vec3::ZERO;
                self.air_k[i] = climate;
                self.ground_k[i] = climate;
                self.vapour[i] = 0.0;
                self.cloud[i] = 0.0;
                self.charge[i] = 0.0;
                self.eta[i] = 0.0;
                self.current[i] = Vec3::ZERO;
            }
            self.vapour[i] = self.vapour[i].max(0.0);
            self.cloud[i] = self.cloud[i].max(0.0);
            self.charge[i] = self.charge[i].max(0.0);
        }
    }
}

/// The Coriolis parameter at a cell, per second: twice the spin the air feels,
/// resolved on the local vertical. The planet turns so the ground moves toward
/// increasing `atan2(z, x)` (`daylight`: the sun sets toward decreasing), which
/// is a spin vector along -Y. So `f` is negative over the +Y hemisphere: a low
/// there turns clockwise seen from above, the way the ground under it turns.
pub(super) fn coriolis(s: &super::AtmosphereSettings, c: Vec3) -> f32 {
    let omega = TAU / DAY_S * s.coriolis_scale;
    -2.0 * omega * c.y
}

/// Rotate a tangent vector about the local vertical by `angle` radians.
pub(super) fn turn(u: Vec3, up: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    u * cos + up.cross(u) * sin
}

/// The mean of a vector field over a cell's open neighbours, carried onto the
/// cell's tangent plane; `None` if every neighbour is a wall.
pub(super) fn neighbour_mean(
    grid: &super::Grid,
    field: &[Vec3],
    cell: usize,
    blocked: impl Fn(usize) -> bool,
) -> Option<Vec3> {
    let c = grid.centre[cell];
    let (mut sum, mut count) = (Vec3::ZERO, 0.0);
    for side in 0..grid.sides[cell] as usize {
        let k = grid.neighbour[cell][side] as usize;
        if !blocked(k) {
            sum += field[k];
            count += 1.0;
        }
    }
    (count > 0.0).then(|| {
        let m = sum / count;
        m - c * c.dot(m)
    })
}
