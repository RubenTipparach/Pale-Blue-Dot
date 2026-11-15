//! Eggs, pairing, births, inheritance and inbreeding (`animal-husbandry`
//! design 9 and 13, E2-E8).
//!
//! A **brood** is a stored record: a clutch of fertile eggs (carried, under a
//! broody hen, or in an incubator) or a pregnancy. What the farm does on its
//! own (a pair conceiving, a brood hatching or being born) is worked out here
//! as a list of new records, in day order, up to a moment: [`farm_events`].
//! The app proposes each one under the world's author. Nothing here edits a
//! record the player wrote: a brood is *done* when an animal names it.

use super::record::{AnimalBody, Life, Via};
use super::{BreedingSettings, Genome, Keeping, LandSpecies, Sex, Young, genome};
use crate::fauna::hash2;
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A brood's record kind and schema.
pub const BROOD: &str = "brood";
pub const BROOD_SCHEMA: u32 = 1;
/// An incubator piece's (E4): id is its cell's key.
pub const INCUBATOR: &str = "incubator";
pub const INCUBATOR_SCHEMA: u32 = 1;

/// Where a brood is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Place {
    /// Fertile eggs collected and in the player's hands: their parentage.
    #[default]
    Carried,
    /// Under a broody hen, by her record id.
    Hen(u64),
    /// In the incubator on this cell.
    Incubator(u32),
    /// A pregnancy.
    Womb,
}

/// A brood: `rec @p brood <id> 1 (...)` for eggs the player handled, and
/// `rec @whusbandry:pen/<pen> brood ...` for a conception.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct BroodBody {
    pub body: String,
    pub sp: u16,
    pub dam: u64,
    pub sire: u64,
    /// Eggs in it, or 0 for a pregnancy (the litter is drawn at birth).
    pub eggs: u8,
    /// World day the eggs were laid, or the day of conception.
    pub laid: f64,
    /// World day the eggs were set; `None` while carried.
    pub set: Option<f64>,
    pub place: Place,
}

impl Default for BroodBody {
    fn default() -> Self {
        Self {
            body: crate::fauna::HOME_BODY.into(),
            sp: 0,
            dam: 0,
            sire: 0,
            eggs: 0,
            laid: 0.0,
            set: None,
            place: Place::Carried,
        }
    }
}

impl BroodBody {
    pub fn record(&self, id: u64) -> Record {
        Record::of(BROOD, id, BROOD_SCHEMA, self)
    }

    /// The day it hatches or is born, or `None` while carried.
    pub fn due(&self, young: &Young) -> Option<f64> {
        match (self.place, young.birth) {
            (Place::Hen(_) | Place::Incubator(_), super::Birth::Eggs { incubation_days, .. }) => {
                self.set.map(|s| s + f64::from(incubation_days))
            }
            (Place::Womb, super::Birth::Live { gestation_days, .. }) => {
                Some(self.laid + f64::from(gestation_days))
            }
            _ => None,
        }
    }

    /// Whether carried eggs are still warm at `day`.
    pub fn warm(&self, day: f64, s: &BreedingSettings) -> bool {
        day - self.laid <= f64::from(s.cold_after_days)
    }
}

/// An incubator piece.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct IncubatorBody {
    pub state: super::record::Standing,
}

impl IncubatorBody {
    pub fn record(&self, cell: u32) -> Record {
        Record::of(INCUBATOR, u64::from(cell), INCUBATOR_SCHEMA, self)
    }
}

/// Every brood this build can read, by id.
pub fn broods(records: &Records) -> BTreeMap<u64, BroodBody> {
    records
        .of_kind(BROOD)
        .filter_map(|r| r.read::<BroodBody>().map(|b| (r.id, b)))
        .collect()
}

pub fn next_brood_id(records: &Records) -> u64 {
    records.of_kind(BROOD).map(|r| r.id + 1).max().unwrap_or(1)
}

/// Every standing incubator's cell.
pub fn incubators(records: &Records) -> Vec<u32> {
    records
        .of_kind(INCUBATOR)
        .filter_map(|r| {
            let b = r.read::<IncubatorBody>()?;
            (b.state == super::record::Standing::Placed).then_some(r.id as u32)
        })
        .collect()
}

