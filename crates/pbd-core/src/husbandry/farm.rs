//! Needs, mood, health, straying and products, as a closed form of world
//! time (decisions 6 and 7; S7, S8, S9).
//!
//! An animal's state is what the player last wrote (`as_of`, health, days
//! hungry, product tallies). Under conditions that do not change, which is
//! what a farm is while nobody touches it, every quantity moves linearly
//! and is clamped, so [`FarmState::advance`] jumps straight to any later
//! time. The one change it knows of in advance, a hand feed running out, it
//! splits at. Advancing by `a` then `b` is advancing by `a + b`: the
//! stepwise answer and the closed form agree, which is what
//! `world-persistence` asks of every process and what a test pins.

use super::{Keeping, NeedSettings, Sex};
use std::collections::BTreeMap;

/// What a pen gives an animal, as the app reads it off the world: unchanged
/// while the player is away.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Conditions {
    /// The pen's grass feeds the herd in it.
    pub grazing: bool,
    /// A pen cell is next to water.
    pub water: bool,
    /// Under a roof (no buildings yet: never, in this slice).
    pub shelter: bool,
    /// The pen's cells are enough for the herd in it.
    pub space: bool,
}

/// An animal's farm state at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct FarmState {
    /// World days (world seconds over `DAY_S`).
    pub as_of: f64,
    /// 0..100.
    pub health: f32,
    /// World days since it last had food.
    pub hungry: f32,
    /// A hand feed covers food until this world day.
    pub fed_until: f64,
    /// Each product's tally by good key: whole units are waiting.
    pub made: BTreeMap<String, f32>,
}

impl FarmState {
    /// A newly kept animal: healthy, fed, nothing waiting.
    pub fn new(as_of: f64) -> Self {
        Self {
            as_of,
            health: 100.0,
            hungry: 0.0,
            fed_until: as_of,
            made: BTreeMap::new(),
        }
    }

    /// The needs met at `day` under `c`.
    pub fn met(&self, c: Conditions, day: f64) -> u32 {
        let food = c.grazing || day < self.fed_until;
        [food, c.water, c.shelter, c.space]
            .into_iter()
            .filter(|m| *m)
            .count() as u32
    }

    /// Its mood at `day`: each need met adds the same.
    pub fn mood(&self, c: Conditions, day: f64, n: &NeedSettings) -> f32 {
        self.met(c, day) as f32 * n.mood_per_need
    }

    pub fn sick(&self, n: &NeedSettings) -> bool {
        self.health < n.sick_below
    }

    /// Whether it has gone looking for food (decision 6: neglect never
    /// kills; food brings it back).
    pub fn strayed(&self, n: &NeedSettings) -> bool {
        self.hungry >= n.stray_after_days
    }

    /// The state at world day `to` (no earlier than `as_of`), the conditions
    /// held all the while.
    pub fn advance(
        &self,
        to: f64,
        c: Conditions,
        keep: &Keeping,
        sex: Sex,
        yield_factor: f32,
        n: &NeedSettings,
    ) -> FarmState {
        let mut s = self.clone();
        if to.partial_cmp(&s.as_of) != Some(std::cmp::Ordering::Greater) {
            return s;
        }
        // A hand feed running out is the one boundary inside the span.
        if s.fed_until > s.as_of && s.fed_until < to {
            let split = s.fed_until;
            s = s.segment(split, c, keep, sex, yield_factor, n);
        }
        s.segment(to, c, keep, sex, yield_factor, n)
    }

    /// One span over which the needs met do not change.
    fn segment(
        mut self,
        to: f64,
        c: Conditions,
        keep: &Keeping,
        sex: Sex,
        yield_factor: f32,
        n: &NeedSettings,
    ) -> FarmState {
        let dt = (to - self.as_of) as f32;
        let met = self.met(c, self.as_of);
        let food = c.grazing || self.as_of < self.fed_until;
        let content = met as f32 * n.mood_per_need >= n.content_mood;
        // Health falls while it is not content, else recovers (S8: shelter
        // cannot be met before buildings, so decision 6's "two or more unmet"
        // would sicken every animal; it returns with them).
        let rate = if content {
            n.health_per_day
        } else {
            -n.health_per_day
        };
        let h0 = self.health;
        self.health = (h0 + rate * dt).clamp(0.0, 100.0);
        // Productive time: content, and not sick. Content means health is
        // rising, so it is the span after it climbs past the sick line.
        let productive = if content {
            let wait = ((n.sick_below - h0) / n.health_per_day).max(0.0);
            (dt - wait).max(0.0)
        } else {
            0.0
        };
        if productive > 0.0 {
            for p in keep.products.iter().filter(|p| p.gives(sex)) {
                let per_day = p.amount * yield_factor / p.period_days;
                let cap = per_day * n.cap_days.max(p.period_days);
                let tally = self.made.entry(p.good.clone()).or_insert(0.0);
                *tally = (*tally + per_day * productive).min(cap.max(*tally));
            }
        }
        self.hungry = if food { 0.0 } else { self.hungry + dt };
        self.as_of = to;
        self
    }

    /// Whole units of `good` waiting.
    pub fn waiting(&self, good: &str) -> u32 {
        self.made.get(good).map_or(0, |t| t.max(0.0).floor() as u32)
    }

    /// Take every whole unit waiting, keeping the fraction; how many.
    pub fn collect(&mut self, good: &str) -> u32 {
        let whole = self.waiting(good);
        if let Some(t) = self.made.get_mut(good) {
            *t -= whole as f32;
        }
        whole
    }

    /// A hand feed at `day`: food until `day + hand_feed_days`, and no
    /// longer hungry.
    pub fn hand_feed(&mut self, day: f64, n: &NeedSettings) {
        self.fed_until = self.fed_until.max(day + f64::from(n.hand_feed_days));
        self.hungry = 0.0;
    }
}
