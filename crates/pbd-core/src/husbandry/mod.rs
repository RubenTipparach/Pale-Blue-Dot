//! Keeping animals (`openspec/changes/animal-husbandry`, the first playable
//! slice, design decision 12): the land species and how each is kept, the
//! genome, taming, pens, needs and products, and the stored records.
//!
//! Every rule is here, pure and deterministic, with no Bevy; the app adapts
//! them to the world (`pbd_app::husbandry`). The farm is DERIVED from world
//! time and the record's last player-written state (S7): nothing here steps a
//! clock or writes on its own, so a farm costs nothing while the player is
//! elsewhere and catching up a hundred days is one call.
//!
//! Land species live in their own append-only roster beside the fish (S2):
//! a caught fish is saved as its index into `fauna.ron`'s list, which every
//! fish rule reads, so mixing animals into it would ask every one of those
//! rules to filter.

pub mod breeding;
pub mod farm;
pub mod genome;
pub mod herds;
pub mod pen;
pub mod record;
pub mod taming;
pub mod young;

use crate::fauna::{Class, Status};
use crate::goods::Good;
use crate::inventory::Item;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use farm::{Conditions, FarmState};
pub use genome::Genome;
pub use pen::{Edge, Pen, PenRefusal, find_pen};
pub use record::{AnimalBody, FenceBody};
pub use taming::Trust;
pub use young::{Birth, Stage, Young};

/// The shipped `husbandry.ron`, which is also the one source of the
/// defaults: a missing file or field takes these.
pub const HUSBANDRY_RON: &str = include_str!("../../../../assets/config/husbandry.ron");

/// How a body is built and moves: the rig the app poses (S11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Plan {
    /// Two legs, a bobbing head, a tail that flicks: the hen.
    Bird,
    /// Four legs, a head that grazes: the woolback and the plodder.
    Quadruped,
}

/// A sex. Hens lay and cows give milk; a product may name one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Sex {
    #[default]
    F,
    M,
}

/// Something a species eats from the hand (S3).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Food {
    /// A grass block, which every kit holds and a shovel gives back: what a
    /// grazer eats in its pen anyway, so it eats it from the hand too.
    Turf,
    /// A good, by its saved code: farming's wheat, turnips and seed.
    Good(String),
}

impl Food {
    /// Whether `item` is this food.
    pub fn is(&self, item: Item) -> bool {
        match self {
            Food::Turf => matches!(
                item,
                Item::Block(Material::Grass | Material::DryGrass | Material::JungleGrass)
            ),
            Food::Good(code) => Good::by_code(code).is_some_and(|g| item == Item::Good(g)),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Food::Turf => crate::locale::tr("husbandry.food.turf").into(),
            Food::Good(code) => {
                Good::by_code(code).map_or_else(|| code.clone(), |g| g.name().into())
            }
        }
    }
}

/// Something an animal gives on a schedule (decision 7).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    /// The good's saved code (`goods::GOODS`): egg, wool, milk.
    pub good: String,
    /// Units per period at a yield factor of one.
    pub amount: f32,
    /// World days per period.
    pub period_days: f32,
    /// Only animals of this sex give it; none for both.
    #[serde(default)]
    pub sex: Option<Sex>,
}

impl Product {
    pub fn good(&self) -> Option<Good> {
        Good::by_code(&self.good)
    }

    pub fn gives(&self, sex: Sex) -> bool {
        self.sex.is_none_or(|s| s == sex)
    }
}

impl Keeping {
    /// Whether it eats `item` from the hand.
    pub fn eats(&self, item: Item) -> bool {
        self.foods.iter().any(|f| f.is(item))
    }

    /// What it eats, for the HUD: "wheat seed or wheat".
    pub fn food_name(&self) -> String {
        let mut names = self.foods.iter().map(Food::name);
        let first = names.next().unwrap_or_default();
        names.fold(first, |all, next| {
            crate::locale::tr_with("husbandry.food.or", &[("first", &all), ("second", &next)])
                .into()
        })
    }
}

/// How a tameable species is kept (decisions 3, 4, 6).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Keeping {
    /// What it eats from the hand, its favourite first.
    pub foods: Vec<Food>,
    /// Trust one feed gives a calm animal (full is 100).
    pub trust_gain: f32,
    /// Pen cells each animal needs for space.
    pub space_cells: f32,
    /// Grass cells of a pen one animal grazes a day.
    pub grazing_cells: f32,
    pub products: Vec<Product>,
    /// Coats and patterns, the earlier dominant (decision 9).
    pub coats: Vec<String>,
    pub patterns: Vec<String>,
    /// The rare recessive morph's name.
    pub morph: String,
    /// Days from birth to adult; a tamed wild animal is this old at least.
    pub adult_days: f32,
    /// One in this many of a herd is male.
    pub male_one_in: u32,
    /// Its life stages and how it is born (design 13, E1).
    pub young: Young,
}

