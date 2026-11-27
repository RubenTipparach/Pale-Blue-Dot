//! Cooking's shared state for the systems it touches (`cooking` design S5
//! and S6): the events the starter missions count, and the effects of what
//! was eaten, read by walking, fishing and digging.
//!
//! In the library rather than the desktop binary because walking and
//! fishing live here; the panel, eating and the HUD are `desktop/kitchen.rs`.

use crate::config::KitchenConfig;
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::cooking::{Effect, MEAL_ID, MEAL_KIND, Meals};
use pbd_core::goods::Good;
use pbd_core::inventory::Item;

/// A dish was cooked and collected (`starter-missions`: "cook x fish").
/// Raised on the frame the durable write was accepted.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct Cooked {
    /// The recipe's code.
    pub recipe: String,
    pub dish: Item,
    pub count: u16,
    /// How many caught fish went into it.
    pub fish: u16,
}

/// A dish was eaten. Raised on the frame the durable write was accepted.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ate {
    pub dish: Good,
    pub effect: Effect,
}

/// Raw food was eaten from the hand (Survival only: raw meat, an egg, milk,
/// a tree fruit), with no effect. Raised on the frame the save took it, so
/// it munches as a dish does (`audio-and-ambience`, "Munching, take 2").
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Munched {
    pub good: Good,
}

/// What the running effects do this frame: each a multiplier, 1 when it is
/// not running. Kept in step with the save's `meal` record by
/// [`refresh_effects`]; walking, fishing and digging read it.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct MealEffects {
    pub stride: f32,
    pub steady: f32,
    pub keen: f32,
}

impl Default for MealEffects {
    fn default() -> Self {
        Self {
            stride: 1.0,
            steady: 1.0,
            keen: 1.0,
        }
    }
}

/// The running effects, as the save holds them; none when it holds none.
pub fn meals(save: &WorldSave) -> Meals {
    save.records
        .get(MEAL_KIND, MEAL_ID)
        .and_then(|r| r.read::<Meals>())
        .unwrap_or_default()
}

/// Work out this frame's multipliers from the save and the world clock.
pub fn refresh_effects(
    save: Option<Res<WorldSave>>,
    sun: Option<Res<crate::sky::Sun>>,
    kitchen: Res<KitchenConfig>,
    mut effects: ResMut<MealEffects>,
) {
    let (Some(save), Some(sun)) = (save, sun) else {
        return;
    };
    let now_s = sun.clock.seconds;
    let running = meals(&save);
    let s = &kitchen.0.strengths;
    let next = MealEffects {
        stride: running.factor(Effect::Stride, now_s, s),
        steady: running.factor(Effect::Steady, now_s, s),
        keen: running.factor(Effect::Keen, now_s, s),
    };
    if *effects != next {
        *effects = next;
    }
}
