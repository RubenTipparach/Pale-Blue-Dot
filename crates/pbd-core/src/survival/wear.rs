//! Tool wear (`survival-mode` design 11): Survival only. Each owned tool or
//! weapon carries the uses it has left; one use a finished action, two for a
//! block above the tool's tier; at zero it is broken (it works as bare
//! hands, and stays in its slot); a repair restores every use and never
//! raises a tier.

use serde::{Deserialize, Serialize};

use super::config::SurvivalConfig;
use super::mode::WorldMode;
use crate::inventory::{Tier, Tool};

/// What wears: a tool of the four, or a survival weapon or straw by good
/// code.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Wearer {
    Tool(Tool, Tier),
    /// A melee weapon of `weapons.melee`, by its good code.
    Weapon(String),
    Bow,
    Straw,
}

impl Wearer {
    /// Uses a new one has.
    pub fn max_uses(&self, cfg: &SurvivalConfig) -> u32 {
        let w = &cfg.wear;
        match self {
            Wearer::Tool(Tool::Rod, tier) => w.rod_uses.of(*tier),
            Wearer::Tool(_, tier) => w.tool_uses.of(*tier),
            Wearer::Weapon(code) => {
                let tier = cfg
                    .weapons
                    .melee
                    .get(code)
                    .map_or(Tier::Stone, |m| m.tier());
                w.tool_uses.of(tier)
            }
            Wearer::Bow => w.bow_uses,
            Wearer::Straw => w.straw_uses,
        }
    }

    fn tier(&self, cfg: &SurvivalConfig) -> Tier {
        match self {
            Wearer::Tool(_, tier) => *tier,
            Wearer::Weapon(code) => cfg
                .weapons
                .melee
                .get(code)
                .map_or(Tier::Stone, |m| m.tier()),
            Wearer::Bow | Wearer::Straw => Tier::Stone,
        }
    }
}

/// An action with a tool, for wear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// A block broken, of ore or rock `block_tier` (`Tier::reach`'s scale;
    /// `None` for one with no tier).
    BlockBroken {
        block_tier: Option<u8>,
    },
    TreeFelled,
    Tilled,
    FishLanded,
    /// A hit landed on a creature or a townsperson.
    HitLanded,
    /// A sip with a straw, an arrow loosed.
    Used,
    /// A swing at nothing.
    Missed,
    /// A dig given up half way.
    DigAbandoned,
    /// A cast that caught nothing.
    EmptyCast,
}

/// Uses an action spends with a tool of `tool_tier`.
pub fn uses_spent(cfg: &SurvivalConfig, action: Action, tool_tier: Tier) -> u32 {
    match action {
        Action::Missed | Action::DigAbandoned | Action::EmptyCast => 0,
        Action::BlockBroken {
            block_tier: Some(t),
        } if t > tool_tier.digit() => cfg.wear.harder_block_uses,
        _ => 1,
    }
}

/// An owned tool's wear, as the owned-tool record keeps it. A record
/// without it (an old save) reads full: `Option<Wear>` absent is
/// [`Wear::full`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Wear {
    pub uses_left: u32,
}

/// What a use did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spend {
    /// Nothing spent: Explorer, or an action that costs nothing, or a tool
    /// already broken.
    None,
    Spent(u32),
    /// It reached zero: "Pickaxe broke", the crack sound, bare hands now.
    Broke,
}

impl Wear {
    pub fn full(cfg: &SurvivalConfig, wearer: &Wearer) -> Self {
        Self {
            uses_left: wearer.max_uses(cfg),
        }
    }

    /// A saved wear, or full for a record from before tool wear.
    pub fn of_saved(cfg: &SurvivalConfig, wearer: &Wearer, saved: Option<Wear>) -> Self {
        saved.unwrap_or_else(|| Self::full(cfg, wearer))
    }

    pub fn is_broken(self) -> bool {
        self.uses_left == 0
    }

    /// An action in a world of `mode`. Explorer never wears.
    pub fn spend(
        &mut self,
        cfg: &SurvivalConfig,
        mode: WorldMode,
        wearer: &Wearer,
        action: Action,
    ) -> Spend {
        if !mode.is_survival() || self.is_broken() {
            return Spend::None;
        }
        let n = uses_spent(cfg, action, wearer.tier(cfg));
        if n == 0 {
            return Spend::None;
        }
        self.uses_left = self.uses_left.saturating_sub(n);
        if self.is_broken() {
            Spend::Broke
        } else {
            Spend::Spent(n)
        }
    }

    /// The share of uses left, 0..1, for the slot's bar.
    pub fn share(self, cfg: &SurvivalConfig, wearer: &Wearer) -> f32 {
        self.uses_left as f32 / wearer.max_uses(cfg).max(1) as f32
    }

    /// The slot's bar: shown once below full, red below a tenth.
    pub fn bar(self, cfg: &SurvivalConfig, wearer: &Wearer) -> Option<(f32, bool)> {
        let share = self.share(cfg, wearer);
        (share < 1.0).then_some((share, share < cfg.wear.red_below_share))
    }

    /// A repair: every use back, the tier unchanged.
    pub fn repair(&mut self, cfg: &SurvivalConfig, wearer: &Wearer) {
        *self = Self::full(cfg, wearer);
    }
}

/// What a workbench repair takes (design 11): half the head materials of the
/// tool's recipe, rounded up. The head is every input that is not a handle
/// good (`wear.handle_goods`). A recipe with no head (the bow's planks and
/// rope) asks half of every input, so no repair is free ("Weapon repair").
pub fn repair_inputs(cfg: &SurvivalConfig, recipe_inputs: &[(String, u16)]) -> Vec<(String, u16)> {
    let half = |list: &mut dyn Iterator<Item = &(String, u16)>| -> Vec<(String, u16)> {
        list.map(|(good, n)| (good.clone(), n.div_ceil(2)))
            .filter(|(_, n)| *n > 0)
            .collect()
    };
    let head = half(
        &mut recipe_inputs
            .iter()
            .filter(|(good, _)| !cfg.wear.handle_goods.contains(good)),
    );
    if head.is_empty() {
        half(&mut recipe_inputs.iter())
    } else {
        head
    }
}

