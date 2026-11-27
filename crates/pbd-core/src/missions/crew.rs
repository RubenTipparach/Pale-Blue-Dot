//! Crew missions (`coop-missions`): goals a world's players meet together,
//! kept once a world and counted with each hand's share.
//!
//! - **The crew is the world's players** (design 1): progress is the world
//!   record `crew_missions`, never a copy in each player's own record. A
//!   player who joins halfway adds to it as it stands.
//! - **One counting rule** (design 2): a share grows by what
//!   [`super::count`] says the thing done adds, the same function a
//!   player's own tree counts with, given the objective's total so far.
//! - **Two fields make an objective a crew's** (design 3): `hands: n` is
//!   met only once `n` different players have counted toward it, and
//!   `each: n` only once every player present past [`CrewFile::grace_s`]
//!   has counted `n`. `hands` asks no more hands than the crew has ever had,
//!   so a world played alone can still finish a crew chain, more slowly
//!   (design 4's "a crew chain can be finished alone").
//! - **The step for two** (`Together`, design 4) is not built: it waits for
//!   the server's held transaction and the beam's request.

use super::{
    Catalog, Chain, Happened, Kind, Mission, MissionEntry, MissionFile, ObjectiveEntry, Standing,
    Step, standing_in,
};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The record kind crew progress is stored as, its schema and its one id
/// (the world's).
pub const CREW_MISSIONS_KIND: &str = "crew_missions";
pub const CREW_MISSIONS_SCHEMA: u32 = 1;
pub const CREW_MISSIONS_ID: u64 = 0;

/// The key prefix of the crew missions' words in the locale file:
/// `crew_mission.<id>.title`, `.help` and `.objective.<n>`.
pub const CREW_KEYS: &str = "crew_mission";

/// One crew objective as the file writes it: a player objective's fields
/// and the two crew fields.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrewObjectiveEntry {
    pub kind: Kind,
    pub what: String,
    pub count: u32,
    /// Met only once this many different players have counted toward it.
    #[serde(default, skip_serializing_if = "is_nought")]
    pub hands: u32,
    /// Met only once every player present past the grace time has counted
    /// at least this many.
    #[serde(default, skip_serializing_if = "is_nought")]
    pub each: u32,
}

fn is_nought(n: &u32) -> bool {
    *n == 0
}

/// One crew mission as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrewMissionEntry {
    /// Saved in the record: append-only, never renamed once shipped.
    pub id: String,
    #[serde(default)]
    pub needs: Vec<String>,
    pub objectives: Vec<CrewObjectiveEntry>,
    /// Its icon, a path under `assets/items/` without `.png`.
    #[serde(default)]
    pub icon: String,
}

/// The `crew` section of `missions.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrewFile {
    /// How long a player who has just joined is exempt from `each`, seconds
    /// of world time (design 3: 300).
    pub grace_s: f64,
    pub missions: Vec<CrewMissionEntry>,
}

/// What a crew objective asks beyond its count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrewRule {
    pub hands: u32,
    pub each: u32,
}

/// The crew chain, resolved: the missions as a player tree resolves them,
/// each objective's crew rule beside it, and the grace time.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewChain {
    pub chain: Chain,
    /// By mission id, one rule an objective, in the objectives' order.
    pub rules: BTreeMap<String, Vec<CrewRule>>,
    pub grace_s: f64,
}

impl CrewFile {
    /// Resolve the crew chain against what the build knows, by the same
    /// rules as a player's tree ([`MissionFile::resolve_keyed`]), its words
    /// under [`CREW_KEYS`]. The grace time is checked: finite and not
    /// below nought.
    pub fn resolve(&self, catalog: &Catalog, awaited: &[String]) -> Result<CrewChain, String> {
        if !self.grace_s.is_finite() || self.grace_s < 0.0 {
            return Err(format!("crew: grace_s {} is not a time", self.grace_s));
        }
        let file = MissionFile {
            awaited: awaited.to_vec(),
            missions: self
                .missions
                .iter()
                .map(|m| MissionEntry {
                    id: m.id.clone(),
                    title: None,
                    help: None,
                    needs: m.needs.clone(),
                    objectives: m
                        .objectives
                        .iter()
                        .map(|o| ObjectiveEntry {
                            kind: o.kind,
                            what: o.what.clone(),
                            count: o.count,
                            label: None,
                        })
                        .collect(),
                    unlocks: Vec::new(),
                    after: Vec::new(),
                    branch: "crew".into(),
                    icon: m.icon.clone(),
                    coins: 0,
                    mode: None,
                })
                .collect(),
            crew: None,
        };
        let chain = file.resolve_keyed(catalog, CREW_KEYS)?;
        let mut rules = BTreeMap::new();
        for m in &self.missions {
            for o in &m.objectives {
                if o.each > o.count {
                    return Err(format!("{}: each {} over its count", m.id, o.each));
                }
            }
            let row = m
                .objectives
                .iter()
                .map(|o| CrewRule {
                    hands: o.hands,
                    each: o.each,
                })
                .collect();
            rules.insert(m.id.clone(), row);
        }
        Ok(CrewChain {
            chain,
            rules,
            grace_s: self.grace_s,
        })
    }
}

