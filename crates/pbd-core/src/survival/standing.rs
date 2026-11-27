//! Townsfolk who fight back (`survival-mode` design 7, combat C3): a town's
//! standing with the player, and a guard's blow.
//!
//! A strike on a townsperson in Survival harms nobody (the townsfolk are
//! derived from the settlement record) but lowers the town's standing, a
//! `standing` record per town, and brings the guards. In Explorer a swing at
//! a townsperson does nothing: [`struck`] answers `None`.

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;
use super::mode::WorldMode;

/// The record kind and schema of a town's standing.
pub const KIND: &str = "standing";
pub const SCHEMA: u32 = 1;

/// The least and most a standing can be.
pub const RANGE: (i32, i32) = (-100, 100);

/// A town's standing with the player, keyed by its site id. An absent
/// record reads 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Standing {
    pub site: u32,
    pub value: i32,
}

/// A saved standing, or 0 for a town nobody has a record of.
pub fn of_saved(saved: Option<Standing>) -> i32 {
    saved.map_or(0, |s| s.value)
}

/// The standing after a strike on one of the town's people, or `None` in
/// Explorer, where nothing happens.
pub fn struck(cfg: &SurvivalConfig, mode: WorldMode, value: i32) -> Option<i32> {
    mode.is_survival()
        .then(|| (value - cfg.townsfolk.struck_standing).clamp(RANGE.0, RANGE.1))
}

/// Whether a fight goes on: the player inside the town's walls and their
/// margin, and up.
pub fn fight_on(cfg: &SurvivalConfig, from_centre_m: f32, walls_m: f32, player_down: bool) -> bool {
    !player_down && from_centre_m <= walls_m + cfg.townsfolk.leave_m
}

/// A guard in a fight: closing, winding up, or recovering after a blow.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Guard {
    /// Seconds into a wind-up, while within reach.
    pub windup_s: Option<f32>,
    /// Seconds until it may wind up again.
    pub rest_s: f32,
}

/// What a guard does this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GuardAct {
    /// Walks toward the player at this pace, m/s.
    Close(f32),
    /// Stands and winds up (the telegraph).
    WindUp,
    /// The blow lands for this much.
    Blow(f32),
    /// Stands, recovering.
    Rest,
}

impl Guard {
    /// One step with the player `distance_m` away.
    pub fn step(&mut self, cfg: &SurvivalConfig, distance_m: f32, dt: f32) -> GuardAct {
        let t = &cfg.townsfolk;
        let dt = dt.max(0.0);
        self.rest_s = (self.rest_s - dt).max(0.0);
        if let Some(w) = self.windup_s {
            let w = w + dt;
            if w >= t.guard_windup_s {
                self.windup_s = None;
                self.rest_s = (t.guard_every_s - t.guard_windup_s).max(0.0);
                // The blow lands only on a player still within reach.
                return if distance_m <= t.guard_reach_m * 1.2 {
                    GuardAct::Blow(t.guard_damage)
                } else {
                    GuardAct::Rest
                };
            }
            self.windup_s = Some(w);
            return GuardAct::WindUp;
        }
        if distance_m > t.guard_reach_m {
            return GuardAct::Close(t.guard_mps);
        }
        if self.rest_s > 0.0 {
            return GuardAct::Rest;
        }
        self.windup_s = Some(0.0);
        GuardAct::WindUp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strike_lowers_the_towns_standing_in_survival_only() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(of_saved(None), 0);
        assert_eq!(struck(&cfg, WorldMode::Survival, 0), Some(-10));
        assert_eq!(struck(&cfg, WorldMode::Survival, -95), Some(-100));
        assert_eq!(struck(&cfg, WorldMode::Explorer, 0), None);
        let s = Standing {
            site: 7,
            value: -20,
        };
        let back: Standing = ron::from_str(&ron::to_string(&s).unwrap()).unwrap();
        assert_eq!(of_saved(Some(back)), -20);
    }

    #[test]
    fn the_fight_ends_past_the_walls_or_when_downed() {
        let cfg = SurvivalConfig::shipped();
        assert!(fight_on(&cfg, 30.0, 25.0, false));
        assert!(!fight_on(&cfg, 38.0, 25.0, false));
        assert!(!fight_on(&cfg, 5.0, 25.0, true));
    }

    #[test]
    fn a_guard_closes_winds_up_and_strikes_on_a_beat() {
        let cfg = SurvivalConfig::shipped();
        let dt = 1.0 / 30.0;
        let mut g = Guard::default();
        assert_eq!(g.step(&cfg, 10.0, dt), GuardAct::Close(3.2));
        let mut blows = Vec::new();
        let mut first_windup = None;
        for i in 0..(4.0 / dt) as usize {
            match g.step(&cfg, 1.0, dt) {
                GuardAct::WindUp if first_windup.is_none() => first_windup = Some(i),
                GuardAct::Blow(d) => blows.push((i as f32 * dt, d)),
                _ => {}
            }
        }
        assert_eq!(first_windup, Some(0));
        // The telegraph first, then one blow every guard_every_s.
        assert!((blows[0].0 - 0.7).abs() < 0.05, "{blows:?}");
        assert!((blows[1].0 - blows[0].0 - 1.4).abs() < 0.07, "{blows:?}");
        assert!(blows.iter().all(|(_, d)| *d == 8.0));
    }

    #[test]
    fn a_player_who_steps_out_of_the_windup_is_not_hit() {
        let cfg = SurvivalConfig::shipped();
        let mut g = Guard::default();
        let dt = 1.0 / 30.0;
        g.step(&cfg, 1.0, dt);
        let mut acts = Vec::new();
        for _ in 0..30 {
            acts.push(g.step(&cfg, 5.0, dt));
        }
        assert!(!acts.iter().any(|a| matches!(a, GuardAct::Blow(_))));
    }
}
