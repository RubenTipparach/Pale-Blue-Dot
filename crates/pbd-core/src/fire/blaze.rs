//! A forest fire (design 3, 5, 6 and 7): the `blaze` record, its one-second
//! spread in cell-key order with seeded draws, what starts one and what
//! stops it.
//!
//! Everything here is a pure function of the record, the tick, the world's
//! records and yield set, and what the [`Ground`] says of each cell, so the
//! same blaze burns the same cells on every machine and after a reload.
//! Nothing iterates a hash map: burning cells are kept sorted by key and
//! neighbours come in key order (`roads::neighbours`).

use super::config::FireConfig;
use super::dryness::{self, Cured, Factors, Plant};
use super::scar::{self, Burnt};
use crate::records::{Author, Proposal, Record, Records, YieldSet};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The record kinds and their schemas.
pub const BLAZE_KIND: &str = "blaze";
pub const BLAZE_SCHEMA: u32 = 1;
pub const BURNT_KIND: &str = "burnt";
pub const BURNT_SCHEMA: u32 = 1;
/// A player's act of lighting a blaze, id the blaze's.
pub const IGNITION_KIND: &str = "ignition";
pub const IGNITION_SCHEMA: u32 = 1;
/// A bucket poured on a cell, id the cell's key: the player's.
pub const DOUSED_KIND: &str = "doused";
pub const DOUSED_SCHEMA: u32 = 1;
/// The world process's name.
pub const PROCESS: &str = "fire";

/// The kinds this module writes, for the save's kind list.
pub const KINDS: [(&str, u32); 4] = [
    (BLAZE_KIND, BLAZE_SCHEMA),
    (BURNT_KIND, BURNT_SCHEMA),
    (IGNITION_KIND, IGNITION_SCHEMA),
    (DOUSED_KIND, DOUSED_SCHEMA),
];

/// One cell's facts, from the ground the app has loaded (design 13).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Land {
    /// What grows on it; a tree's cell is its tree.
    pub plant: Plant,
    /// Its top layer.
    pub top: i32,
    /// A water cell, or one water stands on.
    pub water: bool,
    /// A snow top.
    pub snow: bool,
    /// Inside a kept animals' pen.
    pub pen: bool,
}

/// The weather at a cell.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Air {
    pub ground_wet: f32,
    pub humidity: f32,
    pub air_c: f32,
    pub rain_mm_h: f32,
    /// Surface wind, m/s, tangent.
    pub wind: Vec3,
    /// Snow falls or lies here.
    pub snow: bool,
}

/// What the app knows of the world's cells. `land` is `None` for a cell it
/// cannot say (off the loaded ground): such a cell is a break.
pub trait Ground {
    fn land(&self, key: u32) -> Option<Land>;
    fn air(&self, key: u32) -> Air;
    /// The body's radius, m: the soil's patches are measured on it.
    fn radius_m(&self) -> f32;
    /// Where a cell is: a unit vector. The lattice's own by default.
    fn centre(&self, key: u32) -> Option<Vec3> {
        crate::roads::centre(key)
    }
    /// A cell's 5 or 6 neighbours, in key order.
    fn neighbours(&self, key: u32) -> Vec<u32> {
        crate::roads::neighbours(key)
    }
}

/// Who lit a blaze.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LitBy {
    Player(u32),
    Lightning,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    Burning,
    /// Out, at this world second.
    Out(f64),
}

/// A cell alight.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Burning {
    pub cell: u32,
    pub plant: Plant,
    /// Time left, tenths of a second.
    pub left_ds: u16,
    /// A conifer burning in its crown.
    pub crown: bool,
}

/// The blaze record's body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Blaze {
    pub started_s: f64,
    pub origin: u32,
    pub lit_by: LitBy,
    pub state: State,
    /// Ticks run.
    pub tick: u64,
    /// Sorted by cell key.
    pub burning: Vec<Burning>,
    /// Cells ever lit, burning and burnt.
    pub cells: u32,
    /// A limit stopped its spread.
    pub limited: bool,
}

/// The player's act of lighting one.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ignition {
    pub cell: u32,
    pub at_s: f64,
}

/// A bucket poured.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doused {
    pub at_s: f64,
}

impl Blaze {
    pub fn record(&self, id: u64) -> Record {
        Record::of(BLAZE_KIND, id, BLAZE_SCHEMA, self)
    }

    pub fn is_burning(&self) -> bool {
        self.state == State::Burning
    }

