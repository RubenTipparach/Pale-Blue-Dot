//! Town jobs (`townsfolk` design 8 and 9): a town's board, made from its
//! needs and the game day's seed, and the jobs the player has taken.
//!
//! - **A job is a mission row.** Its objectives are the starter missions'
//!   ([`ObjectiveEntry`]: a kind, what it counts, how many), and they are
//!   counted by the same function, [`Progress::record`], from the same
//!   [`Happened`] events. A town job names its thing as `<site>:<thing>`
//!   where only that town's counts (a sale to its trader, a parcel handed
//!   to its people).
//! - **A board** shows three to five jobs ([`JobsConfig::base`] plus one at
//!   each goodwill step in `more_jobs_at`). Each place on it holds one job
//!   for [`JobsConfig::refresh_days`] game days, the places changing on
//!   staggered days, so the same day gives the same board and a job not
//!   taken is replaced after three days. The first place is always the
//!   town's own trade (a harbour's is fish).
//! - **A taken job** is stored in the player record `town_jobs` with its
//!   town and its reward. It never fails or expires (Explorer is calm). Done,
//!   it waits to be handed in at its town; handing in pays its coins and the
//!   town's goodwill, and its id is kept so it is never posted again.

use super::trade::{TradeConfig, Trader};
use super::values::{Group, Values};
use crate::missions::{Chain, Happened, Kind, Mission, Objective, ObjectiveEntry, Progress, Want};
use crate::sites::SiteKind;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const JOBS_KIND: &str = "town_jobs";
pub const JOBS_SCHEMA: u32 = 1;
pub const JOBS_ID: u64 = 0;

/// What a job asks for. Saved by its snake-case name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    /// Sell goods to the town's trader.
    Trade,
    /// Bring the town a good it lacks (sold to its trader).
    Gather,
    /// Bring the town a herd animal's goods, until predators come.
    Hunt,
    /// Carry a parcel to another town.
    Deliver,
    /// Cook a dish.
    Cook,
    /// Catch fish.
    Fish,
    /// Dig stone or ore.
    Mine,
    /// Collect from kept animals.
    Herd,
    /// Bring in a harvest.
    Harvest,
    /// Light the town's lamps at dusk.
    Lamps,
    /// Find a lost villager and walk them home.
    Lost,
    /// Mend a broken piece of a town building.
    Repair,
    /// Dig a rockfall off the town's road (`townsfolk` design 46).
    Rockfall,
    /// Walk with a trader along the road to the next town (design 46).
    Escort,
}

impl JobKind {
    pub const ALL: [JobKind; 14] = [
        JobKind::Trade,
        JobKind::Gather,
        JobKind::Hunt,
        JobKind::Deliver,
        JobKind::Cook,
        JobKind::Fish,
        JobKind::Mine,
        JobKind::Herd,
        JobKind::Harvest,
        JobKind::Lamps,
        JobKind::Lost,
        JobKind::Repair,
        JobKind::Rockfall,
        JobKind::Escort,
    ];

    /// Its name in data and in locale keys (`jobs.kind.<code>`).
    pub fn code(self) -> &'static str {
        match self {
            JobKind::Trade => "trade",
            JobKind::Gather => "gather",
            JobKind::Hunt => "hunt",
            JobKind::Deliver => "deliver",
            JobKind::Cook => "cook",
            JobKind::Fish => "fish",
            JobKind::Mine => "mine",
            JobKind::Herd => "herd",
            JobKind::Harvest => "harvest",
            JobKind::Lamps => "lamps",
            JobKind::Lost => "lost",
            JobKind::Repair => "repair",
            JobKind::Rockfall => "rockfall",
            JobKind::Escort => "escort",
        }
    }
}

