//! Hunting (`survival-mode` design 17): Survival only. Wild game takes
//! damage and dies; a kill drops its meat and bones at once and its hide
//! whole when the carcass is dressed; a killed herd member comes back after
//! `herd_back_days` world days of play, kept by a `hunted` record so a
//! reload does not bring it straight back. Protected species and every
//! animal a player keeps or bought are never harmed.

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;
use super::mode::WorldMode;
use super::water::Note;

/// What a strike landed on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Quarry {
    /// A wild animal of `hunting.game`, by species id.
    Wild(String),
    /// A protected species (`fauna.ron`'s `status: Protected`).
    Protected,
    /// An animal a player keeps or bought: tamed, penned, the horse.
    Kept,
}

/// A good and a count.
pub type Drops = Vec<(String, u16)>;

/// What a strike did.
#[derive(Clone, Debug, PartialEq)]
pub enum Strike {
    /// Nothing: Explorer, a kept animal (no note), or a protected one ("Protected").
    Refused(Option<Note>),
    /// Hurt, with this much health left; it runs.
    Hurt { left: f32 },
    /// Killed: what drops at once. The carcass lies for dressing.
    Killed { drops: Drops },
}

/// A creature's health at full, for a species of `hunting.game`.
pub fn full_health(cfg: &SurvivalConfig, species: &str) -> Option<f32> {
    cfg.hunting.game.get(species).map(|g| g.health)
}

/// A strike of `damage` on `quarry` with `health` left, in a world of `mode`.
pub fn strike(
    cfg: &SurvivalConfig,
    mode: WorldMode,
    quarry: &Quarry,
    health: f32,
    damage: f32,
) -> Strike {
    if !mode.is_survival() {
        return Strike::Refused(None);
    }
    let species = match quarry {
        Quarry::Protected => return Strike::Refused(Some(Note::Protected)),
        Quarry::Kept => return Strike::Refused(None),
        Quarry::Wild(species) => species,
    };
    let Some(game) = cfg.hunting.game.get(species) else {
        return Strike::Refused(None);
    };
    let left = (health - damage.max(0.0)).max(0.0);
    if left > 0.0 {
        Strike::Hurt { left }
    } else {
        Strike::Killed {
            drops: game.drops.clone(),
        }
    }
}

/// The blade that dresses a carcass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blade {
    HuntingAxe,
    /// Any other blade: a spear, a sword, an axe.
    Other,
}

/// What dressing a carcass gives: its hides and, with a hunting axe,
/// `dress_bonus_meat` more meat; any other blade takes the hides
/// `other_blade_hide_share` of the time, by `kill_hash` so a replay agrees.
pub fn dress(cfg: &SurvivalConfig, species: &str, blade: Blade, kill_hash: u64) -> Drops {
    let h = &cfg.hunting;
    let Some(game) = h.game.get(species) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let hide = match blade {
        Blade::HuntingAxe => true,
        Blade::Other => {
            let roll = (mix(kill_hash) >> 11) as f64 / (1u64 << 53) as f64;
            roll < f64::from(h.other_blade_hide_share)
        }
    };
    if hide && game.hide > 0 {
        out.push((h.hide.clone(), game.hide));
    }
    if blade == Blade::HuntingAxe
        && h.dress_bonus_meat > 0
        && game.drops.iter().any(|(g, _)| *g == h.meat)
    {
        out.push((h.meat.clone(), h.dress_bonus_meat));
    }
    out
}

/// Whether an undressed carcass is still there `lain_s` seconds after the
/// kill (after it, it is gone with its hide).
pub fn carcass_lies(cfg: &SurvivalConfig, lain_s: f32) -> bool {
    lain_s < cfg.hunting.carcass_s
}

/// A herd member killed: the `hunted` record, keyed by its herd square.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Hunted {
    /// The herd square's id (`husbandry.ron` `herds`).
    pub square: u64,
    /// Which member of the square's herd.
    pub member: u32,
    /// The world day of the kill.
    pub day: f64,
}

impl Hunted {
    /// Whether the member is back at `day` (world days of play).
    pub fn back_at(&self, cfg: &SurvivalConfig, day: f64) -> bool {
        day - self.day >= cfg.hunting.herd_back_days
    }
}

/// A 64-bit mix (splitmix64's finaliser), for the rolls a replay must agree
/// on.
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn woolback() -> Quarry {
        Quarry::Wild("woolback".into())
    }

    #[test]
    fn a_woolback_hunted_and_dressed() {
        let cfg = SurvivalConfig::shipped();
        let hp = full_health(&cfg, "woolback").unwrap();
        assert_eq!(hp, 40.0);
        let first = strike(&cfg, WorldMode::Survival, &woolback(), hp, 24.0);
        assert_eq!(first, Strike::Hurt { left: 16.0 });
        let Strike::Killed { drops } = strike(&cfg, WorldMode::Survival, &woolback(), 16.0, 24.0)
        else {
            panic!("not killed");
        };
        let s = |g: &str, n| (g.to_string(), n);
        assert_eq!(drops, vec![s("raw-meat", 3), s("wool", 2), s("bone", 2)]);
        assert_eq!(
            dress(&cfg, "woolback", Blade::HuntingAxe, 1),
            vec![s("hide", 1), s("raw-meat", 1)]
        );
        assert!(carcass_lies(&cfg, 119.0) && !carcass_lies(&cfg, 120.0));
    }

    #[test]
    fn another_blade_takes_the_hide_about_half_the_time() {
        let cfg = SurvivalConfig::shipped();
        let hides = (0..1000u64)
            .filter(|k| !dress(&cfg, "hare", Blade::Other, *k).is_empty())
            .count();
        assert!((420..580).contains(&hides), "{hides}");
        assert_eq!(
            dress(&cfg, "hare", Blade::Other, 5),
            dress(&cfg, "hare", Blade::Other, 5)
        );
    }

    #[test]
    fn protected_and_kept_animals_are_refused() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(
            strike(&cfg, WorldMode::Survival, &Quarry::Protected, 10.0, 30.0),
            Strike::Refused(Some(Note::Protected))
        );
        assert_eq!(
            strike(&cfg, WorldMode::Survival, &Quarry::Kept, 10.0, 30.0),
            Strike::Refused(None)
        );
        assert_eq!(
            strike(&cfg, WorldMode::Explorer, &woolback(), 40.0, 30.0),
            Strike::Refused(None)
        );
    }

    #[test]
    fn a_herd_member_is_back_after_three_days_and_not_before_a_reload() {
        let cfg = SurvivalConfig::shipped();
        let kill = Hunted {
            square: 0x51,
            member: 2,
            day: 10.25,
        };
        let back: Hunted = ron::from_str(&ron::to_string(&kill).unwrap()).unwrap();
        assert_eq!(back, kill);
        assert!(!back.back_at(&cfg, 10.3));
        assert!(!back.back_at(&cfg, 13.2));
        assert!(back.back_at(&cfg, 13.25));
    }
}