    pub fn burning_at(&self, cell: u32) -> Option<&Burning> {
        self.burning
            .binary_search_by_key(&cell, |b| b.cell)
            .ok()
            .map(|i| &self.burning[i])
    }
}

/// The process's author for blaze `id`.
pub fn author(id: u64) -> Author {
    Author::World {
        process: PROCESS.into(),
        kind: BLAZE_KIND.into(),
        id,
    }
}

/// Every blaze in the records, in id order.
pub fn blazes(records: &Records) -> Vec<(u64, Blaze)> {
    records
        .of_kind(BLAZE_KIND)
        .filter_map(|r| Some((r.id, r.read::<Blaze>()?)))
        .collect()
}

/// The blazes still burning.
pub fn live(records: &Records) -> Vec<(u64, Blaze)> {
    blazes(records)
        .into_iter()
        .filter(|(_, b)| b.is_burning())
        .collect()
}

/// A cell's scar, if it has one.
pub fn burnt(records: &Records, cell: u32) -> Option<Burnt> {
    records.get(BURNT_KIND, u64::from(cell))?.read()
}

/// Whether a cell had a bucket poured on it within `bucket_wet_days`.
pub fn doused(cfg: &FireConfig, records: &Records, cell: u32, now_s: f64) -> bool {
    records
        .get(DOUSED_KIND, u64::from(cell))
        .and_then(|r| r.read::<Doused>())
        .is_some_and(|d| {
            let days = (now_s - d.at_s) / f64::from(crate::daylight::DAY_S);
            (0.0..=cfg.bucket_wet_days).contains(&days)
        })
}

/// What a tick and an ignition read: the rules, the world and the moment.
pub struct World<'a, G: Ground> {
    pub cfg: &'a FireConfig,
    pub seed: u64,
    pub ground: &'a G,
    pub records: &'a Records,
    pub yields: &'a YieldSet,
    pub now_s: f64,
    /// The clock's `year_fraction` now.
    pub year_fraction: f64,
}

const SALT_PATCH: u32 = 0x60;