/// `assets/config/jobs.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobsConfig {
    /// Jobs on a board before goodwill adds more.
    pub base: u32,
    /// The most a board shows.
    pub most: u32,
    /// Game days a place on the board keeps its job.
    pub refresh_days: u32,
    /// A job pays its objectives' value times this, in whole coins.
    pub pay_share: f32,
    /// What a job's objective is worth where it is no good's value: a fish
    /// of any kind, a parcel, a villager, a lamp, a piece mended.
    pub unit_value: BTreeMap<JobKind, u32>,
    /// How many each kind asks for, least and most.
    pub counts: BTreeMap<JobKind, (u32, u32)>,
    /// The kinds a board posts, in the order they are drawn from.
    pub posted: Vec<JobKind>,
    /// Each kind of town's own job: the board's first place.
    pub signature: BTreeMap<SiteKind, JobKind>,
    /// The dishes a cook job asks for, by recipe code.
    pub dishes: Vec<String>,
    /// The goods a mine job asks for: stone, or ore.
    pub stone: Vec<String>,
    pub ore: Vec<String>,
    /// The goods a herd job collects, and a hunt brings.
    pub herd: Vec<String>,
    pub hunt: Vec<String>,
    /// The produce a harvest job asks for.
    pub harvest: Vec<String>,
    /// Who keeps a town's trade and who gives its jobs, by their roles in
    /// the town's people, first match wins (`townsfolk` decision 1).
    pub trader_roles: Vec<String>,
    pub giver_roles: Vec<String>,
}

impl JobsConfig {
    pub fn parse(text: &str) -> Result<JobsConfig, String> {
        let cfg: JobsConfig = ron::from_str(text).map_err(|e| format!("jobs.ron: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.base == 0 || self.most < self.base || self.refresh_days == 0 {
            return Err("jobs: base, most and refresh_days".into());
        }
        if !(self.pay_share.is_finite() && self.pay_share > 0.0) {
            return Err("jobs: pay_share must be positive".into());
        }
        if self.posted.is_empty() {
            return Err("jobs: nothing posted".into());
        }
        for kind in &self.posted {
            let (lo, hi) = self.count_range(*kind);
            if lo == 0 || hi < lo {
                return Err(format!("jobs: {kind:?} counts"));
            }
        }
        Ok(())
    }

    fn count_range(&self, kind: JobKind) -> (u32, u32) {
        self.counts.get(&kind).copied().unwrap_or((1, 1))
    }
}

/// One job, posted or taken.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    /// Stable for the life of the world: from the town, the place on its
    /// board and the period, never 0.
    pub id: u64,
    /// The town that posted it, which pays it.
    pub site: u32,
    pub kind: JobKind,
    pub objectives: Vec<ObjectiveEntry>,
    pub reward: u64,
    /// A delivery's town.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<u32>,
    /// What it asks for, for its line: a trade code, a recipe code.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub thing: String,
}

impl Job {
    /// The job as a mission row the counting reads. Its title is its id;
    /// the game names it by [`Job::kind`] and [`Job::thing`].
    pub fn mission(&self) -> Mission {
        Mission {
            id: self.mission_id(),
            title: String::new(),
            help: String::new(),
            needs: Vec::new(),
            objectives: self
                .objectives
                .iter()
                .map(|o| Objective {
                    kind: o.kind,
                    want: want_of(&o.what),
                    count: o.count,
                    label: String::new(),
                })
                .collect(),
            pending: None,
            unlocks: Vec::new(),
            after: Vec::new(),
            branch: String::new(),
            icon: String::new(),
            coins: 0,
        }
    }

    /// Its id as the counting names it.
    pub fn mission_id(&self) -> String {
        format!("job-{}", self.id)
    }

    /// How many it asks for, all objectives together.
    pub fn wanted(&self) -> u32 {
        self.objectives.iter().map(|o| o.count).sum()
    }
}

/// An objective's `what` as the counting matches it: `<prefix>*` is any
/// name starting with the prefix, `a|b` is either, `any` is anything.
pub fn want_of(what: &str) -> Want {
    if what == "any" {
        Want::Any
    } else if let Some(prefix) = what.strip_suffix('*') {
        Want::Prefix(prefix.to_string())
    } else {
        Want::Names(what.split('|').map(str::to_string).collect())
    }
}

/// Dusk by the sun's clock, hours (`town-missions-tree` design 8): a board
/// shows its lamps job from the first, and a lamp is lit until the second.
pub const DUSK_FROM_H: f32 = 17.0;
pub const DUSK_TO_H: f32 = 22.0;

/// Whether `hour` (0..24) is dusk.
pub fn is_dusk(hour: f32) -> bool {
    (DUSK_FROM_H..DUSK_TO_H).contains(&hour)
}

/// Whether a posted job is shown on its board at `hour`: a lamps job from
/// dusk to midnight, every other job all day.
pub fn shown_at(job: &Job, hour: f32) -> bool {
    job.kind != JobKind::Lamps || hour >= DUSK_FROM_H
}

