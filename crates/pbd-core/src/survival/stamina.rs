//! Stamina and what it costs in food (`survival-mode` design 12).
//!
//! The same model as `tools/survival/stamina_model.py` and the mockup's
//! Stamina tab: an effort spends points (sprinting and swimming burn food
//! while they do, and owe the points as strain); after a short pause with
//! nothing spending, the bar refills, each point costing food at the bar's
//! height (less in the bottom half), 1.5 times as much while strain is owed;
//! food below half slows the refill, and starving it refills only to half,
//! for free. Empty is winded until a quarter is back.
//!
//! Tool work spends nothing: digging, chopping, swinging, fishing and
//! crafting are [`Effort::None`]. Explorer has no bar at all: its walker
//! never holds a [`Stamina`].

use serde::{Deserialize, Serialize};

use super::config::{EffortRate, StaminaConfig};

/// What the body is doing this step that spends stamina continuously.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effort {
    /// Walking, standing, digging, chopping, swinging, fishing, crafting.
    #[default]
    None,
    Sprint,
    Swim,
    /// Swimming with sprint held.
    SwimSprint,
    /// In deep water, not moving.
    Tread,
    /// A bow held fully drawn.
    BowDrawn,
}

impl Effort {
    fn rate(self, cfg: &StaminaConfig) -> Option<EffortRate> {
        let s = &cfg.spend;
        match self {
            Effort::None => None,
            Effort::Sprint => Some(s.sprint),
            Effort::Swim => Some(s.swim),
            Effort::SwimSprint => Some(s.swim_sprint),
            Effort::Tread => Some(s.tread),
            Effort::BowDrawn => Some(s.bow_drawn),
        }
    }

    /// Whether it is the hard work that heats the body and drinks water
    /// faster (design 13 and 15): a sprint or a hard swim.
    pub fn is_hard(self) -> bool {
        matches!(self, Effort::Sprint | Effort::SwimSprint)
    }
}

/// A one-off spend: no strain, no food while doing it (the owner's
/// "slightly").
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Burst {
    Jump,
    /// Metres risen: a layer stepped up, a ledge climbed out of water, a
    /// ladder.
    Climb(f32),
    SpearThrow,
}

/// The bar, as saved in the player's record.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Stamina {
    /// Points, 0..max.
    pub pts: f32,
    /// Points spent by a strain effort not yet refilled.
    pub strain: f32,
    /// Seconds of play since anything last spent it.
    pub since_spend_s: f32,
    /// Empty, and not yet back to `winded_until_pts`.
    pub winded: bool,
}

/// What a step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StaminaStep {
    /// Food the step took, fp: the effort's burn and the refill's cost.
    pub food_fp: f32,
    /// The bar just ran out.
    pub became_winded: bool,
    /// The bar is back to a quarter after being winded.
    pub recovered: bool,
}

/// What the bar allows the walker (design 12, "Empty is winded").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub can_sprint: bool,
    pub swim_speed_mult: f32,
    pub jump_mult: f32,
    pub can_climb_out: bool,
    /// A winded swimmer floats with the head out and does not sink.
    pub floats: bool,
}

impl Limits {
    /// No limit.
    pub const FREE: Limits = Limits {
        can_sprint: true,
        swim_speed_mult: 1.0,
        jump_mult: 1.0,
        can_climb_out: true,
        floats: false,
    };
}

impl Stamina {
    /// A full bar.
    pub fn full(cfg: &StaminaConfig) -> Self {
        Self {
            pts: cfg.max_pts,
            strain: 0.0,
            since_spend_s: f32::MAX,
            winded: false,
        }
    }

    /// Food a point refilled costs at bar height `pts`, before strain.
    pub fn cost_fp_per_pt(cfg: &StaminaConfig, pts: f32) -> f32 {
        let half = cfg.max_pts / 2.0;
        if pts <= half {
            cfg.cost_low_fp_per_pt
        } else {
            cfg.cost_low_fp_per_pt
                + (cfg.cost_high_fp_per_pt - cfg.cost_low_fp_per_pt) * (pts - half) / half
        }
    }

    /// The refill's rate share for the food left: whole at or above
    /// `full_rate_above_fp`, down in a straight line to `rate_at_empty_food`
    /// at 0 fp.
    pub fn food_rate_share(cfg: &StaminaConfig, food_fp: f32) -> f32 {
        if food_fp >= cfg.full_rate_above_fp {
            1.0
        } else {
            let k = (food_fp.max(0.0) / cfg.full_rate_above_fp).clamp(0.0, 1.0);
            cfg.rate_at_empty_food + (1.0 - cfg.rate_at_empty_food) * k
        }
    }

