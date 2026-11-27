//! Water and drinking (`survival-mode` design 15).
//!
//! A water bar that play time empties, faster in heat and with hard work.
//! Only fresh water fills it: the sea, a brackish river mouth and a bucket
//! filled at the sea give nothing and the note "Salt water" (the owner,
//! 2026-11-27: "salt water is undrinkable", "that should be one way to test
//! water salinity"). A sip of salt water costs nothing either: trying is how
//! a player tells.

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;
use crate::locale::{Localized, tr};

/// Where a sip comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterSource {
    /// A river, a lake or a swamp pool (`planet_gen::fresh_water_at`).
    Fresh,
    /// The sea.
    Sea,
    /// A river's last cells before the sea: salt to drink.
    Brackish,
    /// A water bucket in hand, filled where `salt` says.
    Bucket { salt: bool },
    /// A standing desert cactus, with a straw in hand.
    Cactus,
    /// A still's fresh water.
    Still,
}

impl WaterSource {
    /// Whether it is salt: the sea, a river mouth, a bucket filled at either.
    pub fn is_salt(self) -> bool {
        matches!(
            self,
            WaterSource::Sea | WaterSource::Brackish | WaterSource::Bucket { salt: true }
        )
    }
}

/// A short line the game shows above the hotbar for a survival event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Note {
    /// A sip of salt water: nothing taken.
    SaltWater,
    /// A cactus with no sips left.
    Dry,
    /// The water bar fell through half.
    Thirsty,
    /// The water bar fell through a quarter.
    VeryThirsty,
    /// A strike at a protected animal.
    Protected,
}

impl Note {
    /// Its words, from the locale file.
    pub fn text(self) -> Localized {
        tr(match self {
            Note::SaltWater => "survival.note.salt_water",
            Note::Dry => "survival.note.dry",
            Note::Thirsty => "survival.note.thirsty",
            Note::VeryThirsty => "survival.note.very_thirsty",
            Note::Protected => "survival.note.protected",
        })
    }
}

/// What a sip did: the result the game shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sip {
    /// Water gained.
    pub water: f32,
    /// Seconds of play the sip takes.
    pub sip_s: f32,
    /// The note to show, if any.
    pub note: Option<Note>,
    /// Whether it counts for a `drink` objective (fresh water only).
    pub counts: bool,
}

/// A sip from `source`, given the bar now (a sip never raises it past the
/// top). A cactus's or a bucket's own count of sips is the caller's
/// ([`CactusDrunk`], the bucket's item).
pub fn sip(cfg: &SurvivalConfig, source: WaterSource, water_now: f32) -> Sip {
    let d = &cfg.drinks;
    let (gain, sip_s) = match source {
        _ if source.is_salt() => {
            return Sip {
                water: 0.0,
                sip_s: d.fresh.sip_s,
                note: Some(Note::SaltWater),
                counts: false,
            };
        }
        WaterSource::Fresh => (d.fresh.water, d.fresh.sip_s),
        WaterSource::Bucket { .. } => (d.bucket.water, d.bucket.sip_s),
        WaterSource::Cactus => (d.cactus.water, d.cactus.sip_s),
        WaterSource::Still => (d.still.water, d.still.sip_s),
        // Salt, answered above.
        WaterSource::Sea | WaterSource::Brackish => (0.0, d.fresh.sip_s),
    };
    Sip {
        water: gain.min((cfg.water.max - water_now).max(0.0)),
        sip_s,
        note: None,
        counts: true,
    }
}

/// Water standing in rain with an empty hand held up gives over `dt`.
pub fn rain_catch(cfg: &SurvivalConfig, raining: bool, dt: f32) -> f32 {
    if raining {
        cfg.drinks.rain_per_s * dt
    } else {
        0.0
    }
}

/// Water a second of play uses at `feel_c`, with `hard` work going on.
pub fn drain_per_s(cfg: &SurvivalConfig, feel_c: f32, hard: bool) -> f32 {
    let w = &cfg.water;
    let base = w.max / (w.comfort_empty_min * 60.0);
    let mult = if feel_c > w.heat_above_c {
        1.0 + (feel_c - w.heat_above_c) * w.heat_per_c_above
    } else if feel_c < w.cold_below_c {
        w.cold_mult
    } else {
        1.0
    };
    base * mult + if hard { w.exertion_per_s } else { 0.0 }
}

