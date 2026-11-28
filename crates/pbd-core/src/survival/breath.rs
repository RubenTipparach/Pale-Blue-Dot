//! Breath under water and suit air where the air is too thin
//! (`survival-mode` design 5).

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;

/// Where the head is, for breathing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Air {
    /// Breathable air.
    Open,
    /// The head's cell is water (and the body is not floating head out).
    UnderWater,
    /// On foot where the atmosphere is below a breathable density: vacuum,
    /// an airless body, very high altitude.
    Thin,
    /// Inside a sealed craft cabin: the craft's air.
    Cabin,
}

/// Whether the air on foot `height_m` above the body's mean radius is too
/// thin to breathe (design "Thin air on foot").
pub fn thin_at(cfg: &SurvivalConfig, height_m: f32) -> bool {
    height_m > cfg.suit_air.thin_above_m
}

/// The two reserves, as saved in the player's record.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Breath {
    /// Seconds of breath left under water.
    pub breath_s: f32,
    /// Seconds of suit air left.
    pub suit_s: f32,
}

impl Breath {
    pub fn full(cfg: &SurvivalConfig) -> Self {
        Self {
            breath_s: cfg.breath.under_s,
            suit_s: cfg.suit_air.reserve_s,
        }
    }

    /// `dt` seconds of play with the head in `air`. Returns the health
    /// drained, with no breath (or suit air) left.
    pub fn step(&mut self, cfg: &SurvivalConfig, air: Air, dt: f32) -> f32 {
        let b = &cfg.breath;
        let s = &cfg.suit_air;
        let breath_refill = b.under_s / b.refill_s;
        let suit_refill = s.reserve_s / s.refill_s;
        let mut short = 0.0;
        match air {
            Air::UnderWater => {
                short = (dt - self.breath_s).max(0.0);
                self.breath_s = (self.breath_s - dt).max(0.0);
            }
            Air::Thin => {
                short = (dt - self.suit_s).max(0.0);
                self.suit_s = (self.suit_s - dt).max(0.0);
            }
            Air::Open | Air::Cabin => {}
        }
        if air != Air::UnderWater {
            self.breath_s = (self.breath_s + breath_refill * dt).min(b.under_s);
        }
        if matches!(air, Air::Open | Air::Cabin) {
            self.suit_s = (self.suit_s + suit_refill * dt).min(s.reserve_s);
        }
        short * b.drown_pts_per_s
    }

    /// Whether the HUD's bubbles show: only while a reserve is not full.
    pub fn shown(&self, cfg: &SurvivalConfig) -> bool {
        self.breath_s < cfg.breath.under_s || self.suit_s < cfg.suit_air.reserve_s
    }
}

/// The head's air for a body in water: a winded swimmer or one passed out
/// in deep water floats with the head out (design 12 and 14), so neither
/// drowns on its own.
pub fn head_air(head_in_water: bool, floats_head_out: bool) -> Air {
    if head_in_water && !floats_head_out {
        Air::UnderWater
    } else {
        Air::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thin_above_the_ceiling_only() {
        let cfg = SurvivalConfig::shipped();
        assert!(!thin_at(&cfg, 150.0), "a summit breathes");
        assert!(!thin_at(&cfg, 600.0));
        assert!(thin_at(&cfg, 601.0));
    }

    #[test]
    fn a_long_dive_drains_health_and_surfacing_refills_breath() {
        let cfg = SurvivalConfig::shipped();
        let mut b = Breath::full(&cfg);
        let mut hurt = 0.0;
        for _ in 0..300 {
            hurt += b.step(&cfg, Air::UnderWater, 0.1);
        }
        assert!(hurt < 1e-3 && b.breath_s.abs() < 1e-3);
        for _ in 0..100 {
            hurt += b.step(&cfg, Air::UnderWater, 0.1);
        }
        assert!((hurt - 50.0).abs() < 0.1, "{hurt}");
        for _ in 0..30 {
            b.step(&cfg, Air::Open, 0.1);
        }
        assert!((b.breath_s - 30.0).abs() < 1e-3);
        assert!(!b.shown(&cfg));
    }

    #[test]
    fn the_suit_empties_in_thin_air_and_refills_in_the_cabin() {
        let cfg = SurvivalConfig::shipped();
        let mut b = Breath::full(&cfg);
        b.step(&cfg, Air::Thin, 20.0);
        assert!((b.suit_s - 40.0).abs() < 1e-3 && b.shown(&cfg));
        assert!(b.step(&cfg, Air::Thin, 45.0) > 0.0);
        b.step(&cfg, Air::Cabin, 5.0);
        assert!((b.suit_s - 60.0).abs() < 1e-3);
    }

    #[test]
    fn a_floating_body_breathes() {
        assert_eq!(head_air(true, true), Air::Open);
        assert_eq!(head_air(true, false), Air::UnderWater);
    }
}