/// A town job's thing: `<site>:<thing>`.
pub fn scoped(site: u32, thing: &str) -> String {
    format!("{site}:{thing}")
}

/// SplitMix64: one well-mixed number from another.
pub fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The board's period for one place on it: places change on staggered
/// days, one each day for a board of three.
pub fn period(day: u64, place: u32, refresh_days: u32) -> u64 {
    (day + place as u64) / refresh_days as u64
}

/// A town as its board reads it.
#[derive(Clone, Copy, Debug)]
pub struct Town<'a> {
    pub site: u32,
    pub kind: SiteKind,
    /// Other towns a delivery may go to, nearest first.
    pub neighbours: &'a [u32],
    /// The towns a stored road of the world's joins to this one, by site:
    /// a road job is posted only where there is one (design 46).
    pub roads_to: &'a [u32],
}

/// What a board is made from.
#[derive(Clone, Copy, Debug)]
pub struct Board<'a> {
    pub jobs: &'a JobsConfig,
    pub trade: &'a TradeConfig,
    pub values: &'a Values,
    /// The world's seed.
    pub seed: u64,
}

impl Board<'_> {
    /// The jobs posted in `town` on game `day` at `goodwill`, leaving out any
    /// the player has taken or handed in (`gone`). The same inputs give the
    /// same board.
    pub fn posted(&self, town: Town, day: u64, goodwill: u32, gone: &BTreeSet<u64>) -> Vec<Job> {
        let places =
            (self.jobs.base + self.trade.goodwill.extra_jobs(goodwill)).min(self.jobs.most);
        (0..places)
            .filter_map(|place| self.job_at(town, day, place))
            .filter(|job| !gone.contains(&job.id))
            .collect()
    }

    /// The job in one place of a board, or `None` where the town has none of
    /// that kind to give (no other town to deliver to).
    pub fn job_at(&self, town: Town, day: u64, place: u32) -> Option<Job> {
        let p = period(day, place, self.jobs.refresh_days);
        let roll =
            mix(mix(mix(self.seed ^ 0x6a6f_6273) ^ town.site as u64) ^ ((p << 8) | place as u64));
        let id = roll.max(1);
        let signature = self.jobs.signature.get(&town.kind).copied();
        let posted = &self.jobs.posted;
        let first = (mix(roll) % posted.len() as u64) as usize;
        match (place, signature) {
            (0, Some(kind)) => self.make(town, kind, id, mix(roll ^ 0x5eed)),
            // A kind the town cannot give (no other town to deliver to, no
            // road for a road job) gives its place to the next posted kind.
            _ => (0..posted.len()).find_map(|k| {
                let kind = posted[(first + k) % posted.len()];
                self.make(town, kind, id, mix(roll ^ 0x5eed))
            }),
        }
    }

    /// One job of a kind, its count and its thing rolled from `roll`.
    pub fn make(&self, town: Town, kind: JobKind, id: u64, roll: u64) -> Option<Job> {
        let (lo, hi) = self.jobs.count_range(kind);
        let count = lo + (roll % (hi - lo + 1) as u64) as u32;
        let pick = |list: &[String]| -> Option<String> {
            (!list.is_empty()).then(|| list[(mix(roll) % list.len() as u64) as usize].clone())
        };
        let trader = Trader {
            cfg: self.trade,
            values: self.values,
            kind: town.kind,
            goodwill: 0,
            fish_ids: &[],
        };
        let site = town.site;
        let (objective, thing, to): (ObjectiveEntry, String, Option<u32>) = match kind {
            JobKind::Trade => (
                entry(Kind::Sell, format!("{}*", scoped(site, "")), count),
                String::new(),
                None,
            ),
            JobKind::Gather => {
                // The good the town lacks most, among those it buys.
                let lacking: Vec<String> = self
                    .values
                    .codes()
                    .filter(|c| trader.buys(c) && !c.starts_with("block:"))
                    .filter(|c| {
                        self.values.get(c).is_some_and(|v| {
                            v.group != Group::Dish && self.trade.need(town.kind, v.group) >= 1.3
                        })
                    })
                    .map(str::to_string)
                    .collect();
                let code = pick(&lacking).or_else(|| pick(&self.jobs.stone))?;
                (entry(Kind::Sell, scoped(site, &code), count), code, None)
            }
            JobKind::Hunt => {
                let names: Vec<String> = self.jobs.hunt.iter().map(|c| scoped(site, c)).collect();
                (
                    entry(Kind::Sell, names.join("|"), count),
                    self.jobs.hunt.join("|"),
                    None,
                )
            }
            JobKind::Deliver => {
                let to = *town
                    .neighbours
                    .get((roll % town.neighbours.len().max(1) as u64) as usize)?;
                (
                    entry(Kind::Deliver, scoped(to, "parcel"), 1),
                    String::new(),
                    Some(to),
                )
            }
            JobKind::Cook => {
                let dish = pick(&self.jobs.dishes)?;
                (entry(Kind::Cook, dish.clone(), count), dish, None)
            }
            JobKind::Fish => (entry(Kind::Catch, "any".into(), count), String::new(), None),
            JobKind::Mine => {
                let ore =
                    matches!(town.kind, SiteKind::Cliff | SiteKind::Cave) || roll.is_multiple_of(3);
                let list = if ore {
                    &self.jobs.ore
                } else {
                    &self.jobs.stone
                };
                (
                    entry(Kind::Gather, list.join("|"), count),
                    list.join("|"),
                    None,
                )
            }
            JobKind::Herd => {
                let good = pick(&self.jobs.herd)?;
                (entry(Kind::Collect, good.clone(), count), good, None)
            }
            JobKind::Harvest => {
                let crop = pick(&self.jobs.harvest)?;
                (entry(Kind::Harvest, crop.clone(), count), crop, None)
            }
            JobKind::Lamps => (
                entry(Kind::Light, scoped(site, "lamp"), count),
                String::new(),
                None,
            ),
            JobKind::Lost => (
                entry(Kind::Escort, scoped(site, "villager"), 1),
                String::new(),
                None,
            ),
            JobKind::Repair => (
                entry(Kind::Repair, scoped(site, "piece"), 1),
                String::new(),
                None,
            ),
            JobKind::Rockfall => {
                town.roads_to.first()?;
                (
                    entry(Kind::Clear, scoped(site, "road"), count),
                    String::new(),
                    None,
                )
            }
            JobKind::Escort => {
                let to = *town
                    .roads_to
                    .get((roll % town.roads_to.len().max(1) as u64) as usize)?;
                (
                    entry(Kind::Escort, scoped(to, "trader"), 1),
                    String::new(),
                    Some(to),
                )
            }
        };
        let unit = self.unit_value(kind, &thing);
        let reward =
            ((objective.count as f32 * unit as f32 * self.jobs.pay_share).round() as u64).max(1);
        Some(Job {
            id,
            site,
            kind,
            objectives: vec![objective],
            reward,
            to,
            thing,
        })
    }

    /// What one of a job's objective is worth: the value of what it asks
    /// for (the mean, for a choice), or the kind's unit value.
    fn unit_value(&self, kind: JobKind, thing: &str) -> u32 {
        let named: Vec<u32> = thing
            .split('|')
            .filter_map(|c| self.values.get(c))
            .map(|v| v.coins)
            .collect();
        if named.is_empty() {
            return self.jobs.unit_value.get(&kind).copied().unwrap_or(1);
        }
        (named.iter().sum::<u32>() as f32 / named.len() as f32)
            .round()
            .max(1.0) as u32
    }
}

