//! Each town's goodwill toward the player (`townsfolk` design 12): raised by
//! jobs done and by trade, worth better prices (up to 10%) and more jobs on
//! the board. Lowered only by the consequences change, later, never here.
//!
//! Stored as `goodwill` under the town's site id, written when it moves.

use serde::{Deserialize, Serialize};

pub const GOODWILL_KIND: &str = "goodwill";
pub const GOODWILL_SCHEMA: u32 = 1;

/// `trade.ron`'s `goodwill`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GoodwillConfig {
    /// Points a trade adds.
    pub per_trade: u32,
    /// Points a job finished adds.
    pub per_job: u32,
    /// The most a town holds.
    pub most: u32,
    /// Points in a step.
    pub step: u32,
    /// The price bonus each step gives, a share.
    pub bonus_per_step: f32,
    /// The steps at which the board shows one more job.
    pub more_jobs_at: Vec<u32>,
}

impl GoodwillConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.step == 0 || self.most < self.step {
            return Err("goodwill: step must be positive and no more than most".into());
        }
        if !(self.bonus_per_step.is_finite() && self.bonus_per_step >= 0.0)
            || self.bonus_per_step * self.steps_most() as f32 > 0.25
        {
            return Err("goodwill: the bonus at the most must be at most 25%".into());
        }
        Ok(())
    }

    /// The step `points` reach.
    pub fn level(&self, points: u32) -> u32 {
        points.min(self.most) / self.step
    }

    fn steps_most(&self) -> u32 {
        self.most / self.step
    }

    /// The price bonus at `points`: a share off what the trader asks and on
    /// what it pays.
    pub fn bonus(&self, points: u32) -> f32 {
        self.level(points) as f32 * self.bonus_per_step
    }

    /// How many more jobs than the base a board shows at `points`.
    pub fn extra_jobs(&self, points: u32) -> u32 {
        let level = self.level(points);
        self.more_jobs_at.iter().filter(|at| level >= **at).count() as u32
    }
}

/// A town's goodwill, as stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goodwill {
    pub points: u32,
}

impl Goodwill {
    /// Raise by `by`, held at the most. Whether it moved.
    pub fn raise(&mut self, by: u32, cfg: &GoodwillConfig) -> bool {
        let after = self.points.saturating_add(by).min(cfg.most);
        let moved = after != self.points;
        self.points = after;
        moved
    }
}
