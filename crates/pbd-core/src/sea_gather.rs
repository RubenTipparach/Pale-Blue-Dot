//! Gathering the sea floor (`sea-gathering`): what the sea's pieces give
//! when taken by hand, how long each takes to grow back, what the grinder
//! makes of it, and which can be set down again as an ornament; the
//! records a take and an ornament are kept as; and which creatures a hand
//! may take.
//!
//! The pieces themselves are derived where they are drawn
//! (`pbd-app`'s `fish::places`): only a take is stored, as a
//! [`SEA_TAKE_RECORD`] keyed by the piece's derived id.

use crate::fauna::{Mover, Species, Status};
use crate::goods::Good;
use crate::locale::{Localized, tr};
use serde::{Deserialize, Serialize};

/// What one of the sea's pieces is, as an item (design 1): one per model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SeaPiece {
    StaghornCoral,
    FingerCoral,
    BrainCoral,
    TableCoral,
    SoftCoral,
    TubeSponge,
    BarrelSponge,
    SeaFan,
    SeaAnemone,
    SeaLettuce,
    CoralRubble,
    Mussels,
    Kelp,
}

impl SeaPiece {
    pub const ALL: [SeaPiece; 13] = [
        SeaPiece::StaghornCoral,
        SeaPiece::FingerCoral,
        SeaPiece::BrainCoral,
        SeaPiece::TableCoral,
        SeaPiece::SoftCoral,
        SeaPiece::TubeSponge,
        SeaPiece::BarrelSponge,
        SeaPiece::SeaFan,
        SeaPiece::SeaAnemone,
        SeaPiece::SeaLettuce,
        SeaPiece::CoralRubble,
        SeaPiece::Mussels,
        SeaPiece::Kelp,
    ];

    /// Its good's code, the saved name.
    pub fn code(self) -> &'static str {
        match self {
            SeaPiece::StaghornCoral => "staghorn-coral",
            SeaPiece::FingerCoral => "finger-coral",
            SeaPiece::BrainCoral => "brain-coral",
            SeaPiece::TableCoral => "table-coral",
            SeaPiece::SoftCoral => "soft-coral",
            SeaPiece::TubeSponge => "tube-sponge",
            SeaPiece::BarrelSponge => "barrel-sponge",
            SeaPiece::SeaFan => "sea-fan",
            SeaPiece::SeaAnemone => "sea-anemone",
            SeaPiece::SeaLettuce => "sea-lettuce",
            SeaPiece::CoralRubble => "coral-rubble",
            SeaPiece::Mussels => "mussels",
            SeaPiece::Kelp => "kelp",
        }
    }

    pub fn by_code(code: &str) -> Option<SeaPiece> {
        SeaPiece::ALL.into_iter().find(|p| p.code() == code)
    }

    /// Its good.
    pub fn good(self) -> Good {
        Good::by_code(self.code()).expect("every sea piece is a good")
    }

    /// The piece a good is, if it is one.
    pub fn of_good(good: Good) -> Option<SeaPiece> {
        SeaPiece::by_code(good.code())
    }

    pub fn name(self) -> Localized {
        self.good().name()
    }

    /// World days a taken piece takes to grow back (design 3).
    pub fn regrow_days(self) -> f64 {
        match self {
            SeaPiece::BrainCoral | SeaPiece::TableCoral => 8.0,
            SeaPiece::BarrelSponge => 5.0,
            SeaPiece::StaghornCoral | SeaPiece::FingerCoral | SeaPiece::SeaFan => 4.0,
            SeaPiece::SoftCoral
            | SeaPiece::TubeSponge
            | SeaPiece::SeaAnemone
            | SeaPiece::Mussels => 3.0,
            SeaPiece::SeaLettuce | SeaPiece::CoralRubble | SeaPiece::Kelp => 2.0,
        }
    }

    /// What the grinder makes of it (design 6): so many in, so much
    /// fertiliser out. `None` for what is not crushed (mussels are food).
    pub fn crush(self) -> Option<(u16, u16)> {
        match self {
            SeaPiece::BrainCoral | SeaPiece::TableCoral => Some((1, 3)),
            SeaPiece::StaghornCoral | SeaPiece::FingerCoral | SeaPiece::BarrelSponge => {
                Some((1, 2))
            }
            SeaPiece::SoftCoral
            | SeaPiece::TubeSponge
            | SeaPiece::SeaFan
            | SeaPiece::SeaAnemone => Some((1, 1)),
            SeaPiece::SeaLettuce | SeaPiece::CoralRubble | SeaPiece::Kelp => Some((2, 1)),
            SeaPiece::Mussels => None,
        }
    }

    /// Whether it can be set down as an ornament (design 8): the corals,
    /// sponges, fans and anemones.
    pub fn ornament(self) -> bool {
        !matches!(
            self,
            SeaPiece::SeaLettuce | SeaPiece::CoralRubble | SeaPiece::Mussels | SeaPiece::Kelp
        )
    }

    /// The grinder's recipe code for it.
    pub fn crush_recipe(self) -> String {
        format!("crush-{}", self.code())
    }
}

/// The good the grinder makes of the sea's pieces (design 6).
pub const FERTILIZER: &str = "fertilizer";

/// A `sea-take` record (design 3): a sea piece taken, keyed by the piece's
/// derived id. It grows back [`SeaPiece::regrow_days`] after.
pub const SEA_TAKE_RECORD: &str = "sea-take";
pub const SEA_TAKE_SCHEMA: u32 = 1;