fn entry(kind: Kind, what: String, count: u32) -> ObjectiveEntry {
    ObjectiveEntry {
        kind,
        what,
        count,
        label: None,
    }
}

/// Who in a town keeps its trade and who gives its jobs: indices into its
/// people. Every town with anyone in it has both (design 6: "each town has
/// a trader"): with no one in a trading role the first person trades, and
/// with no one in a giving role the trader gives the jobs too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keepers {
    pub trader: usize,
    pub giver: usize,
}

impl JobsConfig {
    /// The keepers among `roles`, a town's people's roles in order; `None`
    /// for a town of nobody.
    pub fn keepers(&self, roles: &[&str]) -> Option<Keepers> {
        if roles.is_empty() {
            return None;
        }
        let first = |wanted: &[String], skip: Option<usize>| {
            wanted.iter().find_map(|w| {
                roles
                    .iter()
                    .enumerate()
                    .find(|(i, r)| **r == w.as_str() && Some(*i) != skip)
                    .map(|(i, _)| i)
            })
        };
        let trader = first(&self.trader_roles, None).unwrap_or(0);
        let giver = first(&self.giver_roles, Some(trader)).unwrap_or(trader);
        Some(Keepers { trader, giver })
    }
}

/// The player's town jobs, as stored.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TakenJobs {
    /// The jobs taken and not yet handed in, in the order taken.
    pub taken: Vec<Job>,
    /// Their counts, kept by the starter missions' own [`Progress`].
    pub progress: Progress,
    /// Every job handed in, by id: never posted again.
    #[serde(default)]
    pub handed_in: BTreeSet<u64>,
}

