//! Life stages and how each kept species is born (`animal-husbandry` design
//! 8 and 13, E1). A stage is DERIVED from the birth day and never stored, so
//! retuning a stage never touches a save.

use super::Sex;
use serde::{Deserialize, Serialize};

/// One stage before adulthood.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// What the young is called in it: chick, lamb.
    pub name: String,
    /// The age it ends at, world days.
    pub until_days: f32,
    /// Its size against the adult's, 0..1.
    pub size: f32,
}

/// How the young come.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum Birth {
    /// Laid as eggs and hatched under a broody hen or in an incubator.
    Eggs {
        /// World days from setting to hatching.
        incubation_days: f32,
        /// How many one broody hen sits on.
        clutch_max: u8,
    },
    /// Carried and born live.
    Live {
        /// World days from conception to birth; also the cooldown after one.
        gestation_days: f32,
        /// Young in a litter, before inbreeding, inclusive.
        litter: (u8, u8),
    },
}

/// A kept species' young.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Young {
    /// The stages in order; the last ends at `Keeping::adult_days`.
    pub stages: Vec<Stage>,
    /// The adult's name, female then male: hen and cockerel.
    pub adult: (String, String),
    pub birth: Birth,
}

impl Young {
    /// The stage at `day` of an animal born on `born`, and its size against
    /// the adult's. A tamed animal was born adult (`born` is set back by
    /// `adult_days` when it is tamed), so it reads as an adult.
    pub fn stage(&self, sex: Sex, born: f64, day: f64) -> (&str, f32) {
        let age = (day - born) as f32;
        for stage in &self.stages {
            if age < stage.until_days {
                return (&stage.name, stage.size);
            }
        }
        let name = match sex {
            Sex::F => &self.adult.0,
            Sex::M => &self.adult.1,
        };
        (name, 1.0)
    }

    /// The age adulthood starts at: the end of the last stage.
    pub fn adult_at(&self) -> f32 {
        self.stages.last().map_or(0.0, |s| s.until_days)
    }

    pub fn validate(&self, at: &str, adult_days: f32) -> Result<(), String> {
        let mut last = 0.0;
        for s in &self.stages {
            if s.name.is_empty()
                || s.until_days.partial_cmp(&last) != Some(std::cmp::Ordering::Greater)
                || !(0.0..1.0).contains(&s.size)
                || s.size == 0.0
            {
                return Err(format!(
                    "{at}: stages are named, in order, and smaller than the adult"
                ));
            }
            last = s.until_days;
        }
        if (self.adult_at() - adult_days).abs() > 1e-4 {
            return Err(format!("{at}: the last stage ends at adult_days"));
        }
        match self.birth {
            Birth::Eggs {
                incubation_days,
                clutch_max,
            } if incubation_days > 0.0 && clutch_max > 0 => Ok(()),
            Birth::Live {
                gestation_days,
                litter,
            } if gestation_days > 0.0 && litter.0 > 0 && litter.1 >= litter.0 => Ok(()),
            _ => Err(format!("{at}: a birth takes time and gives young")),
        }
    }
}