    /// One-off spend. Never strain; it resets the refill's pause.
    pub fn burst(&mut self, cfg: &StaminaConfig, burst: Burst) -> StaminaStep {
        let pts = match burst {
            Burst::Jump => cfg.spend.jump_pts,
            Burst::Climb(m) => cfg.spend.climb_pts_per_m * m.max(0.0),
            Burst::SpearThrow => cfg.spend.spear_throw_pts,
        };
        self.pts = (self.pts - pts).max(0.0);
        self.since_spend_s = 0.0;
        let mut out = StaminaStep::default();
        if self.pts <= 0.0 && !self.winded {
            self.winded = true;
            out.became_winded = true;
        }
        out
    }

    /// One step of `dt` seconds of play with `effort` going on and
    /// `food_fp` in the body. `cold` halves the refill (design 14). Returns
    /// the food taken; the caller takes it from the food bar.
    pub fn step(
        &mut self,
        cfg: &StaminaConfig,
        effort: Effort,
        food_fp: f32,
        cold: bool,
        dt: f32,
    ) -> StaminaStep {
        let mut out = StaminaStep::default();
        // A winded body cannot sprint: a sprint held is a walk, a hard swim
        // a plain one.
        let effort = match (self.winded, effort) {
            (true, Effort::Sprint) => Effort::None,
            (true, Effort::SwimSprint) => Effort::Swim,
            (_, e) => e,
        };
        if let Some(rate) = effort.rate(cfg) {
            let spent = (rate.pts_per_s * dt).min(self.pts);
            self.pts -= spent;
            if rate.strain {
                self.strain = (self.strain + rate.pts_per_s * dt).min(cfg.max_pts - self.pts);
            }
            out.food_fp += rate.fp_per_s * dt;
            self.since_spend_s = 0.0;
            if self.pts <= 0.0 && !self.winded {
                self.winded = true;
                out.became_winded = true;
            }
            return out;
        }
        self.since_spend_s = (self.since_spend_s + dt).min(f32::MAX / 2.0);
        if self.since_spend_s < cfg.regen_delay_s {
            return out;
        }
        let starving = food_fp <= 0.0;
        let cap = if starving {
            cfg.starving_cap_pts
        } else {
            cfg.max_pts
        };
        if self.pts < cap {
            let mut rate = cfg.regen_pts_per_s * Self::food_rate_share(cfg, food_fp);
            if cold {
                rate *= cfg.cold_regen_mult;
            }
            let r = (rate * dt).min(cap - self.pts);
            if !starving {
                let k = if self.strain > 0.0 {
                    cfg.strain_mult
                } else {
                    1.0
                };
                out.food_fp += Self::cost_fp_per_pt(cfg, self.pts + r / 2.0) * r * k;
            }
            self.pts += r;
            self.strain = (self.strain - r).max(0.0);
        }
        if self.winded && self.pts >= cfg.winded_until_pts {
            self.winded = false;
            out.recovered = true;
        }
        out
    }