/// Why taking or handing in a job was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobRefused {
    /// Already taken, or already handed in.
    Taken,
    /// Not done yet.
    NotDone,
    /// Handed in at the wrong town.
    WrongTown,
    /// No such job taken.
    NotTaken,
}

impl TakenJobs {
    /// The taken jobs as a chain the counting reads.
    pub fn chain(&self) -> Chain {
        Chain {
            missions: self.taken.iter().map(Job::mission).collect(),
        }
    }

    /// Every id the boards leave out: taken or handed in.
    pub fn gone(&self) -> BTreeSet<u64> {
        self.taken
            .iter()
            .map(|j| j.id)
            .chain(self.handed_in.iter().copied())
            .collect()
    }

    /// Take a job from a board.
    pub fn take(&mut self, job: Job) -> Result<(), JobRefused> {
        if self.gone().contains(&job.id) {
            return Err(JobRefused::Taken);
        }
        self.taken.push(job);
        Ok(())
    }

    /// Count one happening against every taken job, by the starter
    /// missions' function. The ids of jobs it finished.
    pub fn record(&mut self, happened: &Happened) -> (bool, Vec<u64>) {
        let chain = self.chain();
        let step = self.progress.record(&chain, happened, &|_| 0);
        let done = step
            .done
            .iter()
            .filter_map(|id| {
                self.taken
                    .iter()
                    .find(|j| &j.mission_id() == id)
                    .map(|j| j.id)
            })
            .collect();
        (step.changed, done)
    }

    /// Whether a taken job is done and waiting to be handed in.
    pub fn is_done(&self, id: u64) -> bool {
        self.taken
            .iter()
            .find(|j| j.id == id)
            .is_some_and(|j| self.progress.is_done(&j.mission_id()))
    }

    /// A taken job's counts so far, one per objective.
    pub fn counts(&self, id: u64) -> Vec<u32> {
        self.taken
            .iter()
            .find(|j| j.id == id)
            .map(|j| self.progress.counts(&j.mission()))
            .unwrap_or_default()
    }

    /// Give a taken job up (`town-missions-tree` design 9): it leaves the
    /// taken list with its counts, and is not handed in, so its board place
    /// shows it again while it keeps it. The job, for its own records.
    pub fn abandon(&mut self, id: u64) -> Result<Job, JobRefused> {
        let at = self
            .taken
            .iter()
            .position(|j| j.id == id)
            .ok_or(JobRefused::NotTaken)?;
        let job = self.taken.remove(at);
        let mission = job.mission_id();
        self.progress.done.remove(&mission);
        self.progress.counts.remove(&mission);
        self.progress.seen.remove(&mission);
        if self.progress.focus.as_deref() == Some(mission.as_str()) {
            self.progress.focus = None;
        }
        Ok(job)
    }

    /// Hand in a done job at `site`, its town: it leaves the taken list and
    /// its id is kept. The job, for its reward and goodwill.
    pub fn hand_in(&mut self, id: u64, site: u32) -> Result<Job, JobRefused> {
        let at = self
            .taken
            .iter()
            .position(|j| j.id == id)
            .ok_or(JobRefused::NotTaken)?;
        if self.taken[at].site != site {
            return Err(JobRefused::WrongTown);
        }
        if !self.is_done(id) {
            return Err(JobRefused::NotDone);
        }
        let job = self.taken.remove(at);
        self.progress.done.remove(&job.mission_id());
        self.handed_in.insert(job.id);
        Ok(job)
    }

    /// The done jobs waiting at `site`.
    pub fn ready_at(&self, site: u32) -> Vec<&Job> {
        self.taken
            .iter()
            .filter(|j| j.site == site && self.is_done(j.id))
            .collect()
    }
}