/// The thirst note the bar falling from `before` to `after` raises, once as
/// it passes each threshold (design 18, "Water before the fire").
pub fn thirst_note(cfg: &SurvivalConfig, before: f32, after: f32) -> Option<Note> {
    let w = &cfg.water;
    if before > w.very_thirsty_at && after <= w.very_thirsty_at {
        Some(Note::VeryThirsty)
    } else if before > w.thirsty_at && after <= w.thirsty_at {
        Some(Note::Thirsty)
    } else {
        None
    }
}

/// A cactus drunk from (design 15), kept as a record keyed like a tree's,
/// in `FruitPicked`'s shape.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct CactusDrunk {
    /// The world day of the last sip.
    pub day: f64,
    /// Sips owed back at that day.
    pub taken: u8,
}

impl CactusDrunk {
    /// Sips the cactus holds at `day`: one back a `regrow_days`.
    pub fn sips_left(cfg: &SurvivalConfig, record: Option<&CactusDrunk>, day: f64) -> u8 {
        let full = cfg.drinks.cactus.sips;
        let Some(r) = record else {
            return full;
        };
        let back = ((day - r.day).max(0.0) / cfg.drinks.cactus.regrow_days).floor();
        let owed = (f64::from(r.taken) - back).max(0.0) as u8;
        full.saturating_sub(owed)
    }

    /// A sip at `day`: the record to write and the sip, or the note "Dry"
    /// and no record change.
    pub fn drink(
        cfg: &SurvivalConfig,
        record: Option<&CactusDrunk>,
        day: f64,
        water_now: f32,
    ) -> (Option<CactusDrunk>, Sip) {
        let left = Self::sips_left(cfg, record, day);
        if left == 0 {
            return (
                None,
                Sip {
                    water: 0.0,
                    sip_s: cfg.drinks.cactus.sip_s,
                    note: Some(Note::Dry),
                    counts: false,
                },
            );
        }
        let taken = cfg.drinks.cactus.sips - left + 1;
        (
            Some(CactusDrunk { day, taken }),
            sip(cfg, WaterSource::Cactus, water_now),
        )
    }
}

/// A still (design 18): salt water in, fresh water out while the fire
/// beside it burns; rain adds fresh water fire or not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Still {
    /// Sips of salt water poured in.
    pub salt_sips: u8,
    /// Sips of fresh water ready.
    pub fresh_sips: u8,
    /// Seconds of play toward the next boiled sip.
    pub boil_s: f32,
    /// Seconds of play toward the next sip of rain.
    pub rain_s: f32,
}

impl Still {
    /// Fill it with salt water (at the sea, or a bucket poured in), up to
    /// what it holds.
    pub fn fill_salt(&mut self, cfg: &SurvivalConfig, sips: u8) {
        let room = cfg
            .drinks
            .still
            .sips
            .saturating_sub(self.salt_sips + self.fresh_sips);
        self.salt_sips += sips.min(room);
    }

    /// `dt` seconds of play beside a fire that is `fire_lit`, in `raining`.
    pub fn step(&mut self, cfg: &SurvivalConfig, fire_lit: bool, raining: bool, dt: f32) {
        let cap = cfg.drinks.still.sips;
        if fire_lit && self.salt_sips > 0 {
            self.boil_s += dt;
            while self.boil_s >= cfg.still.boil_s_per_sip && self.salt_sips > 0 {
                self.boil_s -= cfg.still.boil_s_per_sip;
                self.salt_sips -= 1;
                self.fresh_sips += 1;
            }
        } else {
            self.boil_s = 0.0;
        }
        if raining && self.salt_sips + self.fresh_sips < cap {
            self.rain_s += dt;
            while self.rain_s >= cfg.still.rain_s_per_sip && self.salt_sips + self.fresh_sips < cap
            {
                self.rain_s -= cfg.still.rain_s_per_sip;
                self.fresh_sips += 1;
            }
        } else {
            self.rain_s = 0.0;
        }
    }

