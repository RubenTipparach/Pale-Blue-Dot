//! Falls (`survival-mode` design 3): damage from the downward speed at
//! impact, along the body's own up, in its own gravity. Water deep enough to
//! swim in cancels it.

use glam::Vec3;

use super::config::FallConfig;

/// What the body landed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    /// Ground, or water shallower than `water_depth_layers`.
    Ground,
    /// Water this many layers deep at the landing.
    Water { depth_layers: u32 },
}

/// The downward speed at impact along `up`, m/s, from the walker's local
/// velocity and the up the origin gives for its position (never a local
/// position's own direction). Rising or level is 0.
pub fn impact_speed(velocity: Vec3, up: Vec3) -> f32 {
    (-velocity.dot(up)).max(0.0)
}

/// The speed below which a landing never hurts on a body whose surface
/// gravity is `gravity_mps2`: `sqrt(2 g h_safe)`.
pub fn safe_speed(cfg: &FallConfig, gravity_mps2: f32) -> f32 {
    (2.0 * gravity_mps2.max(0.0) * cfg.safe_height_m).sqrt()
}

/// The impact speed a landing is judged at: the fastest downward speed seen
/// while airborne, but never more than a fall of `dropped_m` (the highest
/// point reached minus the landing) could reach under `gravity_mps2`, with
/// 10% and 0.5 m/s over it for a frame's sampling. A physics push that
/// spikes one frame's velocity is not a fall.
pub fn bounded_impact(peak_mps: f32, dropped_m: f32, gravity_mps2: f32) -> f32 {
    let free = (2.0 * gravity_mps2.max(0.0) * dropped_m.max(0.0)).sqrt();
    peak_mps.min(1.1 * free + 0.5)
}

/// Health a landing takes: `max(0, v - v_safe) * k`, or nothing into deep
/// water.
pub fn fall_damage(cfg: &FallConfig, impact_mps: f32, gravity_mps2: f32, landing: Landing) -> f32 {
    if let Landing::Water { depth_layers } = landing
        && depth_layers >= cfg.water_depth_layers
    {
        return 0.0;
    }
    if !impact_mps.is_finite() || gravity_mps2 < 0.1 {
        return 0.0;
    }
    (impact_mps - safe_speed(cfg, gravity_mps2)).max(0.0) * cfg.pts_per_mps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gravity::SURFACE_GRAVITY_MPS2_PER_G;
    use crate::survival::config::SurvivalConfig;

    const G: f32 = SURFACE_GRAVITY_MPS2_PER_G as f32;

    #[test]
    fn a_spike_is_capped_by_the_height_fallen() {
        let cfg = SurvivalConfig::shipped();
        // A 1 m drop under a push that reads 40 m/s for one frame.
        let v = bounded_impact(40.0, 1.0, G);
        assert!(v < 8.5, "{v}");
        assert_eq!(fall_damage(&cfg.fall, v, G, Landing::Ground), 0.0);
        assert_eq!(bounded_impact(40.0, 0.0, G), 0.5);
    }

    #[test]
    fn a_real_fall_is_not_cut() {
        for h in [2.0_f32, 5.0, 12.0, 30.0] {
            let v = landing_speed(h, G);
            assert_eq!(bounded_impact(v, h, G), v, "{h} m");
        }
    }

    #[test]
    fn no_gravity_no_fall_damage() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(fall_damage(&cfg.fall, 30.0, 0.0, Landing::Ground), 0.0);
    }

    /// The speed after falling `layers` metres from rest under `g`.
    fn landing_speed(layers: f32, g: f32) -> f32 {
        (2.0 * g * layers).sqrt()
    }

    #[test]
    fn a_two_layer_step_never_hurts() {
        let cfg = SurvivalConfig::shipped().fall;
        assert_eq!(
            fall_damage(&cfg, landing_speed(2.0, G), G, Landing::Ground),
            0.0
        );
        // Nor a standing jump's 12 m/s landing.
        assert_eq!(fall_damage(&cfg, 12.0, G, Landing::Ground), 0.0);
    }

    #[test]
    fn a_twenty_layer_fall_takes_about_80() {
        let cfg = SurvivalConfig::shipped().fall;
        let twenty = fall_damage(&cfg, landing_speed(20.0, G), G, Landing::Ground);
        let ten = fall_damage(&cfg, landing_speed(10.0, G), G, Landing::Ground);
        assert!((twenty - 80.0).abs() < 1.0, "{twenty}");
        assert!((ten - 40.0).abs() < 1.0, "{ten}");
        assert!(ten < twenty);
    }

    #[test]
    fn the_same_fall_hurts_less_on_a_half_gravity_body() {
        let cfg = SurvivalConfig::shipped().fall;
        let home = fall_damage(&cfg, landing_speed(20.0, G), G, Landing::Ground);
        let half = fall_damage(&cfg, landing_speed(20.0, G / 2.0), G / 2.0, Landing::Ground);
        assert!(half > 0.0 && half < home, "{half} against {home}");
    }

    #[test]
    fn a_fall_into_deep_water_does_not_hurt() {
        let cfg = SurvivalConfig::shipped().fall;
        let v = landing_speed(20.0, G);
        assert_eq!(
            fall_damage(&cfg, v, G, Landing::Water { depth_layers: 2 }),
            0.0
        );
        assert!(fall_damage(&cfg, v, G, Landing::Water { depth_layers: 1 }) > 70.0);
    }

    #[test]
    fn impact_speed_is_along_the_bodys_up() {
        let up = Vec3::new(0.0, 0.6, 0.8);
        assert!((impact_speed(-up * 20.0 + Vec3::X * 5.0, up) - 20.0).abs() < 1e-4);
        assert_eq!(impact_speed(up * 3.0, up), 0.0);
    }
}
