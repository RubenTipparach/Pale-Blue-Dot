//! Heat stroke and passing out (`survival-mode` design 14): at `stroke_c`
//! the body takes `stroke_pts` at once and the player drops where they
//! stand for `out_s` of world time, the world going on; a hit or another
//! player's water wakes them at once, and they wake dazed. Cold hurts but
//! never knocks a player down.

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;

/// Out, dazed, or neither, as saved in the player's record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub enum Consciousness {
    #[default]
    Awake,
    /// Passed out, `left_s` of play to go.
    Out { left_s: f32 },
    /// Woken, `left_s` of being dazed to go.
    Dazed { left_s: f32 },
}

/// Why the player woke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// The time ran out.
    Time,
    /// A hit (its damage still lands).
    Hit,
    /// Another player held use on them with water.
    Revived,
}

/// What the walker may do now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Control {
    /// Controls are off: the body lies where it fell.
    pub controls: bool,
    pub can_sprint: bool,
    pub walk_mult: f32,
    /// In deep water the body floats face up, head out.
    pub floats_face_up: bool,
}

impl Consciousness {
    pub fn is_out(self) -> bool {
        matches!(self, Consciousness::Out { .. })
    }

    /// Passes out (the stroke's blow is the caller's to land).
    pub fn pass_out(cfg: &SurvivalConfig) -> Self {
        Consciousness::Out {
            left_s: cfg.pass_out.out_s,
        }
    }

    /// `dt` seconds of play. Returns the wake, if the time ran out.
    pub fn step(&mut self, cfg: &SurvivalConfig, dt: f32) -> Option<Wake> {
        match *self {
            Consciousness::Awake => None,
            Consciousness::Out { left_s } => {
                if left_s - dt <= 0.0 {
                    *self = Consciousness::Dazed {
                        left_s: cfg.pass_out.dazed_s,
                    };
                    Some(Wake::Time)
                } else {
                    *self = Consciousness::Out {
                        left_s: left_s - dt,
                    };
                    None
                }
            }
            Consciousness::Dazed { left_s } => {
                *self = if left_s - dt <= 0.0 {
                    Consciousness::Awake
                } else {
                    Consciousness::Dazed {
                        left_s: left_s - dt,
                    }
                };
                None
            }
        }
    }

    /// Wakes a passed-out player at once (a hit, a revive): dazed after.
    pub fn wake(&mut self, cfg: &SurvivalConfig) -> bool {
        if self.is_out() {
            *self = Consciousness::Dazed {
                left_s: cfg.pass_out.dazed_s,
            };
            true
        } else {
            false
        }
    }

    pub fn control(self, cfg: &SurvivalConfig, in_deep_water: bool) -> Control {
        match self {
            Consciousness::Awake => Control {
                controls: true,
                can_sprint: true,
                walk_mult: 1.0,
                floats_face_up: false,
            },
            Consciousness::Out { .. } => Control {
                controls: false,
                can_sprint: false,
                walk_mult: 0.0,
                floats_face_up: in_deep_water,
            },
            Consciousness::Dazed { .. } => Control {
                controls: true,
                can_sprint: false,
                walk_mult: cfg.pass_out.dazed_walk_mult,
                floats_face_up: false,
            },
        }
    }

    /// How dark the view is, 0..1: fading over `fade_s` as the player goes
    /// out, dark while out.
    pub fn darkness(self, cfg: &SurvivalConfig) -> f32 {
        match self {
            Consciousness::Out { left_s } => {
                let gone = cfg.pass_out.out_s - left_s;
                if cfg.pass_out.fade_s > 0.0 {
                    (gone / cfg.pass_out.fade_s).clamp(0.0, 1.0)
                } else {
                    1.0
                }
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_for_thirty_seconds_then_dazed_for_sixty() {
        let cfg = SurvivalConfig::shipped();
        let mut c = Consciousness::pass_out(&cfg);
        assert!(!c.control(&cfg, false).controls);
        assert_eq!(c.step(&cfg, 29.0), None);
        assert_eq!(c.step(&cfg, 1.5), Some(Wake::Time));
        let dazed = c.control(&cfg, false);
        assert!(dazed.controls && !dazed.can_sprint && dazed.walk_mult == 0.75);
        c.step(&cfg, 60.0);
        assert_eq!(c, Consciousness::Awake);
    }

    #[test]
    fn the_view_fades_over_a_second_and_a_half() {
        let cfg = SurvivalConfig::shipped();
        let mut c = Consciousness::pass_out(&cfg);
        assert_eq!(c.darkness(&cfg), 0.0);
        c.step(&cfg, 0.75);
        assert!((c.darkness(&cfg) - 0.5).abs() < 1e-4);
        c.step(&cfg, 1.0);
        assert_eq!(c.darkness(&cfg), 1.0);
    }
}