    /// A sip from it: fresh if it has any, else nothing (and no note: it
    /// holds no salt water to drink, only to boil).
    pub fn drink(&mut self, cfg: &SurvivalConfig, water_now: f32) -> Option<Sip> {
        if self.fresh_sips == 0 {
            return None;
        }
        self.fresh_sips -= 1;
        Some(sip(cfg, WaterSource::Still, water_now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sea_water_gives_nothing_and_says_salt_water() {
        let cfg = SurvivalConfig::shipped();
        for source in [
            WaterSource::Sea,
            WaterSource::Brackish,
            WaterSource::Bucket { salt: true },
        ] {
            let s = sip(&cfg, source, 40.0);
            assert_eq!(s.water, 0.0);
            assert_eq!(s.note, Some(Note::SaltWater));
            assert!(!s.counts);
        }
        assert!(!Note::SaltWater.text().is_empty());
    }

    #[test]
    fn a_river_sip_fills_and_counts() {
        let cfg = SurvivalConfig::shipped();
        let s = sip(&cfg, WaterSource::Fresh, 40.0);
        assert_eq!((s.water, s.counts, s.note), (10.0, true, None));
        assert_eq!(sip(&cfg, WaterSource::Fresh, 95.0).water, 5.0);
        assert_eq!(
            sip(&cfg, WaterSource::Bucket { salt: false }, 0.0).water,
            15.0
        );
    }

    #[test]
    fn heat_uses_water_faster_and_cold_slower() {
        let cfg = SurvivalConfig::shipped();
        let comfort = drain_per_s(&cfg, 20.0, false);
        assert!((cfg.water.max / comfort / 60.0 - 50.0).abs() < 1e-3);
        let desert = drain_per_s(&cfg, 43.4, false);
        assert!((desert / comfort - 2.74).abs() < 1e-3);
        assert!((cfg.water.max / desert / 60.0 - 18.2).abs() < 0.1);
        assert!((drain_per_s(&cfg, 0.0, false) / comfort - 0.8).abs() < 1e-5);
        assert!((drain_per_s(&cfg, 20.0, true) - comfort - 0.02).abs() < 1e-6);
    }

    #[test]
    fn a_cactus_gives_three_sips_and_one_back_a_day_across_a_reload() {
        let cfg = SurvivalConfig::shipped();
        let mut record: Option<CactusDrunk> = None;
        let mut water = 10.0;
        for _ in 0..3 {
            let (next, s) = CactusDrunk::drink(&cfg, record.as_ref(), 4.2, water);
            assert_eq!(s.water, 8.0);
            water += s.water;
            record = next;
        }
        let (none, dry) = CactusDrunk::drink(&cfg, record.as_ref(), 4.3, water);
        assert_eq!((none, dry.water, dry.note), (None, 0.0, Some(Note::Dry)));
        // Saved and reopened the same day: still dry.
        let saved = ron::to_string(&record.unwrap()).unwrap();
        let back: CactusDrunk = ron::from_str(&saved).unwrap();
        assert_eq!(CactusDrunk::sips_left(&cfg, Some(&back), 4.9), 0);
        // A world day later, one sip is back; three days, all of them.
        assert_eq!(CactusDrunk::sips_left(&cfg, Some(&back), 5.2), 1);
        assert_eq!(CactusDrunk::sips_left(&cfg, Some(&back), 7.2), 3);
        let (_, s) = CactusDrunk::drink(&cfg, Some(&back), 5.2, 0.0);
        assert_eq!(s.water, 8.0);
    }

    #[test]
    fn the_still_makes_fresh_water_only_beside_a_lit_fire() {
        let cfg = SurvivalConfig::shipped();
        let mut still = Still::default();
        still.fill_salt(&cfg, 10);
        assert_eq!(still.salt_sips, 6);
        still.step(&cfg, false, false, 600.0);
        assert_eq!(still.drink(&cfg, 0.0), None);
        for _ in 0..150 {
            still.step(&cfg, true, false, 1.0);
        }
        assert_eq!((still.salt_sips, still.fresh_sips), (4, 2));
        let s = still.drink(&cfg, 0.0).unwrap();
        assert_eq!((s.water, s.counts), (10.0, true));
    }

    #[test]
    fn rain_fills_a_held_hand_and_the_still() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(rain_catch(&cfg, true, 2.0), 4.0);
        assert_eq!(rain_catch(&cfg, false, 2.0), 0.0);
        let mut still = Still::default();
        for _ in 0..61 {
            still.step(&cfg, false, true, 1.0);
        }
        assert_eq!(still.fresh_sips, 1);
    }

    #[test]
    fn thirsty_shows_once_at_half_and_again_at_a_quarter() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(thirst_note(&cfg, 50.1, 49.9), Some(Note::Thirsty));
        assert_eq!(thirst_note(&cfg, 49.9, 49.0), None);
        assert_eq!(thirst_note(&cfg, 25.1, 24.9), Some(Note::VeryThirsty));
    }
}
