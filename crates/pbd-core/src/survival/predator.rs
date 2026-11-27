//! The hunt (`survival-mode` design 6, combat C2): a predator's states,
//! notice, stalk, warn, attack and flee, as a pure step of what it senses.
//!
//! In an Explorer world it never leaves [`Phase::Wander`]: the step returns
//! at once and nothing bites. The game moves the animal by
//! [`Hunt::motion`] and lands a bite's damage through the server.

use super::config::{HuntConfig, Predator, SurvivalConfig};
use super::mode::WorldMode;

/// Where a predator is in its hunt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// The herd's own pottering; it has not noticed the player.
    #[default]
    Wander,
    Stalk,
    Warn,
    Attack,
    Flee,
}

/// What a predator senses this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Senses {
    /// To the player, m.
    pub distance_m: f32,
    pub night: bool,
    /// Passed out or dead: never attacked.
    pub player_down: bool,
    /// Its health left, a share of full.
    pub health_share: f32,
    /// The player struck it this step.
    pub struck: bool,
}

/// How the game should move it this step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    /// Its own wander.
    Free,
    /// Toward the player at this pace, m/s, the body this low (0 up, 1 flat).
    Toward { mps: f32, crouch: f32 },
    /// Held where it is, crouched this low.
    Hold { crouch: f32 },
    /// Away from the player at its run.
    Away,
}

/// One predator's hunt.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hunt {
    pub phase: Phase,
    /// Seconds in this phase.
    pub in_phase_s: f32,
    /// It has warned once already in this hunt, so a re-warn is short.
    pub warned: bool,
    /// Seconds it will not notice anything (after fleeing).
    pub calm_s: f32,
    /// Seconds the player has been out of reach.
    pub away_s: f32,
}

impl Hunt {
    fn go(&mut self, phase: Phase) {
        if self.phase != phase {
            self.phase = phase;
            self.in_phase_s = 0.0;
        }
    }

    /// One step of `dt` seconds. Returns a bite's damage, at the end of a
    /// lunge that reached the player.
    pub fn step(
        &mut self,
        cfg: &SurvivalConfig,
        p: &Predator,
        mode: WorldMode,
        s: &Senses,
        dt: f32,
    ) -> Option<f32> {
        if !mode.is_survival() {
            *self = Hunt::default();
            return None;
        }
        let h: &HuntConfig = &cfg.hunt;
        let dt = dt.max(0.0);
        self.in_phase_s += dt;
        self.calm_s = (self.calm_s - dt).max(0.0);
        let notice = p.notice_m * if s.night { h.night_mult } else { 1.0 };
        // Fleeing comes first: hurt, or the player downed.
        let hunting = !matches!(self.phase, Phase::Wander | Phase::Flee);
        if s.health_share < h.flee_below && (hunting || s.struck) {
            self.go(Phase::Flee);
            return None;
        }
        if s.player_down && hunting {
            self.go(Phase::Flee);
            return None;
        }
        // Struck while it can fight: it fights back at once.
        if s.struck && self.phase != Phase::Flee && !s.player_down {
            self.warned = true;
            self.away_s = 0.0;
            self.go(Phase::Attack);
            self.in_phase_s = 0.0;
            return None;
        }
        if hunting {
            if s.distance_m > notice * 1.5 {
                self.away_s += dt;
            } else {
                self.away_s = 0.0;
            }
            if self.away_s >= h.give_up_s {
                self.away_s = 0.0;
                self.go(Phase::Flee);
                return None;
            }
        }
        match self.phase {
            Phase::Wander => {
                if self.calm_s <= 0.0 && !s.player_down && s.distance_m <= notice {
                    self.warned = false;
                    self.go(Phase::Stalk);
                }
                None
            }
            Phase::Stalk => {
                if s.distance_m <= p.warn_m {
                    self.go(Phase::Warn);
                }
                None
            }
            Phase::Warn => {
                let hold = if self.warned { h.rewarn_s } else { p.warn_s };
                if s.distance_m > notice {
                    self.go(Phase::Stalk);
                } else if self.in_phase_s >= hold && s.distance_m <= p.lunge_m {
                    self.warned = true;
                    self.go(Phase::Attack);
                } else if self.in_phase_s >= hold && s.distance_m > p.warn_m {
                    self.go(Phase::Stalk);
                }
                None
            }
            Phase::Attack => {
                if self.in_phase_s >= h.lunge_s {
                    let bit = s.distance_m <= p.bite_m;
                    // Back off and warn again, briefly.
                    self.go(Phase::Warn);
                    return bit.then_some(p.damage);
                }
                None
            }
            Phase::Flee => {
                if self.in_phase_s >= h.flee_s {
                    self.calm_s = h.calm_s;
                    self.warned = false;
                    self.go(Phase::Wander);
                }
                None
            }
        }
    }