/// A player in the world now, and the world time they joined at, s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hand {
    pub player: u32,
    pub since_s: f64,
}

/// Why a crew objective whose count is reached is not met yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Short {
    /// More different players must count toward it: this many more.
    Hands(u32),
    /// These players present past the grace time have not counted their
    /// share.
    Each(Vec<u32>),
}

impl Short {
    /// The panel's line for it.
    pub fn text(&self) -> crate::locale::Localized {
        match self {
            Short::Hands(_) => crate::locale::tr("crew.needs_hands"),
            Short::Each(_) => crate::locale::tr("crew.needs_each"),
        }
    }
}

/// The `crew_missions` record's body: what the crew has done, and every
/// open mission's shares.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CrewProgress {
    pub done: BTreeSet<String>,
    /// Each mission's shares, kept once it is done: one map an objective,
    /// in the chain's order, from player to what they counted. An open
    /// objective's total is the sum.
    pub shares: BTreeMap<String, Vec<BTreeMap<u32, u32>>>,
    /// The names a once-a-name objective has counted, by mission.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub seen: BTreeMap<String, BTreeSet<String>>,
    /// Every player who has been in the crew: present when it counted.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub members: BTreeSet<u32>,
}

impl CrewProgress {
    /// The crew's progress in `records`: none kept is a crew with nothing
    /// done, which is every world from before crew missions. `None` when
    /// the record is of a schema this build cannot read, so nothing writes
    /// over it.
    pub fn from_records(records: &Records) -> Option<Self> {
        match records.get(CREW_MISSIONS_KIND, CREW_MISSIONS_ID) {
            None => Some(Self::default()),
            Some(r) if r.schema == CREW_MISSIONS_SCHEMA => r.read(),
            Some(_) => None,
        }
    }

    /// The record that stores it.
    pub fn record(&self) -> Record {
        Record::of(
            CREW_MISSIONS_KIND,
            CREW_MISSIONS_ID,
            CREW_MISSIONS_SCHEMA,
            self,
        )
    }

    pub fn is_done(&self, id: &str) -> bool {
        self.done.contains(id)
    }

    /// Where `mission` stands, by the player trees' rule.
    pub fn standing(&self, chain: &Chain, mission: &Mission) -> Standing {
        standing_in(&self.done, chain, mission)
    }

    /// Each share of `mission`'s objective `index`: player to count.
    pub fn shares(&self, mission: &Mission, index: usize) -> BTreeMap<u32, u32> {
        self.shares
            .get(&mission.id)
            .and_then(|s| s.get(index))
            .cloned()
            .unwrap_or_default()
    }

    /// The total of `mission`'s objective `index`: its count once done.
    pub fn total(&self, mission: &Mission, index: usize) -> u32 {
        if self.is_done(&mission.id) {
            return mission.objectives.get(index).map_or(0, |o| o.count);
        }
        self.shares(mission, index).values().sum()
    }

    /// Why `mission`'s objective `index` is not met though its count is
    /// reached, with `present` in the world at `now_s`; `None` when it is
    /// met or its count is not reached.
    pub fn short(
        &self,
        crew: &CrewChain,
        mission: &Mission,
        index: usize,
        present: &[Hand],
        now_s: f64,
    ) -> Option<Short> {
        let objective = mission.objectives.get(index)?;
        if self.is_done(&mission.id) || self.total(mission, index) < objective.count {
            return None;
        }
        let rule = crew
            .rules
            .get(&mission.id)
            .and_then(|r| r.get(index))
            .copied()
            .unwrap_or_default();
        let shares = self.shares(mission, index);
        // No more hands than the crew has had: a world played alone is a
        // crew of one.
        let crew_size = self.members.len().max(1) as u32;
        let hands = shares.values().filter(|n| **n > 0).count() as u32;
        let wanted = rule.hands.min(crew_size);
        if hands < wanted {
            return Some(Short::Hands(wanted - hands));
        }
        if rule.each > 0 {
            let missing: Vec<u32> = present
                .iter()
                .filter(|h| now_s - h.since_s >= crew.grace_s)
                .filter(|h| shares.get(&h.player).copied().unwrap_or(0) < rule.each)
                .map(|h| h.player)
                .collect();
            if !missing.is_empty() {
                return Some(Short::Each(missing));
            }
        }
        None
    }

    /// Whether every objective of `mission` is met.
    pub fn met(&self, crew: &CrewChain, mission: &Mission, present: &[Hand], now_s: f64) -> bool {
        mission.objectives.iter().enumerate().all(|(i, o)| {
            self.total(mission, i) >= o.count
                && self.short(crew, mission, i, present, now_s).is_none()
        })
    }