// ---------------------------------------------------------------- inheritance

/// The young's genome (E7): one allele of each gene from each parent, by
/// `hash(sire, dam, n, gene, side)`, each mutating one time in
/// `mutate_one_in`. `n` is the pair's birth count so far. The genes that
/// mutated come back too.
pub fn inherit(
    sire: (u64, &Genome),
    dam: (u64, &Genome),
    n: u32,
    keep: &Keeping,
    s: &BreedingSettings,
) -> (Genome, Vec<usize>) {
    let mut out = [0u8; genome::BYTES];
    let mut mutated = Vec::new();
    let pair = hash2(sire.0, dam.0);
    for gene in 0..genome::GENES {
        for (side, parent) in [sire.1, dam.1].into_iter().enumerate() {
            let h = hash2(hash2(pair, u64::from(n)), (gene * 2 + side) as u64 + 0x1e6e);
            let (a, b) = parent.alleles(gene);
            let mut allele = if h & 1 == 0 { a } else { b };
            if (h >> 8) % u64::from(s.mutate_one_in.max(1)) == 0 {
                mutated.push(gene);
                let r = ((h >> 24) & 0xffff) as f32 / 65535.0;
                allele = match gene {
                    genome::COAT => step_list(allele, r, keep.coats.len()),
                    genome::PATTERN => step_list(allele, r, keep.patterns.len()),
                    genome::MORPH => 1 - allele.min(1),
                    _ => {
                        let by = (r - 0.5) * 2.0 * s.mutate_step * 255.0;
                        (f32::from(allele) + by).round().clamp(0.0, 255.0) as u8
                    }
                };
            }
            out[gene * 2 + side] = allele;
        }
    }
    mutated.dedup();
    (Genome(out), mutated)
}

/// Another entry of a list of `len`, never the same one.
fn step_list(allele: u8, r: f32, len: usize) -> u8 {
    if len < 2 {
        return allele;
    }
    let by = 1 + (r * (len - 1) as f32).min((len - 2) as f32) as usize;
    ((usize::from(allele) + by) % len) as u8
}

/// The young's sex, by the same hash.
pub fn sex_of(sire: u64, dam: u64, n: u32) -> Sex {
    if hash2(hash2(sire, dam), u64::from(n) ^ 0x5e_c5e3) & 1 == 0 {
        Sex::F
    } else {
        Sex::M
    }
}

/// Every value a quantitative gene of the young can take before mutation:
/// the four allele pairs, each one in four (the screen's ticks).
pub fn offspring_values(sire: &Genome, dam: &Genome, gene: usize) -> [f32; 4] {
    let (a, b) = sire.alleles(gene);
    let (c, d) = dam.alleles(gene);
    let v = |x: u8, y: u8| (f32::from(x) + f32::from(y)) / 510.0;
    [v(a, c), v(a, d), v(b, c), v(b, d)]
}

/// The odds of each shown entry of a discrete gene (coat, pattern) for the
/// young, before mutation: index to quarters.
pub fn discrete_odds(sire: &Genome, dam: &Genome, gene: usize) -> BTreeMap<usize, u32> {
    let (a, b) = sire.alleles(gene);
    let (c, d) = dam.alleles(gene);
    let mut out = BTreeMap::new();
    for (x, y) in [(a, c), (a, d), (b, c), (b, d)] {
        *out.entry(usize::from(x.min(y))).or_insert(0) += 1;
    }
    out
}

/// Quarters of the young that show the morph.
pub fn morph_quarters(sire: &Genome, dam: &Genome) -> u32 {
    let (a, b) = sire.alleles(genome::MORPH);
    let (c, d) = dam.alleles(genome::MORPH);
    [(a, c), (a, d), (b, c), (b, d)]
        .into_iter()
        .filter(|(x, y)| *x == 1 && *y == 1)
        .count() as u32
}