/// One land species (decision 1): what the field guide, the herds, the rig
/// and the keeping read, side by side.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LandSpecies {
    /// Its id; its name is `species.<id>.name` in the locale file.
    pub id: String,
    pub class: Class,
    pub status: Status,
    pub plan: Plan,
    /// A herd's size, fewest and most.
    pub herd: (u8, u8),
    /// Body length nose to tail and height at the back, m.
    pub length_m: f32,
    pub height_m: f32,
    /// Walking and running speed, m/s.
    pub walk_mps: f32,
    pub run_mps: f32,
    /// Metres one full leg cycle carries it: what keeps feet from sliding.
    pub stride_m: f32,
    /// Body, belly or wattle, legs and head, linear-ish RGB 0..1.
    pub colours: [[f32; 3]; 4],
    /// How it is kept; `None` for a species that is not.
    #[serde(default)]
    pub keep: Option<Keeping>,
}

impl LandSpecies {
    /// Its name: "Yard hen".
    pub fn name(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("species.{}.name", self.id))
    }

    /// Its name mid-sentence ("the yard hen"): `species.<id>.lower`.
    pub fn lower_name(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("species.{}.lower", self.id))
    }

    /// Its name as missions and their saved progress name it: the English
    /// name in lower case, whatever the player's language ("yard hen").
    pub fn data_name(&self) -> String {
        crate::locale::english(&format!("species.{}.name", self.id)).to_lowercase()
    }

    /// Its field-guide entry.
    pub fn entry(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("species.{}.entry", self.id))
    }

    /// Its field-guide tip.
    pub fn tip(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("species.{}.tip", self.id))
    }

    /// Why it cannot be tamed, or `None` where it can.
    pub fn refusal(&self) -> Option<crate::locale::Localized> {
        self.status.refusal(self.class).or_else(|| {
            self.keep
                .is_none()
                .then(|| crate::locale::tr("husbandry.refusal.not_kept"))
        })
    }
}

/// Taming by feeding (decision 3; S4).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TamingSettings {
    /// Fear radius while the player moves, and while they stand still (the
    /// crouch the game does not have yet), m.
    pub fear_moving_m: f32,
    pub fear_still_m: f32,
    /// How close a feed or a use reaches, m.
    pub reach_m: f32,
    /// Trust lost to a spook.
    pub spook_cost: f32,
    /// Trust lost an in-game hour while unfinished.
    pub decay_per_hour: f32,
    /// Seconds a newly tamed animal follows.
    pub follow_s: f32,
}

/// Needs, mood and health (decision 6; S8, S9).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NeedSettings {
    /// Mood each met need adds, and the mood a product needs.
    pub mood_per_need: f32,
    pub content_mood: f32,
    /// Health gained (or lost, with two needs unmet) a world day.
    pub health_per_day: f32,
    /// Below this an animal is sick and gives nothing.
    pub sick_below: f32,
    /// World days without food before it strays.
    pub stray_after_days: f32,
    /// World days a hand feed lasts.
    pub hand_feed_days: f32,
    /// At most this many world days of a product (or one period, if longer) wait.
    pub cap_days: f32,
}

/// Wild herds (S10) and drawing them (S11).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HerdSettings {
    /// The squares the surface is cut into, m on a side.
    pub square_m: f32,
    /// The chance a square holds a herd, 0..1.
    pub chance: f32,
    /// How far a herd's members stand from its centre, m.
    pub spread_m: f32,
    /// Animals within this of the player are drawn, m.
    pub near_m: f32,
    /// Seconds an animal takes to fade in or out.
    pub fade_s: f32,
}

/// `assets/config/husbandry.ron`.
/// Eggs, pairing, inheritance and inbreeding (design 13, E3-E8).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BreedingSettings {
    /// A dam pairs only at this mood or more, 0..100.
    pub mood_min: f32,
    /// Both pair only at this health or more, 0..100.
    pub health_min: f32,
    /// The pairing check runs once a world day, at this fraction of it.
    pub morning: f32,
    /// One allele in this many mutates.
    pub mutate_one_in: u32,
    /// A quantitative allele mutates by up to this, 0..1.
    pub mutate_step: f32,
    /// The young's hardiness falls by this times F.
    pub hardiness_per_f: f32,
    /// Generations the pedigree is read back for F.
    pub generations: u32,
    /// The screen warns above this F (first cousins).
    pub warn_f: f32,
    /// A carried fertile egg is cold this long after it was laid, days.
    pub cold_after_days: f32,
    /// What one incubator holds.
    pub incubator_eggs: u8,
    /// A lamp must stand this near an incubator, metres.
    pub lamp_m: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HusbandrySettings {
    pub taming: TamingSettings,
    pub needs: NeedSettings,
    pub herds: HerdSettings,
    pub breeding: BreedingSettings,
    /// Each body's land species, in roster order. APPEND ONLY: a record
    /// names its species by index. A body with none has no land life.
    pub land: BTreeMap<String, Vec<LandSpecies>>,
}

