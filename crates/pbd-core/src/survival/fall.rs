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

/// Health a landing takes: `max(0, v - v_safe) * k`, or nothing into deep
/// water.
pub fn fall_damage(cfg: &FallConfig, impact_mps: f32, gravity_mps2: f32, landing: Landing) -> f32 {
    if let Landing::Water { depth_layers } = landing
        && depth_layers >= cfg.water_depth_layers
    {
        return 0.0;
    }
    if !impact_mps.is_finite() {
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