impl<G: Ground> World<'_, G> {
    /// A cell's [`Factors`], or `None` where the ground cannot say.
    pub fn factors(&self, cell: u32) -> Option<Factors> {
        let land = self.ground.land(cell)?;
        let d = &self.cfg.dryness;
        let air = self.ground.air(cell);
        let centre = self.ground.centre(cell)?;
        let cured: Cured = dryness::cured(self.cfg, self.year_fraction, centre.y);
        let weather = dryness::weather(self.cfg, air.ground_wet, air.humidity, air.air_c);
        let round = self.ground.neighbours(cell);
        let lands: Vec<Option<Land>> = round.iter().map(|&n| self.ground.land(n)).collect();
        let trees = lands
            .iter()
            .filter(|l| l.is_some_and(|l| l.plant.is_tree()))
            .count();
        let tops: Vec<f32> = lands.iter().flatten().map(|l| l.top as f32).collect();
        let rise = if tops.is_empty() {
            0.0
        } else {
            land.top as f32 - tops.iter().sum::<f32>() / tops.len() as f32
        };
        let n = crate::planet_gen::noise01(
            self.seed ^ 0x6669_7265,
            centre,
            self.ground.radius_m() / d.patch.field_m,
            2,
        );
        let roll = crate::tree_kit::roll(cell, SALT_PATCH);
        let near_pen = self.within(cell, d.grazed.cells, |l| l.pen);
        let to_water = self.rings_to(cell, (d.water.to_cells as u32).saturating_sub(1), |l| {
            l.water
        });
        Some(Factors {
            plant: land.plant,
            cured,
            weather,
            shade: dryness::shade(d, land.plant, trees, false),
            water: dryness::water(d, to_water),
            hollow: dryness::hollow(d, rise),
            patch: dryness::patch(d, n, roll),
            grazed: dryness::grazed(d, land.plant, near_pen),
            zeroed: land.snow
                || land.water
                || air.snow
                || self.scarred(cell)
                || doused(self.cfg, self.records, cell, self.now_s),
        })
    }

    /// `plant_dry` at a cell (0 where the ground cannot say).
    pub fn plant_dry(&self, cell: u32) -> f32 {
        self.factors(cell)
            .map_or(0.0, |f| dryness::plant_dry(self.cfg, &f))
    }

    /// A burnt cell whose fuel has not grown back (design 8).
    pub fn scarred(&self, cell: u32) -> bool {
        burnt(self.records, cell).is_some_and(|b| !scar::burns_again(self.cfg, &b, self.now_s))
    }

    /// Whether a cell within `rings` of `cell` (itself included) is `what`.
    fn within(&self, cell: u32, rings: u32, what: impl Fn(&Land) -> bool) -> bool {
        self.rings_to(cell, rings, &what).is_some()
            || self.ground.land(cell).is_some_and(|l| what(&l))
    }

    /// How many rings out the nearest cell that is `what` lies, 1 to
    /// `rings`; `None` if none within.
    fn rings_to(&self, cell: u32, rings: u32, what: impl Fn(&Land) -> bool) -> Option<u32> {
        let mut seen = BTreeSet::from([cell]);
        let mut ring = vec![cell];
        for r in 1..=rings {
            let mut next = Vec::new();
            for &c in &ring {
                for n in self.ground.neighbours(c) {
                    if seen.insert(n) {
                        next.push(n);
                    }
                }
            }
            if next
                .iter()
                .any(|&n| self.ground.land(n).is_some_and(|l| what(&l)))
            {
                return Some(r);
            }
            ring = next;
        }
        None
    }

    /// Whether any live blaze burns `cell`.
    fn burning_anywhere(&self, cell: u32) -> bool {
        live(self.records)
            .iter()
            .any(|(_, b)| b.burning_at(cell).is_some())
    }

    /// Whether `cell` may catch at all, whatever its dryness: something
    /// burns there, the player never touched it, and it is not alight.
    fn may_catch(&self, cell: u32) -> bool {
        !self.yields.has_cell(cell) && !self.burning_anywhere(cell)
    }

    fn row(&self, plant: Plant) -> Option<super::config::FuelRow> {
        let f = &self.cfg.fuel;
        Some(match plant {
            Plant::Grass => f.grass,
            Plant::Shrub => f.shrub,
            Plant::Deciduous => f.deciduous,
            Plant::Conifer => f.conifer,
            Plant::Evergreen => f.evergreen,
            Plant::None | Plant::Cactus => return None,
        })
    }

    fn burn_ds(&self, plant: Plant) -> u16 {
        self.row(plant)
            .map_or(0, |r| (r.burn_s * 10.0).round().clamp(1.0, 65_535.0) as u16)
    }

    /// The wind's term for a step from `from` to `to` (design 5).
    fn wind_term(&self, from: u32, to: u32, wind: Vec3) -> f32 {
        let w = &self.cfg.wind;
        let (Some(a), Some(b)) = (self.ground.centre(from), self.ground.centre(to)) else {
            return 1.0;
        };
        let step = (b - a).normalize_or_zero();
        let speed = wind.length();
        if speed <= 0.0 || step == Vec3::ZERO {
            return 1.0;
        }
        let cos = wind.normalize().dot(step);
        (1.0 + w.down_per_mps * speed * cos.max(0.0) - w.up_per_mps * speed * (-cos).max(0.0))
            .max(w.floor)
    }

    fn slope_term(&self, from: &Land, to: &Land) -> f32 {
        let s = &self.cfg.slope;
        match to.top - from.top {
            d if d > 0 => 1.0 + s.up_per_layer * d as f32,
            d if d < 0 => s.down,
            _ => 1.0,
        }
    }

    fn rain_term(&self, rain_mm_h: f32) -> f32 {
        let r = &self.cfg.rain;
        if rain_mm_h >= r.no_catch_mmh {
            0.0
        } else {
            (-rain_mm_h / r.spread_scale_mmh).exp()
        }
    }

    /// Whether a conifer catching at dryness `d` in this wind burns in its
    /// crown (design 3).
    fn crowns(&self, plant: Plant, d: f32, wind: Vec3) -> bool {
        plant == Plant::Conifer
            && d >= self.cfg.crown.dry
            && wind.length() >= self.cfg.crown.wind_mps
    }

    /// The hazard a second from burning `from` to `to`, and whether `to`
    /// would burn in its crown; `None` where it cannot catch.
    pub fn hazard(&self, from: u32, to: u32) -> Option<(f32, bool)> {
        if !self.may_catch(to) {
            return None;
        }
        let from_land = self.ground.land(from)?;
        let to_land = self.ground.land(to)?;
        let row = self.row(to_land.plant)?;
        let d = self.plant_dry(to);
        if !dryness::can_catch(self.cfg, d) {
            return None;
        }
        let air_from = self.ground.air(from);
        let air_to = self.ground.air(to);
        let rain = self.rain_term(air_to.rain_mm_h);
        if rain <= 0.0 {
            return None;
        }
        let crown = self.crowns(to_land.plant, d, air_to.wind);
        let lambda = row.spread_per_s
            * dryness::burn(self.cfg, d)
            * self.wind_term(from, to, air_from.wind)
            * self.slope_term(&from_land, &to_land)
            * rain
            * if crown { self.cfg.crown.mult } else { 1.0 };
        Some((lambda, crown))
    }
}