    /// What the bar allows now.
    pub fn limits(&self, cfg: &StaminaConfig) -> Limits {
        if self.winded {
            Limits {
                can_sprint: false,
                swim_speed_mult: cfg.winded.swim_speed_mult,
                jump_mult: cfg.winded.jump_mult,
                can_climb_out: false,
                floats: true,
            }
        } else {
            Limits::FREE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survival::config::SurvivalConfig;

    /// The model's body: stamina and the food it is fed from, food burned
    /// standing about at the file's idle rate, stepped at 10 ms as the
    /// Python model and the mockup are.
    struct Body {
        st: Stamina,
        food: f64,
        spent: f64,
        effort_fp: f64,
        cfg: SurvivalConfig,
    }

    impl Body {
        fn new(food: f64) -> Self {
            let cfg = SurvivalConfig::shipped();
            Self {
                st: Stamina::full(&cfg.stamina),
                food,
                spent: 0.0,
                effort_fp: 0.0,
                cfg,
            }
        }
        fn run(&mut self, secs: f32, effort: Effort) {
            let dt = 0.01;
            for _ in 0..(secs / dt).round() as usize {
                let idle = f64::from(self.cfg.food.idle_fp_per_s() * dt);
                let step = self
                    .st
                    .step(&self.cfg.stamina, effort, self.food as f32, false, dt);
                let fp = f64::from(step.food_fp);
                self.spent += idle + fp;
                self.effort_fp += fp;
                self.food -= idle + fp;
            }
        }
        fn burst(&mut self, b: Burst) {
            self.st.burst(&self.cfg.stamina, b);
        }
    }

    fn close(got: impl Into<f64>, want: f64) {
        let got = got.into();
        assert!((got - want).abs() <= 0.05, "got {got}, want {want}");
    }

    #[test]
    // 6.28 fp is design 12's worked example, not two pi.
    #[allow(clippy::approx_constant)]
    fn a_minute_of_sprinting_costs_6_28_fp() {
        let mut b = Body::new(80.0);
        b.run(60.0, Effort::Sprint);
        assert!((b.st.pts - 40.0).abs() < 0.01 && !b.st.winded);
        b.run(10.0, Effort::None);
        close(b.st.pts, 100.0);
        close(b.spent, 6.28);
        close(b.effort_fp, 4.72);
    }

    #[test]
    fn the_same_time_walking_costs_1_56_fp() {
        let mut b = Body::new(80.0);
        b.run(70.0, Effort::None);
        close(b.spent, 1.56);
        assert_eq!(b.effort_fp, 0.0);
    }

    #[test]
    fn a_200_m_swim_costs_4_77_fp() {
        let mut b = Body::new(80.0);
        b.run(50.0, Effort::Swim);
        close(b.st.pts, 60.0);
        b.run(6.0, Effort::None);
        close(b.st.pts, 100.0);
        close(b.spent, 4.77);
        close(b.effort_fp, 3.53);
    }

    #[test]
    fn a_world_day_of_normal_play_costs_112_9_fp() {
        // From a full food bar, as the mockup's day case runs it: food then
        // never reaches 0 while the bar refills, where design 12's starving
        // rule (which the planning model leaves out) would take over.
        let mut b = Body::new(100.0);
        let mut t = 0.0;
        for _ in 0..18 {
            b.run(20.0, Effort::Sprint);
            b.run(20.0, Effort::None);
            t += 40.0;
        }
        for _ in 0..2 {
            b.run(60.0, Effort::Swim);
            b.run(10.0, Effort::None);
            t += 70.0;
        }
        for _ in 0..40 {
            b.burst(Burst::Jump);
            b.run(5.0, Effort::None);
            t += 5.0;
        }
        for _ in 0..30 {
            b.burst(Burst::Climb(1.0));
            b.run(3.0, Effort::None);
            t += 3.0;
        }
        b.run(2880.0 - t, Effort::None);
        close(b.spent, 112.9);
        close(b.effort_fp, 48.9);
    }

    #[test]
    fn the_bottom_half_costs_less_than_the_top() {
        let cfg = SurvivalConfig::shipped().stamina;
        let refill = |from: f32, to: f32| {
            let mut st = Stamina {
                pts: from,
                strain: 0.0,
                since_spend_s: 10.0,
                winded: false,
            };
            let mut fp = 0.0;
            while st.pts < to - 1e-4 {
                let before = st.pts;
                fp += st.step(&cfg, Effort::None, 80.0, false, 0.01).food_fp;
                if st.pts > to {
                    // Only count up to `to`: the step's share past it.
                    let over = (st.pts - to) / (st.pts - before);
                    fp -= over * Stamina::cost_fp_per_pt(&cfg, to) * (st.pts - before);
                    break;
                }
            }
            fp
        };
        let bottom = refill(0.0, 50.0);
        let top = refill(50.0, 100.0);
        assert!((bottom - 1.0).abs() < 0.02, "bottom {bottom}");
        assert!((top - 1.75).abs() < 0.02, "top {top}");
        assert!(bottom < top);
    }

    #[test]
    fn tool_work_spends_nothing() {
        let mut b = Body::new(80.0);
        b.run(60.0, Effort::None);
        assert_eq!(b.st.pts, 100.0);
        assert_eq!(b.effort_fp, 0.0);
    }

    #[test]
    fn empty_is_winded_until_a_quarter_is_back() {
        let cfg = SurvivalConfig::shipped().stamina;
        let mut st = Stamina::full(&cfg);
        let mut winded = false;
        for _ in 0..13_000 {
            winded |= st.step(&cfg, Effort::Swim, 80.0, false, 0.01).became_winded;
        }
        assert!(winded && st.winded);
        let lim = st.limits(&cfg);
        assert!(!lim.can_sprint && lim.floats && lim.swim_speed_mult == 0.5);
        // Treading water still spends; resting refills.
        let mut recovered = false;
        for _ in 0..400 {
            recovered |= st.step(&cfg, Effort::None, 80.0, false, 0.01).recovered;
        }
        assert!(recovered && !st.winded && st.pts >= 25.0);
    }

    #[test]
    fn starving_refills_only_to_half_for_nothing() {
        let cfg = SurvivalConfig::shipped().stamina;
        let mut st = Stamina {
            pts: 0.0,
            strain: 0.0,
            since_spend_s: 10.0,
            winded: false,
        };
        let mut fp = 0.0;
        for _ in 0..3000 {
            fp += st.step(&cfg, Effort::None, 0.0, false, 0.01).food_fp;
        }
        assert_eq!(fp, 0.0);
        assert!((st.pts - 50.0).abs() < 1e-3);
    }

    #[test]
    fn cold_halves_the_refill() {
        let cfg = SurvivalConfig::shipped().stamina;
        let mut warm = Stamina {
            pts: 0.0,
            strain: 0.0,
            since_spend_s: 10.0,
            winded: false,
        };
        let mut cold = warm;
        warm.step(&cfg, Effort::None, 80.0, false, 1.0);
        cold.step(&cfg, Effort::None, 80.0, true, 1.0);
        assert!((warm.pts - 12.0).abs() < 1e-4 && (cold.pts - 6.0).abs() < 1e-4);
    }
}