/// Expressed hardiness, 0..1, after inbreeding (E8).
pub fn hardiness(genome: &Genome, f: f32, s: &BreedingSettings) -> f32 {
    (genome.value(genome::HARDINESS) - s.hardiness_per_f * f).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------- the pedigree

/// Who an animal's parents are and when it was born: the pedigree read off
/// the records.
#[derive(Clone, Debug, Default)]
pub struct Pedigree {
    pub of: BTreeMap<u64, (Option<u64>, Option<u64>, f64)>,
}

impl Pedigree {
    pub fn from_animals<'a>(animals: impl IntoIterator<Item = (&'a u64, &'a AnimalBody)>) -> Self {
        Self {
            of: animals
                .into_iter()
                .map(|(id, b)| (*id, (b.sire, b.dam, b.born)))
                .collect(),
        }
    }

    /// Wright's coefficient of a young of `sire` and `dam`: their kinship,
    /// read back `generations` generations (E8).
    pub fn f_of_young(&self, sire: u64, dam: u64, generations: u32) -> f32 {
        self.kin(sire, dam, 1, generations)
    }

    /// An animal's own F.
    pub fn f(&self, id: u64, generations: u32) -> f32 {
        self.f_at(id, 1, generations)
    }

    fn f_at(&self, id: u64, depth: u32, limit: u32) -> f32 {
        match self.of.get(&id) {
            Some((Some(s), Some(d), _)) => self.kin(*s, *d, depth, limit),
            _ => 0.0,
        }
    }

    /// The coefficient of kinship by the recursive method: the younger's
    /// parents' kinship with the elder. Unknown animals are founders.
    fn kin(&self, a: u64, b: u64, depth: u32, limit: u32) -> f32 {
        if depth > limit || !self.of.contains_key(&a) || !self.of.contains_key(&b) {
            return 0.0;
        }
        if a == b {
            return 0.5 * (1.0 + self.f_at(a, depth + 1, limit));
        }
        let (born_a, born_b) = (self.of[&a].2, self.of[&b].2);
        let (young, old) = if (born_a, a) >= (born_b, b) {
            (a, b)
        } else {
            (b, a)
        };
        let (sire, dam, _) = self.of[&young];
        let side = |p: Option<u64>| p.map_or(0.0, |p| self.kin(p, old, depth + 1, limit));
        0.5 * (side(sire) + side(dam))
    }
}

// ---------------------------------------------------------------- the farm's own events

/// The first pairing check at or after `day`.
pub fn morning_after(day: f64, s: &BreedingSettings) -> f64 {
    let m = f64::from(s.morning);
    let base = day.floor() + m;
    if base >= day { base } else { base + 1.0 }
}

/// One change the farm makes on its own, as records to propose under the
/// world's author named by `author` (`kind`, `id`).
#[derive(Clone, Debug, PartialEq)]
pub struct FarmEvent {
    pub day: f64,
    /// `("pen", pen id)` for a conception, `("brood", brood id)` for a birth.
    pub author: (&'static str, u64),
    pub records: Vec<Record>,
}

/// How an animal stands now, as the app reads it off the farm (S7: taken as
/// it is now for the whole span).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fitness {
    pub mood: f32,
    pub health: f32,
}

