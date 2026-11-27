//! Survival, run by the server (`survival-mode` design 10, task 4.1): each
//! player in a Survival world has [`Vitals`], ticked here by the core's own
//! rule ([`Vitals::tick`]) with what the player's client reports of where
//! they are ([`Conditions`]). A client may tick a copy to predict; these
//! numbers win.
//!
//! The vitals are a world record of kind [`VITALS_KIND`], one per player
//! (its id the player's number), journaled through [`Store::commit_record`]:
//! at once for anything that happens to the player (a hit, a meal, a sip, a
//! death, a wake), and every [`VITALS_EVERY_S`] of play while they only
//! tick, so a long walk is never more than that from its bars. A world from
//! before survival, or an Explorer one, has no record and no bars.

use super::{Server, Store};
use pbd_core::records::{Author, Record, Records};
use pbd_core::survival::body::Eaten;
use pbd_core::survival::health::Cause;
use pbd_core::survival::stamina::Burst;
use pbd_core::survival::water::Sip;
use pbd_core::survival::{Conditions, Event, SurvivalConfig, Tick, Vitals, WorldMode};

/// The record kind a player's vitals are kept under.
pub const VITALS_KIND: &str = "vitals";
/// Its schema.
pub const VITALS_SCHEMA: u32 = 1;
/// Seconds of play between the records a ticking player's vitals are
/// journaled at, when nothing else has happened to them.
pub const VITALS_EVERY_S: f32 = 5.0;

/// Something that happens to a player outside the tick.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// A fall's or a creature's damage.
    Hit(Cause, f32),
    /// A good eaten, at its stars.
    Eat(String, u8),
    /// A sip, as the core's [`pbd_core::survival::water::sip`] gave it.
    Drink(Sip),
    /// A jump, a climb, a throw.
    Burst(Burst),
    /// The player pressed Wake up after a death.
    WakeUp,
    /// Another player revived them.
    Revive,
}

/// What an [`Act`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Acted {
    pub events: Vec<Event>,
    /// A meal's food and water, for [`Act::Eat`].
    pub eaten: Option<Eaten>,
}

/// A player's vitals as a world's records hold them: the saved bars in a
/// Survival world (full where it has none yet), none in Explorer. An old
/// save reads Explorer (`WorldMode::of_saved`) and so has none.
pub fn vitals_of(
    cfg: &SurvivalConfig,
    mode: WorldMode,
    records: &Records,
    player: u32,
) -> Option<Vitals> {
    if !mode.is_survival() {
        return None;
    }
    Some(
        records
            .get(VITALS_KIND, u64::from(player))
            .and_then(Record::read::<Vitals>)
            .unwrap_or_else(|| Vitals::new(cfg)),
    )
}

impl Server {
    /// Give `player` these vitals (`None`: the world is Explorer's, or the
    /// player has none). Their record is the caller's to read
    /// ([`vitals_of`]).
    pub fn set_vitals(&mut self, player: u32, vitals: Option<Vitals>) {
        if let Some(p) = self.players.get_mut(&player) {
            p.vitals = vitals;
            p.since_vitals_s = 0.0;
        }
    }

    /// A player's vitals, as the server holds them.
    pub fn vitals(&self, player: u32) -> Option<&Vitals> {
        self.players.get(&player)?.vitals.as_ref()
    }

    /// Tick `player`'s vitals by `dt_s` of play in `conditions`. `None` for
    /// a player with no vitals (Explorer). The record is journaled when the
    /// tick raised anything, and every [`VITALS_EVERY_S`] otherwise.
    pub fn survive(
        &mut self,
        store: &mut impl Store,
        cfg: &SurvivalConfig,
        player: u32,
        conditions: &Conditions,
        dt_s: f32,
    ) -> Option<Tick> {
        let p = self.players.get_mut(&player)?;
        let vitals = p.vitals.as_mut()?;
        let tick = vitals.tick(cfg, conditions, dt_s);
        p.since_vitals_s += dt_s.max(0.0);
        // A note ("Thirsty") is shown, not kept: it alone does not write.
        let happened = tick
            .events
            .iter()
            .any(|e| !matches!(e, Event::Note(_) | Event::HeatWarning(_)));
        if happened || p.since_vitals_s >= VITALS_EVERY_S {
            self.commit_vitals(store, player);
        }
        Some(tick)
    }

    /// Something happened to `player`: it lands on their vitals and the
    /// record is journaled at once. `None` for a player with no vitals.
    pub fn survival_act(
        &mut self,
        store: &mut impl Store,
        cfg: &SurvivalConfig,
        player: u32,
        act: Act,
    ) -> Option<Acted> {
        let vitals = self.players.get_mut(&player)?.vitals.as_mut()?;
        let mut acted = Acted::default();
        match act {
            Act::Hit(cause, pts) => acted.events = vitals.hit(cfg, cause, pts),
            Act::Eat(good, stars) => acted.eaten = Some(vitals.eat(cfg, &good, stars)),
            Act::Drink(sip) => vitals.drink(cfg, &sip),
            Act::Burst(burst) => acted.events.extend(vitals.burst(cfg, burst)),
            Act::WakeUp => vitals.wake_up(cfg),
            Act::Revive => {
                vitals.revive(cfg);
            }
        }
        self.commit_vitals(store, player);
        Some(acted)
    }

    /// Journal `player`'s vitals now. `false` when the store refused it (the
    /// bars stay as they are, and the next tick tries again).
    pub fn commit_vitals(&mut self, store: &mut impl Store, player: u32) -> bool {
        let Some(p) = self.players.get_mut(&player) else {
            return false;
        };
        let Some(vitals) = p.vitals.as_ref() else {
            return false;
        };
        let record = Record::of(VITALS_KIND, u64::from(player), VITALS_SCHEMA, vitals);
        let kept = store.commit_record(&Author::Player(player), record);
        if kept {
            p.since_vitals_s = 0.0;
        }
        kept
    }
}
