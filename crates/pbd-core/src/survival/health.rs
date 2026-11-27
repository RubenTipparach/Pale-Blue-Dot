//! Health (`survival-mode` design 2): one bar, refilling slowly while there
//! is food, drained by every harm the other rules name.

use super::config::SurvivalConfig;

/// What took health, for the HUD's cause and the server's log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cause {
    Fall,
    /// The head under water with no breath left.
    Drowning,
    /// The suit's air run out where the air is too thin.
    Suffocation,
    /// Food empty.
    Starving,
    /// Water empty.
    Thirst,
    /// Heat exhaustion.
    Heat,
    /// The heat stroke's one blow.
    HeatStroke,
    /// Hypothermia.
    Cold,
    /// A creature's attack.
    Creature,
    /// A townsperson's blow.
    Townsfolk,
}

/// Health refilled a second of play with `food_fp` in the body and `water`
/// in it: whole at or above `hungry_below_fp`, at `regen_hungry_mult` of
/// that below, nothing with food or water empty.
pub fn regen_pts_per_s(cfg: &SurvivalConfig, food_fp: f32, water: f32) -> f32 {
    if food_fp <= 0.0 || water <= 0.0 {
        return 0.0;
    }
    let whole = 1.0 / cfg.health.regen_s_per_pt;
    if food_fp < cfg.health.hungry_below_fp {
        whole * cfg.health.regen_hungry_mult
    } else {
        whole
    }
}

/// Health a second of play takes from an empty food bar and an empty water
/// bar, each with its cause.
pub fn empty_bar_drain(cfg: &SurvivalConfig, food_fp: f32, water: f32) -> [(Cause, f32); 2] {
    [
        (
            Cause::Starving,
            if food_fp <= 0.0 {
                1.0 / cfg.health.starving_drain_s_per_pt
            } else {
                0.0
            },
        ),
        (
            Cause::Thirst,
            if water <= 0.0 {
                1.0 / cfg.water.empty_drain_s_per_pt
            } else {
                0.0
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_refills_a_point_every_four_seconds_and_half_that_hungry() {
        let cfg = SurvivalConfig::shipped();
        assert!((regen_pts_per_s(&cfg, 80.0, 80.0) - 0.25).abs() < 1e-6);
        assert!((regen_pts_per_s(&cfg, 20.0, 80.0) - 0.125).abs() < 1e-6);
        assert_eq!(regen_pts_per_s(&cfg, 0.0, 80.0), 0.0);
        assert_eq!(regen_pts_per_s(&cfg, 80.0, 0.0), 0.0);
    }

    #[test]
    fn an_empty_bar_drains_a_point_every_ten_and_six_seconds() {
        let cfg = SurvivalConfig::shipped();
        let [starve, thirst] = empty_bar_drain(&cfg, 0.0, 0.0);
        assert!((starve.1 - 0.1).abs() < 1e-6 && (thirst.1 - 1.0 / 6.0).abs() < 1e-6);
        let [starve, thirst] = empty_bar_drain(&cfg, 1.0, 1.0);
        assert_eq!((starve.1, thirst.1), (0.0, 0.0));
    }
}