    /// How it moves in its phase.
    pub fn motion(&self, p: &Predator) -> Motion {
        match self.phase {
            Phase::Wander => Motion::Free,
            Phase::Stalk => Motion::Toward {
                mps: p.stalk_mps,
                crouch: 0.15,
            },
            Phase::Warn => Motion::Hold { crouch: 0.3 },
            Phase::Attack => Motion::Toward {
                mps: p.lunge_mps,
                crouch: 0.05,
            },
            Phase::Flee => Motion::Away,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 30.0;

    fn fox() -> (SurvivalConfig, Predator) {
        let cfg = SurvivalConfig::shipped();
        let p = cfg.predators["red-fox"].clone();
        (cfg, p)
    }

    fn senses(distance_m: f32) -> Senses {
        Senses {
            distance_m,
            night: false,
            player_down: false,
            health_share: 1.0,
            struck: false,
        }
    }

    /// Step `seconds` at a fixed distance; the bites landed.
    fn hold(h: &mut Hunt, s: Senses, seconds: f32, mode: WorldMode) -> Vec<f32> {
        let (cfg, p) = fox();
        let mut bites = Vec::new();
        for _ in 0..(seconds / DT).round() as usize {
            bites.extend(h.step(&cfg, &p, mode, &s, DT));
        }
        bites
    }

    #[test]
    fn it_notices_within_range_and_further_at_night() {
        let mut h = Hunt::default();
        hold(&mut h, senses(16.0), 1.0, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Wander);
        hold(&mut h, senses(13.0), 0.1, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Stalk);
        let mut h = Hunt::default();
        hold(
            &mut h,
            Senses {
                night: true,
                ..senses(20.0)
            },
            0.1,
            WorldMode::Survival,
        );
        assert_eq!(h.phase, Phase::Stalk);
    }

    #[test]
    fn a_stalk_turns_to_a_warning_close_in() {
        let mut h = Hunt::default();
        hold(&mut h, senses(10.0), 0.2, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Stalk);
        hold(&mut h, senses(4.5), 0.1, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Warn);
    }

    #[test]
    fn the_warning_is_held_at_least_one_and_a_half_seconds_before_the_first_attack() {
        let mut h = Hunt::default();
        hold(&mut h, senses(10.0), 0.1, WorldMode::Survival);
        hold(&mut h, senses(1.0), DT, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Warn);
        // Right beside the player the whole time: no attack before 1.5 s.
        let mut t = 0.0;
        while h.phase == Phase::Warn {
            hold(&mut h, senses(1.0), DT, WorldMode::Survival);
            t += DT;
        }
        assert_eq!(h.phase, Phase::Attack);
        assert!(t >= 1.5, "attacked after {t} s");
    }

    #[test]
    fn a_lunge_that_reaches_bites_for_the_species_damage() {
        let mut h = Hunt {
            phase: Phase::Attack,
            warned: true,
            ..Hunt::default()
        };
        let bites = hold(&mut h, senses(1.0), 0.7, WorldMode::Survival);
        assert_eq!(bites, vec![6.0]);
        assert_eq!(h.phase, Phase::Warn);
        // Out of the bite at the lunge's end: a miss.
        let mut h = Hunt {
            phase: Phase::Attack,
            warned: true,
            ..Hunt::default()
        };
        assert!(hold(&mut h, senses(3.0), 0.7, WorldMode::Survival).is_empty());
    }

    #[test]
    fn a_rewarn_is_short() {
        let mut h = Hunt {
            phase: Phase::Warn,
            warned: true,
            ..Hunt::default()
        };
        hold(&mut h, senses(1.0), 0.9, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Attack);
    }

    #[test]
    fn struck_it_fights_back_at_once() {
        let mut h = Hunt::default();
        hold(&mut h, senses(10.0), 0.1, WorldMode::Survival);
        let (cfg, p) = fox();
        h.step(
            &cfg,
            &p,
            WorldMode::Survival,
            &Senses {
                struck: true,
                health_share: 0.6,
                ..senses(2.0)
            },
            DT,
        );
        assert_eq!(h.phase, Phase::Attack);
    }

    #[test]
    fn below_a_third_of_its_health_it_flees_then_calms() {
        let mut h = Hunt {
            phase: Phase::Warn,
            ..Hunt::default()
        };
        hold(
            &mut h,
            Senses {
                health_share: 0.3,
                ..senses(2.0)
            },
            DT,
            WorldMode::Survival,
        );
        assert_eq!(h.phase, Phase::Flee);
        assert_eq!(h.motion(&fox().1), Motion::Away);
        hold(
            &mut h,
            Senses {
                health_share: 0.3,
                ..senses(30.0)
            },
            10.1,
            WorldMode::Survival,
        );
        assert_eq!(h.phase, Phase::Wander);
        // Calm: it does not notice for a while, even close.
        hold(&mut h, senses(3.0), 1.0, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Wander);
    }

    #[test]
    fn out_of_reach_a_while_it_gives_up() {
        let mut h = Hunt {
            phase: Phase::Stalk,
            ..Hunt::default()
        };
        hold(&mut h, senses(25.0), 7.5, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Stalk);
        hold(&mut h, senses(25.0), 0.6, WorldMode::Survival);
        assert_eq!(h.phase, Phase::Flee);
    }

    #[test]
    fn a_downed_player_is_left_alone() {
        let mut h = Hunt {
            phase: Phase::Attack,
            warned: true,
            ..Hunt::default()
        };
        let bites = hold(
            &mut h,
            Senses {
                player_down: true,
                ..senses(0.5)
            },
            2.0,
            WorldMode::Survival,
        );
        assert!(bites.is_empty());
        assert_eq!(h.phase, Phase::Flee);
    }

    #[test]
    fn an_explorer_fox_never_attacks() {
        let mut h = Hunt::default();
        let (cfg, p) = fox();
        let mut bites = 0;
        for i in 0..3000 {
            let s = Senses {
                struck: i % 50 == 0,
                night: i % 2 == 0,
                ..senses(0.5)
            };
            bites += usize::from(h.step(&cfg, &p, WorldMode::Explorer, &s, DT).is_some());
            assert_eq!(h.phase, Phase::Wander);
            assert_eq!(h.motion(&p), Motion::Free);
        }
        assert_eq!(bites, 0);
    }
}
