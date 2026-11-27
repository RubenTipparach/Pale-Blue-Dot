//! A swing's rule (`survival-mode` combat C1): when its hit lands, what it
//! may land on, and the raise of a thrown spear or the draw of a bow.
//!
//! The rule owns the clock. A press starts a swing; its hit lands
//! `weapons.hit_share` of the weapon's `time_s` after the press; holding the
//! button starts the next one a `time_s` after the last, on one steady beat.
//! The game draws the swing from this clock and never feeds it back.

use super::config::SurvivalConfig;
use super::weapons::Held;

/// What a landed swing met.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// A creature of the yard.
    Creature,
    /// A townsperson.
    Townsperson,
    /// Another player: never struck (design 10, "No player can damage
    /// another").
    Player,
}

/// Whether a swing that met `target` does anything: never on a player.
pub fn lands(target: Target) -> bool {
    !matches!(target, Target::Player)
}

/// A swing's period, s: the weapon's `time_s`, a tool's or bare hands'
/// `tool_swing_s`.
pub fn period_s(cfg: &SurvivalConfig, held: &Held) -> f32 {
    match held {
        Held::Weapon(code) => cfg
            .weapons
            .melee
            .get(code)
            .map_or(cfg.weapons.tool_swing_s, |m| m.time_s),
        _ => cfg.weapons.tool_swing_s,
    }
}

/// When its hit lands after the press, s.
pub fn hit_s(cfg: &SurvivalConfig, held: &Held) -> f32 {
    period_s(cfg, held) * cfg.weapons.hit_share
}

/// One swing under way: seconds since its press, and its period and hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Under {
    pub t: f32,
    pub period_s: f32,
    pub hit_s: f32,
    /// Swings since the press, the first 0.
    pub count: u32,
}

/// The swing clock.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwingClock {
    under: Option<(Under, Held)>,
}

impl SwingClock {
    /// One step of `dt` seconds with the button `pressed` and `held` in
    /// hand. Returns the hits that landed in it, with what was in hand at
    /// the press (a hit whose swing began with a spear lands as a spear).
    pub fn step(&mut self, cfg: &SurvivalConfig, held: &Held, pressed: bool, dt: f32) -> Vec<Held> {
        let dt = dt.max(0.0);
        let mut hits = Vec::new();
        if self.under.is_none() {
            if !pressed {
                return hits;
            }
            self.under = Some((
                Under {
                    t: 0.0,
                    period_s: period_s(cfg, held),
                    hit_s: hit_s(cfg, held),
                    count: 0,
                },
                held.clone(),
            ));
            // The press's own frame: the clock starts at zero.
            return hits;
        }
        let mut left = dt;
        while let Some((mut u, h)) = self.under.take() {
            let to_end = u.period_s - u.t;
            let step = left.min(to_end);
            if u.t < u.hit_s && u.t + step >= u.hit_s {
                hits.push(h.clone());
            }
            u.t += step;
            left -= step;
            if u.t + 1e-6 < u.period_s {
                self.under = Some((u, h));
                break;
            }
            if !pressed {
                break;
            }
            // Held: the next swing on the beat, with what is in hand now.
            self.under = Some((
                Under {
                    t: 0.0,
                    period_s: period_s(cfg, held),
                    hit_s: hit_s(cfg, held),
                    count: u.count + 1,
                },
                held.clone(),
            ));
            if left <= 0.0 {
                break;
            }
        }
        hits
    }

    /// The swing under way, if any: what the drawing reads.
    pub fn under(&self) -> Option<Under> {
        self.under.as_ref().map(|(u, _)| *u)
    }

    /// What was in hand when the swing under way began.
    pub fn held(&self) -> Option<&Held> {
        self.under.as_ref().map(|(_, h)| h)
    }

    /// Put the swing down at once (the slot changed, the player is downed).
    pub fn stop(&mut self) {
        self.under = None;
    }
}

/// A spear raised to throw or a bow drawn: how long the button has been
/// held, s.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Draw {
    pub held_s: Option<f32>,
}

/// What letting go did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Release {
    /// Held too briefly: nothing leaves the hand.
    Nothing,
    /// It goes, after this long raised or drawn, s.
    Go { drawn_s: f32 },
}