/// The draw for a try from `from` to `to` on `tick`, 0..1.
pub fn draw(seed: u64, blaze: u64, from: u32, to: u32, tick: u64) -> f64 {
    let mut h = seed ^ 0x9e37_79b9_7f4a_7c15;
    for v in [blaze, u64::from(from), u64::from(to), tick] {
        h = splitmix(h ^ v);
    }
    (h >> 11) as f64 / (1u64 << 53) as f64
}

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// What one tick did.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub blaze: Blaze,
    /// Cells lit this tick, in key order.
    pub lit: Vec<u32>,
    /// Cells burnt out this tick, with their scars.
    pub out: Vec<(u32, Burnt)>,
}

impl Step {
    /// The tick as one proposal: the blaze's record and each new scar.
    pub fn proposal(&self, id: u64) -> Proposal {
        let mut records = vec![self.blaze.record(id)];
        records.extend(
            self.out
                .iter()
                .map(|(cell, b)| Record::of(BURNT_KIND, u64::from(*cell), BURNT_SCHEMA, b)),
        );
        Proposal {
            edits: Vec::new(),
            records,
        }
    }
}

/// One tick of `blaze` (design 5 and 7): burning cells lose a second (and
/// more in rain), a doused cell goes out, each burning cell tries each
/// neighbour in key order, and what burnt down is scarred. `None` for a
/// blaze that is out.
pub fn tick<G: Ground>(world: &World<G>, id: u64, blaze: &Blaze) -> Option<Step> {
    if !blaze.is_burning() {
        return None;
    }
    let cfg = world.cfg;
    let mut next = blaze.clone();
    next.tick += 1;
    let t = next.tick;
    let mut out = Vec::new();
    let mut kept = Vec::new();
    for b in &blaze.burning {
        let air = world.ground.air(b.cell);
        let spent = cfg.tick_s * (1.0 + air.rain_mm_h.max(0.0) * cfg.rain.douse_s_per_mmh);
        let left = f32::from(b.left_ds) - spent * 10.0;
        if left <= 0.0 || doused(cfg, world.records, b.cell, world.now_s) {
            out.push((
                b.cell,
                Burnt {
                    blaze: id,
                    out_s: world.now_s,
                    plant: b.plant,
                    crown: b.crown,
                },
            ));
        } else {
            kept.push(Burning {
                left_ds: left.round() as u16,
                ..*b
            });
        }
    }
    // The spread, from the cells burning at the tick's start.
    let mut lit: Vec<Burning> = Vec::new();
    let mut taken = BTreeSet::new();
    let limits = cfg.limits;
    'spread: for b in &blaze.burning {
        for n in world.ground.neighbours(b.cell) {
            if taken.contains(&n) || blaze.burning_at(n).is_some() {
                continue;
            }
            let Some((lambda, crown)) = world.hazard(b.cell, n) else {
                continue;
            };
            let p = 1.0 - (-f64::from(lambda * cfg.tick_s)).exp();
            if draw(world.seed, id, b.cell, n, t) >= p {
                continue;
            }
            if next.cells + lit.len() as u32 >= limits.cells
                || kept.len() + lit.len() >= limits.burning
            {
                next.limited = true;
                break 'spread;
            }
            let plant = world.ground.land(n).map_or(Plant::None, |l| l.plant);
            taken.insert(n);
            lit.push(Burning {
                cell: n,
                plant,
                left_ds: world.burn_ds(plant),
                crown,
            });
        }
    }
    next.cells += lit.len() as u32;
    let lit_cells: Vec<u32> = lit.iter().map(|b| b.cell).collect();
    kept.extend(lit);
    kept.sort_by_key(|b| b.cell);
    next.burning = kept;
    if next.burning.is_empty() {
        next.state = State::Out(world.now_s);
    }
    let mut lit_cells = lit_cells;
    lit_cells.sort_unstable();
    Some(Step {
        blaze: next,
        lit: lit_cells,
        out,
    })
}