impl Default for HusbandrySettings {
    fn default() -> Self {
        // The one source of the defaults is the shipped file (S2).
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Whole {
            taming: TamingSettings,
            needs: NeedSettings,
            herds: HerdSettings,
            breeding: BreedingSettings,
            land: BTreeMap<String, Vec<LandSpecies>>,
        }
        let w: Whole = ron::from_str(HUSBANDRY_RON).expect("the shipped husbandry.ron parses");
        Self {
            taming: w.taming,
            needs: w.needs,
            herds: w.herds,
            breeding: w.breeding,
            land: w.land,
        }
    }
}

impl HusbandrySettings {
    /// A body's land species; empty for a body with none.
    pub fn roster(&self, body: &str) -> &[LandSpecies] {
        self.land.get(body).map_or(&[], Vec::as_slice)
    }

    pub fn validate(&self) -> Result<(), String> {
        let t = &self.taming;
        positive(
            "taming",
            &[
                t.fear_moving_m,
                t.fear_still_m,
                t.reach_m,
                t.spook_cost,
                t.decay_per_hour,
                t.follow_s,
            ],
        )?;
        if t.fear_still_m > t.fear_moving_m {
            return Err("taming: standing still must not frighten more than moving".into());
        }
        let n = &self.needs;
        positive(
            "needs",
            &[
                n.mood_per_need,
                n.content_mood,
                n.health_per_day,
                n.sick_below,
                n.stray_after_days,
                n.hand_feed_days,
                n.cap_days,
            ],
        )?;
        let h = &self.herds;
        positive("herds", &[h.square_m, h.spread_m, h.near_m, h.fade_s])?;
        if !(0.0..=1.0).contains(&h.chance) {
            return Err("herds: chance is 0..1".into());
        }
        let b = &self.breeding;
        positive(
            "breeding",
            &[
                b.mutate_step,
                b.hardiness_per_f,
                b.warn_f,
                b.cold_after_days,
                b.lamp_m,
            ],
        )?;
        if !(0.0..=100.0).contains(&b.mood_min)
            || !(0.0..=100.0).contains(&b.health_min)
            || !(0.0..1.0).contains(&b.morning)
            || b.mutate_one_in == 0
            || b.generations == 0
            || b.incubator_eggs == 0
        {
            return Err("breeding: mood and health 0..100, morning 0..1, counts at least 1".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for (body, species) in &self.land {
            for sp in species {
                let at = format!("{body}/{}", sp.id);
                if !seen.insert(sp.id.clone()) {
                    return Err(format!("{at}: a species lives on one body only"));
                }
                positive(
                    &at,
                    &[
                        sp.length_m,
                        sp.height_m,
                        sp.walk_mps,
                        sp.run_mps,
                        sp.stride_m,
                    ],
                )?;
                if sp.herd.0 == 0 || sp.herd.1 < sp.herd.0 {
                    return Err(format!("{at}: a herd has at least one animal"));
                }
                if sp
                    .colours
                    .iter()
                    .flatten()
                    .any(|c| !(0.0..=1.0).contains(c))
                {
                    return Err(format!("{at}: colours are 0..1"));
                }
                let tameable = matches!(sp.status, Status::Tameable | Status::Domestic);
                if tameable != sp.keep.is_some() {
                    return Err(format!(
                        "{at}: a tameable species says how it is kept, and only one"
                    ));
                }
                if let Some(k) = &sp.keep {
                    positive(
                        &at,
                        &[k.trust_gain, k.space_cells, k.grazing_cells, k.adult_days],
                    )?;
                    for f in &k.foods {
                        if let Food::Good(code) = f
                            && Good::by_code(code).is_none()
                        {
                            return Err(format!("{at}: eats {code}, which is no good"));
                        }
                    }
                    if k.foods.is_empty()
                        || k.coats.is_empty()
                        || k.patterns.is_empty()
                        || k.male_one_in == 0
                    {
                        return Err(format!("{at}: coats, patterns and sexes"));
                    }
                    k.young.validate(&at, k.adult_days)?;
                    for p in &k.products {
                        if p.good().is_none_or(|g| g == Good::FENCE || g == Good::GATE) {
                            return Err(format!("{at}: {} is not a product", p.good));
                        }
                        positive(&at, &[p.amount, p.period_days])?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn positive(name: &str, values: &[f32]) -> Result<(), String> {
    if values.iter().all(|v| v.is_finite() && *v > 0.0) {
        Ok(())
    } else {
        Err(format!("{name}: every value must be finite and positive"))
    }
}

#[cfg(test)]
mod tests;