/// What a town's smith or trader asks (design 11): the tool's value in
/// coins times the share worn, rounded up; nothing for an unworn tool.
pub fn repair_coins(cfg: &SurvivalConfig, value_coins: u32, wear: Wear, wearer: &Wearer) -> u32 {
    let worn = 1.0 - wear.share(cfg, wearer).clamp(0.0, 1.0);
    (value_coins as f32 * worn).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const PICK: Wearer = Wearer::Tool(Tool::Pickaxe, Tier::Stone);

    #[test]
    fn each_action_spends_one_and_a_harder_block_two() {
        let cfg = SurvivalConfig::shipped();
        for action in [
            Action::BlockBroken { block_tier: None },
            Action::BlockBroken {
                block_tier: Some(0),
            },
            Action::TreeFelled,
            Action::Tilled,
            Action::FishLanded,
            Action::HitLanded,
            Action::Used,
        ] {
            assert_eq!(uses_spent(&cfg, action, Tier::Stone), 1, "{action:?}");
        }
        let harder = Action::BlockBroken {
            block_tier: Some(1),
        };
        assert_eq!(uses_spent(&cfg, harder, Tier::Stone), 2);
        assert_eq!(uses_spent(&cfg, harder, Tier::Bronze), 1);
    }

    #[test]
    fn a_missed_swing_costs_nothing() {
        let cfg = SurvivalConfig::shipped();
        let mut w = Wear::full(&cfg, &PICK);
        for action in [Action::Missed, Action::DigAbandoned, Action::EmptyCast] {
            assert_eq!(
                w.spend(&cfg, WorldMode::Survival, &PICK, action),
                Spend::None
            );
        }
        assert_eq!(w.uses_left, 60);
    }

    #[test]
    fn a_tool_breaks_and_is_repaired() {
        let cfg = SurvivalConfig::shipped();
        let mut w = Wear::full(&cfg, &PICK);
        let stone = Action::BlockBroken {
            block_tier: Some(0),
        };
        for _ in 0..59 {
            assert_eq!(
                w.spend(&cfg, WorldMode::Survival, &PICK, stone),
                Spend::Spent(1)
            );
        }
        assert_eq!(w.bar(&cfg, &PICK), Some((1.0 / 60.0, true)));
        assert_eq!(
            w.spend(&cfg, WorldMode::Survival, &PICK, stone),
            Spend::Broke
        );
        assert!(w.is_broken());
        assert_eq!(
            w.spend(&cfg, WorldMode::Survival, &PICK, stone),
            Spend::None
        );
        // Saved and reopened, still broken.
        let back: Wear = ron::from_str(&ron::to_string(&w).unwrap()).unwrap();
        assert!(back.is_broken());
        // At the workbench: half the head, rounded up.
        let recipe = vec![
            ("stone".to_string(), 3),
            ("planks".to_string(), 2),
            ("rope".to_string(), 1),
        ];
        assert_eq!(repair_inputs(&cfg, &recipe), vec![("stone".to_string(), 2)]);
        w.repair(&cfg, &PICK);
        assert_eq!(w.uses_left, 60);
        assert_eq!(w.bar(&cfg, &PICK), None);
    }

    #[test]
    fn a_recipe_with_no_head_asks_half_of_everything() {
        let cfg = SurvivalConfig::shipped();
        let bow = [("planks".to_string(), 2), ("rope".to_string(), 2)];
        assert_eq!(
            repair_inputs(&cfg, &bow),
            vec![("planks".to_string(), 1), ("rope".to_string(), 1)]
        );
        let sword = [
            ("bronze-ingot".to_string(), 3),
            ("planks".to_string(), 1),
            ("leather".to_string(), 1),
        ];
        assert_eq!(
            repair_inputs(&cfg, &sword),
            vec![("bronze-ingot".to_string(), 2)]
        );
    }

    #[test]
    fn a_smith_asks_the_share_worn() {
        let cfg = SurvivalConfig::shipped();
        let half = Wear { uses_left: 30 };
        assert_eq!(repair_coins(&cfg, 9, half, &PICK), 5);
        assert_eq!(repair_coins(&cfg, 9, Wear::full(&cfg, &PICK), &PICK), 0);
    }

    #[test]
    fn explorer_never_wears() {
        let cfg = SurvivalConfig::shipped();
        let mut w = Wear::full(&cfg, &PICK);
        for _ in 0..1000 {
            let a = Action::BlockBroken {
                block_tier: Some(1),
            };
            assert_eq!(w.spend(&cfg, WorldMode::Explorer, &PICK, a), Spend::None);
        }
        assert_eq!(w.uses_left, 60);
    }

    #[test]
    fn uses_by_tier_and_an_old_save_reads_full() {
        let cfg = SurvivalConfig::shipped();
        let tier = |t| Wearer::Tool(Tool::Axe, t).max_uses(&cfg);
        assert_eq!(
            [
                tier(Tier::Stone),
                tier(Tier::Bronze),
                tier(Tier::Iron),
                tier(Tier::Steel)
            ],
            [60, 160, 320, 640]
        );
        assert_eq!(Wearer::Weapon("bronze-sword".into()).max_uses(&cfg), 160);
        assert_eq!(Wearer::Straw.max_uses(&cfg), 30);
        assert_eq!(Wear::of_saved(&cfg, &PICK, None).uses_left, 60);
    }
}