impl Draw {
    /// One step: the button `pressed` or not. On the frame it is let go,
    /// what the release does, given the least raise `min_s`.
    pub fn step(&mut self, pressed: bool, dt: f32, min_s: f32) -> Option<Release> {
        match (self.held_s, pressed) {
            (None, true) => {
                self.held_s = Some(0.0);
                None
            }
            (Some(s), true) => {
                self.held_s = Some(s + dt.max(0.0));
                None
            }
            (Some(s), false) => {
                self.held_s = None;
                Some(if s >= min_s {
                    Release::Go { drawn_s: s }
                } else {
                    Release::Nothing
                })
            }
            (None, false) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Tier, Tool};

    const DT: f32 = 1.0 / 60.0;

    fn run(clock: &mut SwingClock, held: &Held, pressed: bool, seconds: f32) -> Vec<f32> {
        let cfg = SurvivalConfig::shipped();
        let mut at = Vec::new();
        let mut t = 0.0;
        while t < seconds - 1e-6 {
            t += DT;
            if !clock.step(&cfg, held, pressed, DT).is_empty() {
                at.push(t);
            }
        }
        at
    }

    #[test]
    fn the_hit_lands_at_the_rules_moment_after_the_press() {
        let cfg = SurvivalConfig::shipped();
        let sword = Held::Weapon("bronze-sword".into());
        assert!((hit_s(&cfg, &sword) - 0.5 * 0.45).abs() < 1e-6);
        let mut clock = SwingClock::default();
        // The press's frame starts the clock; then one tap's swing.
        assert!(clock.step(&cfg, &sword, true, DT).is_empty());
        let hits = run(&mut clock, &sword, false, 1.0);
        assert_eq!(hits.len(), 1);
        assert!((hits[0] - 0.225).abs() <= DT, "{hits:?}");
        assert!(clock.under().is_none());
    }

    #[test]
    fn holding_swings_on_one_steady_beat() {
        let cfg = SurvivalConfig::shipped();
        let spear = Held::Weapon("stone-spear".into());
        let mut clock = SwingClock::default();
        clock.step(&cfg, &spear, true, DT);
        let hits = run(&mut clock, &spear, true, 2.2);
        assert_eq!(hits.len(), 3, "{hits:?}");
        for w in hits.windows(2) {
            assert!((w[1] - w[0] - 0.7).abs() <= DT * 1.01, "{hits:?}");
        }
        assert_eq!(clock.under().unwrap().count, 3);
    }

    #[test]
    fn a_tool_swings_on_the_tool_period() {
        let cfg = SurvivalConfig::shipped();
        let axe = Held::Tool(Tool::Axe, Tier::Stone);
        assert_eq!(period_s(&cfg, &axe), cfg.weapons.tool_swing_s);
        assert_eq!(period_s(&cfg, &Held::Hand), cfg.weapons.tool_swing_s);
    }

    #[test]
    fn a_hit_carries_what_was_in_hand_at_the_press() {
        let cfg = SurvivalConfig::shipped();
        let mut clock = SwingClock::default();
        let axe = Held::Weapon("iron-hunting-axe".into());
        clock.step(&cfg, &axe, true, DT);
        let mut got = Vec::new();
        for _ in 0..60 {
            got.extend(clock.step(&cfg, &Held::Hand, false, DT));
        }
        assert_eq!(got, vec![axe]);
    }

    #[test]
    fn a_swing_never_lands_on_a_player() {
        assert!(!lands(Target::Player));
        assert!(lands(Target::Creature) && lands(Target::Townsperson));
    }

    #[test]
    fn a_spear_raised_too_briefly_is_not_thrown() {
        let mut d = Draw::default();
        assert_eq!(d.step(true, DT, 0.25), None);
        for _ in 0..5 {
            d.step(true, DT, 0.25);
        }
        assert_eq!(d.step(false, DT, 0.25), Some(Release::Nothing));
        let mut d = Draw::default();
        for _ in 0..40 {
            d.step(true, DT, 0.25);
        }
        match d.step(false, DT, 0.25) {
            Some(Release::Go { drawn_s }) => assert!((drawn_s - 39.0 * DT).abs() < 1e-4),
            other => panic!("{other:?}"),
        }
    }
}
