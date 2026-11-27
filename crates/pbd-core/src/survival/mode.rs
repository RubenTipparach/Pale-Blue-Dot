//! The world's mode (`survival-mode` design 1): Explorer or Survival, chosen
//! when the world is made, stored in its identity record and fixed after.

use serde::{Deserialize, Serialize};

/// How a world plays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum WorldMode {
    /// The calm mode: no health, hunger, stamina, water, breath, wear or
    /// predators that hunt. Every world made before modes existed.
    #[default]
    Explorer,
    /// Every rule of [`crate::survival`].
    Survival,
}

impl WorldMode {
    /// The mode a world's identity names, or Explorer for a world saved
    /// before there were modes (CLAUDE.md, "Saved games survive every
    /// change"): no world changes under a player.
    pub fn of_saved(saved: Option<WorldMode>) -> Self {
        saved.unwrap_or(WorldMode::Explorer)
    }

    /// Whether the survival rules run.
    pub fn is_survival(self) -> bool {
        self == WorldMode::Survival
    }

    /// The mode's name on the new-world screen and in the world list.
    pub fn name(self) -> crate::locale::Localized {
        crate::locale::tr(match self {
            WorldMode::Explorer => "survival.mode.explorer",
            WorldMode::Survival => "survival.mode.survival",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An identity written before modes, shaped as the app's: no `mode`.
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Identity {
        seed: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<WorldMode>,
    }

    #[test]
    fn an_old_save_opens_as_explorer() {
        let old: Identity = ron::from_str("(seed: 7)").unwrap();
        assert_eq!(WorldMode::of_saved(old.mode), WorldMode::Explorer);
        assert!(!WorldMode::of_saved(old.mode).is_survival());
    }

    #[test]
    fn a_survival_world_reopens_as_survival() {
        let made = Identity {
            seed: 7,
            mode: Some(WorldMode::Survival),
        };
        let text = ron::to_string(&made).unwrap();
        let back: Identity = ron::from_str(&text).unwrap();
        assert_eq!(WorldMode::of_saved(back.mode), WorldMode::Survival);
        assert_eq!(back, made);
    }
}