/// Everything the farm does on its own up to `now` (E5, E6), in day order,
/// given the records as they stand. `fit` answers how an animal fares now;
/// `lamp_lit` whether an incubator's cell has a lamp beside it. Running it to
/// `now` once gives the same events as running it to every moment before
/// `now` and storing each answer between: every event's day is decided by
/// the records alone, and only events at or before `now` come out.
pub fn farm_events(
    records: &Records,
    roster: &[LandSpecies],
    s: &BreedingSettings,
    now: f64,
    fit: &dyn Fn(&AnimalBody) -> Fitness,
    lamp_lit: &dyn Fn(u32) -> bool,
) -> Vec<FarmEvent> {
    let mut animals: BTreeMap<u64, AnimalBody> = super::record::animals(records).into_iter().collect();
    let mut all_broods = broods(records);
    let mut next_animal = super::record::next_animal_id(records);
    let mut next_brood = next_brood_id(records);
    let mut out = Vec::new();
    // Bounded: a farm left for a long time still catches up in one call,
    // and a runaway (which space and mood prevent) cannot hang a frame.
    for _ in 0..256 {
        let done: std::collections::BTreeSet<u64> =
            animals.values().filter_map(|a| a.brood).collect();
        let keep_of = |sp: u16| roster.get(sp as usize).and_then(|x| x.keep.as_ref());
        // The earliest birth due.
        let mut best: Option<(f64, u8, u64)> = None;
        for (id, b) in &all_broods {
            if done.contains(id) {
                continue;
            }
            let Some(keep) = keep_of(b.sp) else { continue };
            let Some(due) = b.due(&keep.young) else { continue };
            let ready = match b.place {
                Place::Hen(hen) => animals.get(&hen).is_some_and(|h| h.state == Life::Alive),
                Place::Incubator(cell) => lamp_lit(cell),
                Place::Womb => true,
                Place::Carried => false,
            };
            if ready && due <= now && best.is_none_or(|x| (due, 0, *id) < x) {
                best = Some((due, 0, *id));
            }
        }
        // The earliest conception.
        let pedigree = Pedigree::from_animals(&animals);
        let mut conceive: Option<(f64, u64, u64)> = None;
        for (dam_id, dam) in &animals {
            let Some(keep) = keep_of(dam.sp) else { continue };
            let super::Birth::Live { gestation_days, .. } = keep.young.birth else {
                continue;
            };
            if dam.sex != Sex::F || dam.state != Life::Alive || dam.pen.is_none() {
                continue;
            }
            let carrying = all_broods.iter().any(|(id, b)| {
                b.place == Place::Womb && b.dam == *dam_id && !done.contains(id)
            });
            if carrying {
                continue;
            }
            let last_birth = all_broods
                .values()
                .filter(|b| b.place == Place::Womb && b.dam == *dam_id)
                .filter_map(|b| b.due(&keep.young))
                .fold(f64::NEG_INFINITY, f64::max);
            let f_ok = |a: &AnimalBody| {
                let f = fit(a);
                f.health >= s.health_min
            };
            if fit(dam).mood < s.mood_min || !f_ok(dam) {
                continue;
            }
            let adult = |a: &AnimalBody| a.born + f64::from(keep.adult_days);
            let mates: Vec<(&u64, &AnimalBody)> = animals
                .iter()
                .filter(|(_, m)| {
                    m.sex == Sex::M
                        && m.state == Life::Alive
                        && m.sp == dam.sp
                        && m.body == dam.body
                        && m.pen == dam.pen
                        && f_ok(m)
                })
                .collect();
            let chosen = dam
                .mate
                .and_then(|want| mates.iter().find(|(id, _)| **id == want))
                .or_else(|| mates.first());
            let Some((sire_id, sire)) = chosen else { continue };
            let from = [
                dam.as_of,
                sire.as_of,
                adult(dam),
                adult(sire),
                last_birth + f64::from(gestation_days),
            ]
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max);
            let day = morning_after(from, s);
            if day <= now && conceive.is_none_or(|c| (day, *dam_id) < (c.0, c.1)) {
                conceive = Some((day, *dam_id, **sire_id));
            }
        }
        // Births first on a tie: a dam due and conceiving the same morning
        // gives birth, then waits her cooldown.
        let take_birth = match (best, conceive) {
            (Some(b), Some(c)) => b.0 <= c.0,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        if take_birth {
            let (due, _, brood_id) = best.expect("a birth");
            let brood = all_broods[&brood_id].clone();
            let keep = keep_of(brood.sp).expect("a kept species");
            let (Some(sire), Some(dam)) = (animals.get(&brood.sire), animals.get(&brood.dam)) else {
                // A parent this build cannot read: the brood waits.
                all_broods.remove(&brood_id);
                continue;
            };
            let f = pedigree.f_of_young(brood.sire, brood.dam, s.generations);
            let count = match keep.young.birth {
                super::Birth::Eggs { .. } => u32::from(brood.eggs),
                super::Birth::Live { litter, .. } => {
                    let span = u64::from(litter.1 - litter.0) + 1;
                    let n = u32::from(litter.0) + (hash2(brood_id, 0x1177e5) % span) as u32;
                    ((n as f32 * (1.0 - f)).floor() as u32).max(1)
                }
            };
            let before = animals
                .values()
                .filter(|a| a.sire == Some(brood.sire) && a.dam == Some(brood.dam))
                .count() as u32;
            let (home, pen) = match brood.place {
                Place::Incubator(cell) => (cell, None),
                Place::Hen(hen) => animals.get(&hen).map_or((dam.home, dam.pen), |h| (h.home, h.pen)),
                _ => (dam.home, dam.pen),
            };
            let mut records = Vec::new();
            for k in 0..count {
                let n = before + k;
                let (genes, _) = inherit((brood.sire, &sire.genome()), (brood.dam, &dam.genome()), n, keep, s);
                let young = AnimalBody {
                    body: brood.body.clone(),
                    sp: brood.sp,
                    sex: sex_of(brood.sire, brood.dam, n),
                    born: due,
                    genes: genes.0.to_vec(),
                    sire: Some(brood.sire),
                    dam: Some(brood.dam),
                    pen,
                    home,
                    owner: dam.owner,
                    via: Via::Born,
                    as_of: due,
                    fed_until: due,
                    brood: Some(brood_id),
                    ..AnimalBody::default()
                };
                records.push(young.record(next_animal));
                animals.insert(next_animal, young);
                next_animal += 1;
            }
            out.push(FarmEvent {
                day: due,
                author: (BROOD, brood_id),
                records,
            });
        } else {
            let (day, dam_id, sire_id) = conceive.expect("a conception");
            let dam = &animals[&dam_id];
            let brood = BroodBody {
                body: dam.body.clone(),
                sp: dam.sp,
                dam: dam_id,
                sire: sire_id,
                eggs: 0,
                laid: day,
                set: None,
                place: Place::Womb,
            };
            out.push(FarmEvent {
                day,
                author: ("pen", u64::from(dam.pen.unwrap_or(0))),
                records: vec![brood.record(next_brood)],
            });
            all_broods.insert(next_brood, brood);
            next_brood += 1;
        }
    }
    out
}