    /// Count one thing `author` did toward every open crew mission, with
    /// `present` in the world at `now_s`, then close what that met.
    pub fn count(
        &mut self,
        crew: &CrewChain,
        author: u32,
        happened: &Happened,
        present: &[Hand],
        now_s: f64,
    ) -> Step {
        let mut changed = self.join(author, present);
        let chain = &crew.chain;
        let open: Vec<Mission> = chain
            .missions
            .iter()
            .filter(|m| self.standing(chain, m) == Standing::Open)
            .cloned()
            .collect();
        for mission in &open {
            let mut row = self.shares.get(&mission.id).cloned().unwrap_or_default();
            row.resize(mission.objectives.len(), BTreeMap::new());
            let rules = crew.rules.get(&mission.id);
            let mut named = false;
            for (i, (shares, objective)) in row.iter_mut().zip(&mission.objectives).enumerate() {
                let rule = rules.and_then(|r| r.get(i)).copied().unwrap_or_default();
                let total: u32 = shares.values().sum();
                let had = self.had(shares, total, objective.count, rule, author);
                let added = super::count(objective, happened, had, self.seen.get(&mission.id));
                if added > 0 {
                    *shares.entry(author).or_insert(0) += added;
                    named |= happened.kind.once_a_name();
                    changed = true;
                }
            }
            if named {
                self.seen
                    .entry(mission.id.clone())
                    .or_default()
                    .insert(happened.what.clone());
            }
            if row.iter().any(|s| !s.is_empty()) {
                self.shares.insert(mission.id.clone(), row);
            }
        }
        let mut step = self.settle(crew, present, now_s);
        step.changed |= changed;
        step
    }

    /// Close every open crew mission that is met, until nothing more
    /// closes: after every count, and when the players present change (a
    /// late joiner's grace running out can meet an `each`).
    pub fn settle(&mut self, crew: &CrewChain, present: &[Hand], now_s: f64) -> Step {
        let mut step = Step::default();
        for hand in present {
            step.changed |= self.members.insert(hand.player);
        }
        let chain = &crew.chain;
        loop {
            let closed = chain
                .missions
                .iter()
                .filter(|m| self.standing(chain, m) == Standing::Open)
                .find(|m| self.met(crew, m, present, now_s))
                .map(|m| m.id.clone());
            let Some(id) = closed else {
                break;
            };
            // Its shares are kept: who did what stays on the record.
            self.seen.remove(&id);
            self.done.insert(id.clone());
            step.done.push(id);
            step.changed = true;
        }
        step
    }

    /// What an objective is taken to have counted when `author` counts
    /// toward it: its total, or, once the total is reached, just short of
    /// its count by what `author` still owes it (a first share where more
    /// hands are wanted, the rest of an `each`), so the hands it waits for
    /// can still be counted.
    fn had(
        &self,
        shares: &BTreeMap<u32, u32>,
        total: u32,
        count: u32,
        rule: CrewRule,
        author: u32,
    ) -> u32 {
        if total < count {
            return total;
        }
        let mine = shares.get(&author).copied().unwrap_or(0);
        let crew_size = self.members.len().max(1) as u32;
        let hands = shares.values().filter(|n| **n > 0).count() as u32;
        let owed_hand = u32::from(mine == 0 && hands < rule.hands.min(crew_size));
        let owed = owed_hand.max(rule.each.saturating_sub(mine));
        count - owed.min(count)
    }

    /// Note `author` and everyone present as members. Whether any is new.
    fn join(&mut self, author: u32, present: &[Hand]) -> bool {
        let mut changed = self.members.insert(author);
        for hand in present {
            changed |= self.members.insert(hand.player);
        }
        changed
    }

    /// Every player who counted toward any of `missions`, done or open:
    /// who built the house, for its co-ownership (design 5).
    pub fn hands_of(&self, missions: &[&str]) -> BTreeSet<u32> {
        missions
            .iter()
            .filter_map(|id| self.shares.get(*id))
            .flatten()
            .flat_map(|s| s.iter().filter(|(_, n)| **n > 0).map(|(p, _)| *p))
            .collect()
    }
}

/// The house chain's building missions (design 5), whose hands co-own the
/// house when its hearth is lit.
pub const HOUSE_BUILDERS: [&str; 4] = ["house-floor", "house-walls", "house-roof", "house-hearth"];

/// The records that make every piece of `ids` co-owned by `owners`
/// (design 5): each piece record names them, beside any it already named.
/// Pieces not in `records`, or taken away, are left alone.
pub fn co_own(records: &Records, ids: &[u64], owners: &BTreeSet<u32>) -> Vec<Record> {
    use crate::building::{PIECE_RECORD, PIECE_SCHEMA, PieceBody};
    ids.iter()
        .filter_map(|&id| {
            let record = records.get(PIECE_RECORD, id)?;
            if record.schema != PIECE_SCHEMA {
                return None;
            }
            let mut body: PieceBody = record.read()?;
            body.piece.as_ref()?;
            let before = body.owners.len();
            body.owners.extend(owners.iter().copied());
            body.owners.sort_unstable();
            body.owners.dedup();
            (body.owners.len() != before).then(|| Record::of(PIECE_RECORD, id, PIECE_SCHEMA, &body))
        })
        .collect()
}

#[cfg(test)]
mod tests;
