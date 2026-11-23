//! The stored records (decision 10; S5, S7): a kept animal and a fence
//! piece, each one line of RON on the journal (`crate::records`).
//!
//! Every field but the species has a default, so a body written by an
//! earlier schema reads under a later build with the fields it lacks at
//! their defaults, and the genome pads the genes it lacks
//! ([`Genome::from_bytes`]).

use super::farm::FarmState;
use super::{Genome, Sex};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A kept animal's record kind and the schema this build writes.
pub const ANIMAL: &str = "animal";
pub const ANIMAL_SCHEMA: u32 = 1;
/// A fence piece's.
pub const FENCE: &str = "fence";
pub const FENCE_SCHEMA: u32 = 1;

/// How an animal stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Life {
    #[default]
    Alive,
    /// Ended (butchered, later taken): kept so pedigrees never break.
    Ended,
}

/// How it came to be kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Via {
    /// Tamed by feeding.
    #[default]
    Fed,
    /// Born on the farm (task 8).
    Born,
    /// Bought from a town (`land-transport` design 10): the horse.
    Bought,
}

/// A kept animal: `rec @p animal <id> 1 (...)`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AnimalBody {
    /// The body whose land roster `sp` indexes.
    pub body: String,
    /// The species' index in that roster (append-only).
    pub sp: u16,
    pub sex: Sex,
    /// World day of birth.
    pub born: f64,
    /// The genome's bytes.
    #[serde(rename = "gen")]
    pub genes: Vec<u8>,
    pub sire: Option<u64>,
    pub dam: Option<u64>,
    pub name: Option<String>,
    /// The pen it lives in (its id, the pen's lowest cell key), if any.
    pub pen: Option<u32>,
    /// The cell it lives on: its pen's, or where it was let go.
    pub home: u32,
    /// The player who keeps it.
    pub owner: u32,
    pub state: Life,
    pub via: Via,
    /// The wild seed it was tamed from, so its herd leaves it out (S10).
    pub wild: u64,
    /// The farm state the player last wrote (S7).
    pub as_of: f64,
    pub health: f32,
    pub hungry: f32,
    pub fed_until: f64,
    pub made: BTreeMap<String, f32>,
    /// The brood it was born of (design 13, E5): a brood some animal names
    /// is done. `None` for a tamed animal and every record written before.
    pub brood: Option<u64>,
    /// The sire the player chose for her on the breeding screen (E6).
    pub mate: Option<u64>,
}

impl Default for AnimalBody {
    fn default() -> Self {
        Self {
            body: crate::fauna::HOME_BODY.into(),
            sp: 0,
            sex: Sex::F,
            born: 0.0,
            genes: Genome::default().0.to_vec(),
            sire: None,
            dam: None,
            name: None,
            pen: None,
            home: 0,
            owner: 0,
            state: Life::Alive,
            via: Via::Fed,
            wild: 0,
            as_of: 0.0,
            health: 100.0,
            hungry: 0.0,
            fed_until: 0.0,
            made: BTreeMap::new(),
            brood: None,
            mate: None,
        }
    }
}

impl AnimalBody {
    pub fn genome(&self) -> Genome {
        Genome::from_bytes(&self.genes)
    }

    pub fn farm(&self) -> FarmState {
        FarmState {
            as_of: self.as_of,
            health: self.health,
            hungry: self.hungry,
            fed_until: self.fed_until,
            made: self.made.clone(),
        }
    }

    /// Re-base on a farm state the player's action produced.
    pub fn set_farm(&mut self, s: &FarmState) {
        self.as_of = s.as_of;
        self.health = s.health;
        self.hungry = s.hungry;
        self.fed_until = s.fed_until;
        self.made = s.made.clone();
    }

    pub fn record(&self, id: u64) -> Record {
        Record::of(ANIMAL, id, ANIMAL_SCHEMA, self)
    }
}

/// Where a fence piece stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Standing {
    #[default]
    Placed,
    /// Taken back by the player (records are never deleted).
    Removed,
}

/// A fence or gate piece: `rec @p fence <edge> 1 (...)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct FenceBody {
    pub gate: bool,
    pub state: Standing,
}

impl FenceBody {
    pub fn record(&self, edge: super::Edge) -> Record {
        Record::of(FENCE, edge.0, FENCE_SCHEMA, self)
    }
}

/// Every kept animal in a world, by id, that this build can read.
pub fn animals(records: &Records) -> Vec<(u64, AnimalBody)> {
    records
        .of_kind(ANIMAL)
        .filter_map(|r| r.read::<AnimalBody>().map(|b| (r.id, b)))
        .collect()
}

/// The id the next kept animal takes: one past the highest ever stored.
/// Records are never deleted, so an id is never reused.
pub fn next_animal_id(records: &Records) -> u64 {
    records.of_kind(ANIMAL).map(|r| r.id + 1).max().unwrap_or(1)
}

/// Every standing piece: edge to whether it is a gate.
pub fn pieces(records: &Records) -> BTreeMap<super::Edge, bool> {
    records
        .of_kind(FENCE)
        .filter_map(|r| {
            let body = r.read::<FenceBody>()?;
            (body.state == Standing::Placed).then_some((super::Edge(r.id), body.gate))
        })
        .collect()
}