/// What a torch or a strike does to a plant (design 6).
#[derive(Clone, Debug, PartialEq)]
pub enum Lighting {
    /// A blaze starts on the cell.
    Blaze(Blaze),
    /// A small flame that dies in `damp_flame_s` and saves nothing: the
    /// plant is too green, it rains, or two blazes already burn.
    Damp,
    /// Nothing there burns.
    Nothing,
}

/// A lit torch held to `cell` (design 6).
pub fn ignite<G: Ground>(world: &World<G>, cell: u32, lit_by: LitBy) -> Lighting {
    let Some(land) = world.ground.land(cell) else {
        return Lighting::Nothing;
    };
    if !land.plant.burns() || world.yields.has_cell(cell) {
        return Lighting::Nothing;
    }
    if world.burning_anywhere(cell) {
        return Lighting::Nothing;
    }
    let d = world.plant_dry(cell);
    let air = world.ground.air(cell);
    if !dryness::can_catch(world.cfg, d)
        || air.rain_mm_h >= world.cfg.rain.no_catch_mmh
        || live(world.records).len() >= world.cfg.limits.live_blazes
    {
        return Lighting::Damp;
    }
    let crown = world.crowns(land.plant, d, air.wind);
    Lighting::Blaze(Blaze {
        started_s: world.now_s,
        origin: cell,
        lit_by,
        state: State::Burning,
        tick: 0,
        burning: vec![Burning {
            cell,
            plant: land.plant,
            left_ds: world.burn_ds(land.plant),
            crown,
        }],
        cells: 1,
        limited: false,
    })
}

/// The id a new blaze takes: one past the highest.
pub fn next_id(records: &Records) -> u64 {
    records
        .of_kind(BLAZE_KIND)
        .map(|r| r.id + 1)
        .max()
        .unwrap_or(1)
}

/// Whether a lightning strike on `cell` starts a blaze (design 6): little
/// rain, a very dry plant, and no blaze lit by lightning within the quiet
/// spell and reach. The strike's own landing is `lightning-fire`'s.
pub fn lightning_starts<G: Ground>(world: &World<G>, cell: u32) -> bool {
    let i = &world.cfg.ignite;
    let air = world.ground.air(cell);
    if air.rain_mm_h >= i.lightning_rain_below_mmh || world.plant_dry(cell) < i.lightning_dry {
        return false;
    }
    let Some(here) = world.ground.centre(cell) else {
        return false;
    };
    let quiet_s = f64::from(i.lightning_quiet_days) * f64::from(crate::daylight::DAY_S);
    !blazes(world.records).iter().any(|(_, b)| {
        b.lit_by == LitBy::Lightning
            && world.now_s - b.started_s < quiet_s
            && world.ground.centre(b.origin).is_some_and(|o| {
                crate::sites::arc_m(o, here, world.ground.radius_m()) < i.lightning_quiet_m
            })
    })
}

/// A bucket poured on `cell` (design 7): the player's records for it and
/// its neighbours, which put out what burns there on the next tick and keep
/// them wet for `bucket_wet_days`.
pub fn douse<G: Ground>(ground: &G, cell: u32, at_s: f64) -> Vec<Record> {
    std::iter::once(cell)
        .chain(ground.neighbours(cell))
        .map(|c| Record::of(DOUSED_KIND, u64::from(c), DOUSED_SCHEMA, &Doused { at_s }))
        .collect()
}

/// The player's record of lighting blaze `id` at `cell`.
pub fn ignition_record(id: u64, cell: u32, at_s: f64) -> Record {
    Record::of(IGNITION_KIND, id, IGNITION_SCHEMA, &Ignition { cell, at_s })
}

/// Every live blaze's ticks up to `ticks`, each taken into a working copy of
/// the records so the next sees it: what a server runs once a second, and
/// what a test runs many of. Each proposal is checked against the yields.
pub fn run<G: Ground>(world: &World<G>, ids: &[u64]) -> Vec<(Author, Proposal)> {
    let mut working = world.records.clone();
    let mut out = Vec::new();
    for &id in ids {
        let Some(blaze) = working.get(BLAZE_KIND, id).and_then(|r| r.read::<Blaze>()) else {
            continue;
        };
        let w = World {
            records: &working,
            ..*world
        };
        let Some(step) = tick(&w, id, &blaze) else {
            continue;
        };
        let proposal = step.proposal(id);
        if world.yields.check(&proposal, &working).is_err() {
            continue;
        }
        for r in &proposal.records {
            working.put(r.clone());
        }
        out.push((author(id), proposal));
    }
    out
}

impl<G: Ground> Clone for World<'_, G> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<G: Ground> Copy for World<'_, G> {}