/// A `sea-take` record's body.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct SeaTake {
    /// The world day it was taken.
    pub day: f64,
    /// The good it gave, by code: which regrowth it keeps.
    pub code: String,
}

impl SeaTake {
    /// Whether it has grown back at `day`. A code this build does not know
    /// grows back on the longest clock.
    pub fn due_at(&self, day: f64) -> bool {
        let days = SeaPiece::by_code(&self.code).map_or(8.0, SeaPiece::regrow_days);
        day - self.day >= days
    }
}

/// An `ornament` record (design 8): a sea piece set down, keyed by the
/// cell's key, one to a cell.
pub const ORNAMENT_RECORD: &str = "ornament";
pub const ORNAMENT_SCHEMA: u32 = 1;

/// An `ornament` record's body. `at` is the body-relative point it stands
/// on, metres, in `f64` (CLAUDE.md: a stored position names its frame).
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Ornament {
    pub code: String,
    pub at: (f64, f64, f64),
    pub yaw: f32,
    /// False once it is taken back up.
    pub standing: bool,
}

impl Ornament {
    /// The piece it is, if this build knows it.
    pub fn piece(&self) -> Option<SeaPiece> {
        SeaPiece::by_code(&self.code).filter(|p| p.ornament())
    }
}

/// Whether a hand may take a creature (design 4): `None` where it may,
/// otherwise why not. A protected species is refused whatever reaches it;
/// a bed crawler is taken; anything else is caught with the rod.
pub fn hand_refusal(species: &Species) -> Option<Localized> {
    if species.status == Status::Protected {
        Some(tr("fauna.refusal.protected"))
    } else if species.mover != Mover::Crawler {
        Some(tr("sea.refusal.rod"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every piece is a good with a name, a value and a pack icon, and
    /// every crushed one has its grinder recipe to fertiliser at its yield.
    #[test]
    fn every_piece_is_a_good_valued_and_crushed_as_written() {
        let english = crate::locale::Locale::english();
        let values = crate::economy::values::Values::parse(include_str!(
            "../../../assets/config/values.ron"
        ))
        .unwrap();
        let book = crate::station::RecipeFile::default().resolve().unwrap();
        let fertilizer = Good::by_code(FERTILIZER).expect("fertiliser is a good");
        for piece in SeaPiece::ALL {
            let good = piece.good();
            assert_eq!(SeaPiece::of_good(good), Some(piece));
            assert!(
                english
                    .get(&format!("item.{}.name", piece.code()))
                    .is_some()
            );
            assert!(values.get(piece.code()).is_some(), "{}", piece.code());
            match piece.crush() {
                Some((n, out)) => {
                    let recipe = book.get(&piece.crush_recipe()).expect("its recipe");
                    assert_eq!(recipe.station.code, "grinder");
                    assert_eq!(recipe.inputs, vec![(crate::inventory::Item::Good(good), n)]);
                    assert_eq!(
                        recipe.outputs,
                        vec![(crate::inventory::Item::Good(fertilizer), out)]
                    );
                }
                None => assert!(book.get(&piece.crush_recipe()).is_none()),
            }
        }
        assert!(values.get(FERTILIZER).is_some());
    }

    /// A take grows back on its kind's clock, an unknown code on the
    /// longest, and the bodies round-trip.
    #[test]
    fn a_take_grows_back_on_its_kind_s_clock() {
        let take = SeaTake {
            day: 10.0,
            code: "kelp".into(),
        };
        assert!(!take.due_at(11.9));
        assert!(take.due_at(12.0));
        let coral = SeaTake {
            day: 10.0,
            code: "table-coral".into(),
        };
        assert!(!coral.due_at(17.0));
        assert!(coral.due_at(18.0));
        let odd = SeaTake {
            day: 0.0,
            code: "moon-coral".into(),
        };
        assert!(!odd.due_at(7.9) && odd.due_at(8.0));
        let back: SeaTake = ron::from_str(&ron::to_string(&coral).unwrap()).unwrap();
        assert_eq!(back, coral);
        let ornament = Ornament {
            code: "brain-coral".into(),
            at: (1.0, 2.0, 3.0),
            yaw: 0.5,
            standing: true,
        };
        let back: Ornament = ron::from_str(&ron::to_string(&ornament).unwrap()).unwrap();
        assert_eq!(back, ornament);
        assert_eq!(back.piece(), Some(SeaPiece::BrainCoral));
        let kelp = Ornament {
            code: "kelp".into(),
            ..ornament
        };
        assert_eq!(kelp.piece(), None, "kelp is not an ornament");
    }

    /// The hand takes crawlers, refuses protected creatures (D9), and
    /// leaves schooling fish to the rod.
    #[test]
    fn a_hand_takes_crawlers_and_refuses_the_protected() {
        let settings = crate::fauna::FaunaSettings::default();
        let roster = settings.roster(crate::fauna::HOME_BODY);
        let by = |id: &str| roster.iter().find(|s| s.id == id).unwrap();
        for id in [
            "shore_crab",
            "lobster",
            "giant_clam",
            "octopus",
            "tube_worm",
        ] {
            assert_eq!(hand_refusal(by(id)), None, "{id}");
        }
        for id in ["reef_turtle", "sea_otter", "seal", "reed_dabbler", "beaver"] {
            assert_eq!(
                hand_refusal(by(id)),
                Some(tr("fauna.refusal.protected")),
                "{id}"
            );
        }
        assert_eq!(hand_refusal(by("perch")), Some(tr("sea.refusal.rod")));
    }
}