/// The adult male sharing her pen, if any: makes her eggs fertile (E3).
pub fn cockerel_for(records: &Records, hen: &AnimalBody, keep: &Keeping, day: f64) -> Option<u64> {
    let pen = hen.pen?;
    super::record::animals(records)
        .into_iter()
        .find(|(_, m)| {
            m.sex == Sex::M
                && m.state == Life::Alive
                && m.sp == hen.sp
                && m.body == hen.body
                && m.pen == Some(pen)
                && day >= m.born + f64::from(keep.adult_days)
        })
        .map(|(id, _)| id)
}

/// The undone broods under a hen or in an incubator: the eggs it holds.
pub fn eggs_at(records: &Records, place: Place) -> u32 {
    let done: std::collections::BTreeSet<u64> = super::record::animals(records)
        .into_iter()
        .filter_map(|(_, a)| a.brood)
        .collect();
    broods(records)
        .into_iter()
        .filter(|(id, b)| b.place == place && !done.contains(id))
        .map(|(_, b)| u32::from(b.eggs))
        .sum()
}

/// The last day a hen's clutch hatches, so she lays again from it (E4).
pub fn sitting_until(records: &Records, hen: u64, young: &Young) -> Option<f64> {
    broods(records)
        .values()
        .filter(|b| b.place == Place::Hen(hen))
        .filter_map(|b| b.due(young))
        .reduce(f64::max)
}

/// The carried clutches to take parentage from when `n` fertile eggs are
/// set at `day`: oldest warm first. Each comes back with how many it gives
/// and its new count; the eggs with no warm parentage left are the rest.
pub fn take_parentage(
    records: &Records,
    n: u32,
    day: f64,
    s: &BreedingSettings,
) -> (Vec<(u64, BroodBody, u8)>, u32) {
    let mut carried: Vec<(u64, BroodBody)> = broods(records)
        .into_iter()
        .filter(|(_, b)| b.place == Place::Carried && b.eggs > 0 && b.warm(day, s))
        .collect();
    carried.sort_by(|a, b| a.1.laid.total_cmp(&b.1.laid).then(a.0.cmp(&b.0)));
    let mut left = n;
    let mut out = Vec::new();
    for (id, b) in carried {
        if left == 0 {
            break;
        }
        let give = (u32::from(b.eggs)).min(left);
        left -= give;
        out.push((id, b, give as u8));
    }
    (out, left)
}

/// A kept record's kind as the identity notes it.
pub const KINDS: [(&str, u32); 2] = [(BROOD, BROOD_SCHEMA), (INCUBATOR, INCUBATOR_SCHEMA)];
