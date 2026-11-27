//! Torches and the rain (design 9 and 13): a torch under the open sky in
//! rain goes out, held after `held_out_s` and placed after `placed_out_s`;
//! a roof keeps it lit; snow does not put it out. A placed torch going out
//! leaves the player's block as it is: it is a `lamp_out` record of the
//! world's, and lighting it again a `lamp_lit` record of the player's. Each
//! counts, and a torch is out while the world's count is ahead, so each
//! author writes only its own record and the rain can put a torch out again
//! after the player has relit it.

use super::config::FireConfig;
use crate::records::{Author, Record, Records};
use serde::{Deserialize, Serialize};

pub const OUT_KIND: &str = "lamp_out";
pub const LIT_KIND: &str = "lamp_lit";
pub const SCHEMA: u32 = 1;
pub const KINDS: [(&str, u32); 2] = [(OUT_KIND, SCHEMA), (LIT_KIND, SCHEMA)];

/// A torch's place: its cell and layer.
pub fn lamp_id(cell: u32, layer: u16) -> u64 {
    (u64::from(cell) << 16) | u64::from(layer)
}

pub fn lamp_of(id: u64) -> (u32, u16) {
    ((id >> 16) as u32, id as u16)
}

/// How many times it went out, or was lit again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Count {
    pub n: u32,
}

fn count(records: &Records, kind: &str, id: u64) -> u32 {
    records
        .get(kind, id)
        .and_then(|r| r.read::<Count>())
        .map_or(0, |c| c.n)
}

/// Whether the placed torch at `cell`, `layer` is out.
pub fn is_out(records: &Records, cell: u32, layer: u16) -> bool {
    let id = lamp_id(cell, layer);
    count(records, OUT_KIND, id) > count(records, LIT_KIND, id)
}

/// Every placed torch that is out, as `(cell, layer)`, in key order.
pub fn out_lamps(records: &Records) -> Vec<(u32, u16)> {
    records
        .of_kind(OUT_KIND)
        .map(|r| lamp_of(r.id))
        .filter(|&(c, l)| is_out(records, c, l))
        .collect()
}

/// The world's record putting it out, by [`author`].
pub fn out_record(records: &Records, cell: u32, layer: u16) -> Record {
    let id = lamp_id(cell, layer);
    let n = count(records, LIT_KIND, id).max(count(records, OUT_KIND, id)) + 1;
    Record::of(OUT_KIND, id, SCHEMA, &Count { n })
}

/// The player's record lighting it again.
pub fn lit_record(records: &Records, cell: u32, layer: u16) -> Record {
    let id = lamp_id(cell, layer);
    Record::of(
        LIT_KIND,
        id,
        SCHEMA,
        &Count {
            n: count(records, OUT_KIND, id),
        },
    )
}

/// The process's author for the torch at `cell`.
pub fn author(cell: u32) -> Author {
    Author::World {
        process: super::blaze::PROCESS.into(),
        kind: "lamp".into(),
        id: u64::from(cell),
    }
}

/// Where a torch stands, for the rain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exposure {
    /// The rain a person sees there, 0..1 (`Sample::rain_seen`).
    pub rain_seen: f32,
    /// Whether what falls is snow.
    pub snow: bool,
    /// No solid cell within `sky_layers` above it.
    pub open_sky: bool,
}

/// How long a torch has stood in the rain.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Soak {
    pub wet_s: f32,
}

impl Soak {
    /// A step of `dt` seconds; true on the step it goes out. Out of the
    /// rain, or under a roof, the count starts again.
    pub fn step(&mut self, cfg: &FireConfig, at: Exposure, held: bool, dt: f32) -> bool {
        let t = &cfg.torch;
        if at.open_sky && !at.snow && at.rain_seen >= t.rain_seen {
            let limit = if held { t.held_out_s } else { t.placed_out_s };
            let before = self.wet_s;
            self.wet_s += dt.max(0.0);
            before < limit && self.wet_s >= limit
        } else {
            self.wet_s = 0.0;
            false
        }
    }
}

/// Whether the column above a torch is open: no solid cell in the
/// `sky_layers` above `layer`. `solid(layer)` answers for the column.
pub fn open_sky(cfg: &FireConfig, layer: u16, solid: impl Fn(u32) -> bool) -> bool {
    (1..=cfg.torch.sky_layers).all(|k| !solid(u32::from(layer) + k))
}
