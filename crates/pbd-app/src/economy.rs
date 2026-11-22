//! Coins, trade and town jobs in the game (`townsfolk` design 5 to 12).
//!
//! The rules are `pbd_core::economy`; this module feeds them and stores
//! their records through the save's durable path:
//!
//! - the purse (`purse`) and the taken jobs (`town_jobs`) are the player's,
//!   read when a world opens ([`Economy`]);
//! - a town's market (`town_market`) and goodwill (`goodwill`) are read
//!   from the save when a deal is made there, and written only when one is.
//!
//! **A deal's lines.** A trade writes the town's market and goodwill first,
//! then the purse with the slots after it in one `with` line, the purse and
//! the pack changing together. A torn write costs the last line
//! (`saves::format`), so the worst a crash mid-trade does is move a town's
//! price with neither the coins nor the pack changed. Handing a job in writes
//! the purse before the jobs: a torn write leaves the job to hand in again,
//! never paid and lost.
//!
//! **Who talks.** A [`Talker`] is anyone the interact key can talk to: the
//! townsfolk (`townsfolk` decision 1) carry one, naming their town, their
//! place in its people and their role. A town's trader and job giver are
//! picked from its people's roles (`JobsConfig::keepers`).

use crate::saves::WorldSave;
use crate::towns::Towns;
use crate::ui_kit::{Localized, tr};
use bevy::prelude::*;
use pbd_core::economy::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA, Purse, Reason};
use pbd_core::economy::goodwill::{GOODWILL_KIND, GOODWILL_SCHEMA, Goodwill};
use pbd_core::economy::jobs::{
    Board, JOBS_ID, JOBS_KIND, JOBS_SCHEMA, Job, JobKind, JobRefused, JobsConfig, Keepers,
    TakenJobs, Town, scoped,
};
use pbd_core::economy::trade::{
    Deal, MARKET_KIND, MARKET_SCHEMA, Market, Refused, TradeConfig, Trader,
};
use pbd_core::economy::values::Values;
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots};
use pbd_core::locale::tr_with;
use pbd_core::missions::{Happened, Kind};
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::sites::SiteKind;

/// The values, the trade rules and the job rules, read at startup.
#[derive(Resource, Clone, Debug)]
pub struct EconomyConfig {
    pub values: Values,
    pub trade: TradeConfig,
    pub jobs: JobsConfig,
}

impl EconomyConfig {
    /// Read `values.ron`, `trade.ron` and `jobs.ron`. A file that does not
    /// parse or validate stops the game with its path, as every config does.
    pub fn load() -> Self {
        let read = |name: &str| {
            let path = crate::config::config_dir().join(name);
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        };
        Self::parse(&read("values.ron"), &read("trade.ron"), &read("jobs.ron"))
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn parse(values: &str, trade: &str, jobs: &str) -> Result<Self, String> {
        Ok(Self {
            values: Values::parse(values)?,
            trade: TradeConfig::parse(trade)?,
            jobs: JobsConfig::parse(jobs)?,
        })
    }
}

/// The player's purse and taken jobs in the open world.
#[derive(Resource, Clone, Debug, Default)]
pub struct Economy {
    pub purse: Purse,
    pub jobs: TakenJobs,
    /// Which opening of a world was read: [`WorldSave::serial`].
    joined: Option<u64>,
    /// A job just done, and the seconds its line has left.
    pub notice: Option<(Localized, f32)>,
}

/// How long a "job done" line stays up, seconds.
pub const NOTICE_S: f32 = 4.0;

impl Economy {
    /// Read the purse and the jobs from a world just opened.
    pub fn join(&mut self, save: &WorldSave) {
        self.purse = purse_of(save);
        self.jobs = jobs_of(save);
        self.joined = Some(save.serial());
        self.notice = None;
    }

    /// Whether the open world has been read.
    pub fn joined(&self, save: &WorldSave) -> bool {
        self.joined == Some(save.serial())
    }
}

pub fn purse_of(save: &WorldSave) -> Purse {
    save.records
        .get(PURSE_KIND, PURSE_ID)
        .and_then(|r| r.read())
        .unwrap_or_default()
}

pub fn jobs_of(save: &WorldSave) -> TakenJobs {
    save.records
        .get(JOBS_KIND, JOBS_ID)
        .and_then(|r| r.read())
        .unwrap_or_default()
}

pub fn market_of(save: &WorldSave, site: u32) -> Market {
    save.records
        .get(MARKET_KIND, site as u64)
        .and_then(|r| r.read())
        .unwrap_or_default()
}

pub fn goodwill_of(save: &WorldSave, site: u32) -> Goodwill {
    save.records
        .get(GOODWILL_KIND, site as u64)
        .and_then(|r| r.read())
        .unwrap_or_default()
}

/// Someone the interact key talks to: their town (its site id), their place
/// in its people, and their role (`townsfolk` decision 1).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Talker {
    pub site: u32,
    pub index: u32,
    pub role: String,
}

/// A town as a deal reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct TownInfo {
    pub site: u32,
    pub kind: SiteKind,
    pub name: String,
    /// Other towns, nearest first: where a delivery may go.
    pub neighbours: Vec<u32>,
}

impl TownInfo {
    pub fn as_town(&self) -> Town<'_> {
        Town {
            site: self.site,
            kind: self.kind,
            neighbours: &self.neighbours,
        }
    }
}

/// How many neighbours a delivery chooses among.
pub const DELIVERY_NEIGHBOURS: usize = 3;

/// A held town's info, its neighbours the nearest other held towns.
pub fn town_info(towns: &Towns, site: u32) -> Option<TownInfo> {
    let here = towns.held.iter().find(|h| h.site.id == site)?;
    let mut others: Vec<(f32, u32)> = towns
        .held
        .iter()
        .filter(|h| h.site.id != site)
        .map(|h| (-h.site.direction.dot(here.site.direction), h.site.id))
        .collect();
    others.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    Some(TownInfo {
        site,
        kind: here.site.kind,
        name: here.site.name.clone(),
        neighbours: others
            .into_iter()
            .take(DELIVERY_NEIGHBOURS)
            .map(|(_, id)| id)
            .collect(),
    })
}

/// A town's name, by its site id: what a delivery's line names.
pub fn town_name(towns: &Towns, site: u32) -> String {
    towns
        .held
        .iter()
        .find(|h| h.site.id == site)
        .map(|h| h.site.name.clone())
        .unwrap_or_default()
}

/// A town's talkers in its people's order: its whole roster when its people
/// stand ([`crate::townsfolk::Townsfolk`]), so who keeps its trade does not
/// hang on which figures are drawn near the eye; else the talkers `drawn`
/// there (a town with no people laid).
pub fn roster(
    folk: Option<&crate::townsfolk::Townsfolk>,
    drawn: &[&Talker],
    site: u32,
) -> Vec<Talker> {
    let laid: Vec<Talker> = folk
        .map(|f| {
            f.people(site)
                .map(|p| Talker {
                    site,
                    index: p.id.index,
                    role: p.role.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    if !laid.is_empty() {
        return laid;
    }
    drawn
        .iter()
        .filter(|t| t.site == site)
        .map(|t| (*t).clone())
        .collect()
}

/// The keepers of a town's people, from the talkers standing there.
pub fn keepers(cfg: &JobsConfig, talkers: &[&Talker], site: u32) -> Option<Keepers> {
    let mut here: Vec<&Talker> = talkers.iter().copied().filter(|t| t.site == site).collect();
    here.sort_by_key(|t| t.index);
    let roles: Vec<&str> = here.iter().map(|t| t.role.as_str()).collect();
    let k = cfg.keepers(&roles)?;
    Some(Keepers {
        trader: here[k.trader].index as usize,
        giver: here[k.giver].index as usize,
    })
}

/// Why a deal was refused, in the words the panel shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Trade(Refused),
    Job(JobRefused),
    /// The pack has no room for a delivery's parcel.
    PackFull,
    /// No parcel for this town in the pack.
    NoParcel,
    /// The save refused the write.
    Unsaved,
}

impl Why {
    pub fn words(self) -> Localized {
        tr(match self {
            Why::Trade(Refused::Short) => "trade.short",
            Why::Trade(Refused::NotSold) => "trade.not_sold",
            Why::Trade(Refused::SoldOut) => "trade.sold_out",
            Why::Trade(Refused::PackFull) | Why::PackFull => "trade.pack_full",
            Why::Trade(Refused::NotWanted) => "trade.not_wanted",
            Why::Trade(Refused::NotHeld) => "trade.not_held",
            Why::Job(JobRefused::Taken) => "jobs.taken",
            Why::Job(JobRefused::NotDone) => "jobs.not_done",
            Why::Job(JobRefused::WrongTown) => "jobs.wrong_town",
            Why::Job(JobRefused::NotTaken) => "jobs.not_taken",
            Why::NoParcel => "jobs.no_parcel",
            Why::Unsaved => "trade.unsaved",
        })
    }
}

/// The fish ids of a body's roster, by index: how a fish in the pack is
/// named to the values.
pub fn fish_ids(fauna: &crate::config::FaunaConfig, body: &str) -> Vec<String> {
    fauna.0.roster(body).iter().map(|s| s.id.clone()).collect()
}

/// Everything a deal reads and writes.
pub struct Shop<'a> {
    pub save: &'a mut WorldSave,
    pub slots: &'a mut Slots,
    pub eco: &'a mut Economy,
    pub cfg: &'a EconomyConfig,
    pub fish_ids: &'a [String],
    /// World seconds now (the sun's clock).
    pub now_s: f64,
}

impl Shop<'_> {
    /// The trader of `town`, at its goodwill.
    pub fn trader<'b>(&'b self, town: &TownInfo) -> Trader<'b> {
        Trader {
            cfg: &self.cfg.trade,
            values: &self.cfg.values,
            kind: town.kind,
            goodwill: goodwill_of(self.save, town.site).points,
            fish_ids: self.fish_ids,
        }
    }

    /// The town's market now.
    pub fn market(&self, town: &TownInfo) -> Market {
        market_of(self.save, town.site).settled(self.now_s, &self.cfg.trade)
    }

    /// Buy one `code` from `town`'s trader.
    pub fn buy(&mut self, town: &TownInfo, code: &str) -> Result<Deal, Why> {
        self.deal(town, code, true)
    }

    /// Sell one `code` to `town`'s trader.
    pub fn sell(&mut self, town: &TownInfo, code: &str) -> Result<Deal, Why> {
        self.deal(town, code, false)
    }

    fn deal(&mut self, town: &TownInfo, code: &str, buying: bool) -> Result<Deal, Why> {
        let mut market = market_of(self.save, town.site);
        let mut purse = self.eco.purse;
        let mut slots = self.slots.clone();
        let deal = {
            let trader = self.trader(town);
            if buying {
                trader.buy(&mut market, &mut purse, &mut slots, code, self.now_s)
            } else {
                trader.sell(&mut market, &mut purse, &mut slots, code, self.now_s)
            }
        }
        .map_err(Why::Trade)?;
        let mut goodwill = goodwill_of(self.save, town.site);
        goodwill.raise(self.cfg.trade.goodwill.per_trade, &self.cfg.trade.goodwill);
        let mut jobs = self.eco.jobs.clone();
        let kind = if buying { Kind::Buy } else { Kind::Sell };
        let (counted, done) = jobs.record(&Happened::new(kind, scoped(town.site, code), 1));
        let mut lines = vec![
            StoredRecord::of(MARKET_KIND, town.site as u64, MARKET_SCHEMA, &market),
            StoredRecord::of(GOODWILL_KIND, town.site as u64, GOODWILL_SCHEMA, &goodwill),
        ];
        if counted {
            lines.push(StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs));
        }
        if self.save.store(&Author::Player(0), lines).is_none() {
            return Err(Why::Unsaved);
        }
        self.save.note_record_kinds(&[
            (MARKET_KIND, MARKET_SCHEMA),
            (GOODWILL_KIND, GOODWILL_SCHEMA),
            (JOBS_KIND, JOBS_SCHEMA),
        ]);
        let purse_line = StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
        if !self.save.store_with_hands(purse_line, PURSE_SCHEMA, &slots) {
            return Err(Why::Unsaved);
        }
        info!("{} at {}: {code}", deal.entry.log_line(), town.name);
        self.eco.purse = purse;
        self.eco.jobs = jobs;
        *self.slots = slots;
        self.notice_done(&done);
        Ok(deal)
    }

    /// The jobs posted on `town`'s board today.
    pub fn board(&self, town: &TownInfo) -> Vec<Job> {
        let board = Board {
            jobs: &self.cfg.jobs,
            trade: &self.cfg.trade,
            values: &self.cfg.values,
            seed: self.save.seed(),
        };
        let day = pbd_core::daylight::Clock {
            seconds: self.now_s,
        }
        .day();
        let goodwill = goodwill_of(self.save, town.site).points;
        board.posted(town.as_town(), day, goodwill, &self.eco.jobs.gone())
    }

    /// Take a job from `town`'s board. A delivery hands over its parcel, so
    /// it needs room in the pack.
    pub fn take(&mut self, job: Job) -> Result<(), Why> {
        let mut jobs = self.eco.jobs.clone();
        jobs.take(job.clone()).map_err(Why::Job)?;
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
        let mut slots = self.slots.clone();
        if job.kind == JobKind::Deliver {
            if slots.give(Item::Good(Good::PARCEL), 1) != 0 {
                return Err(Why::PackFull);
            }
            if !self.save.store_with_hands(record, JOBS_SCHEMA, &slots) {
                return Err(Why::Unsaved);
            }
        } else if self.save.store(&Author::Player(0), vec![record]).is_some() {
            self.save.note_record_kinds(&[(JOBS_KIND, JOBS_SCHEMA)]);
        } else {
            return Err(Why::Unsaved);
        }
        info!("job {} taken at site {}: {:?}", job.id, job.site, job.kind);
        self.eco.jobs = jobs;
        *self.slots = slots;
        Ok(())
    }

    /// Hand a done job in at `town`: its coins into the purse and the town's
    /// goodwill up.
    pub fn hand_in(&mut self, town: &TownInfo, id: u64) -> Result<Job, Why> {
        let mut jobs = self.eco.jobs.clone();
        let job = jobs.hand_in(id, town.site).map_err(Why::Job)?;
        let mut purse = self.eco.purse;
        let entry = purse
            .change(Reason::Mission, job.reward)
            .expect("an earning is never refused");
        let mut goodwill = goodwill_of(self.save, town.site);
        goodwill.raise(self.cfg.trade.goodwill.per_job, &self.cfg.trade.goodwill);
        let lines = vec![
            StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse),
            StoredRecord::of(GOODWILL_KIND, town.site as u64, GOODWILL_SCHEMA, &goodwill),
            StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs),
        ];
        if self.save.store(&Author::Player(0), lines).is_none() {
            return Err(Why::Unsaved);
        }
        self.save.note_record_kinds(&[
            (PURSE_KIND, PURSE_SCHEMA),
            (GOODWILL_KIND, GOODWILL_SCHEMA),
            (JOBS_KIND, JOBS_SCHEMA),
        ]);
        info!("{} for job {} at {}", entry.log_line(), job.id, town.name);
        self.eco.purse = purse;
        self.eco.jobs = jobs;
        Ok(job)
    }

    /// Hand `town`'s people a parcel for every delivery taken to it and not
    /// yet done: the parcels leave the pack and the deliveries count. How
    /// many were handed over.
    pub fn deliver(&mut self, town: &TownInfo) -> Result<u32, Why> {
        let parcel = Item::Good(Good::PARCEL);
        let owed: Vec<u64> = self
            .eco
            .jobs
            .taken
            .iter()
            .filter(|j| j.kind == JobKind::Deliver && j.to == Some(town.site))
            .filter(|j| !self.eco.jobs.is_done(j.id))
            .map(|j| j.id)
            .collect();
        if owed.is_empty() {
            return Ok(0);
        }
        let mut slots = self.slots.clone();
        let mut jobs = self.eco.jobs.clone();
        let mut handed = 0;
        let mut done = Vec::new();
        for _ in &owed {
            if !slots.take_item(parcel, 1) {
                break;
            }
            let (_, d) = jobs.record(&Happened::new(
                Kind::Deliver,
                scoped(town.site, "parcel"),
                1,
            ));
            done.extend(d);
            handed += 1;
        }
        if handed == 0 {
            return Err(Why::NoParcel);
        }
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
        if !self.save.store_with_hands(record, JOBS_SCHEMA, &slots) {
            return Err(Why::Unsaved);
        }
        info!("{handed} parcel(s) handed over at {}", town.name);
        self.eco.jobs = jobs;
        *self.slots = slots;
        self.notice_done(&done);
        Ok(handed)
    }

    fn notice_done(&mut self, done: &[u64]) {
        if !done.is_empty() {
            self.eco.notice = Some((tr("jobs.done_notice"), NOTICE_S));
        }
    }
}

/// Join a newly opened world, count this frame's happenings against the
/// taken jobs, and store them when they move. After the starter missions'
/// counting, which reads the messages and leaves them in
/// [`crate::missions::Happenings`].
pub fn count_jobs(
    mut eco: ResMut<Economy>,
    save: Option<ResMut<WorldSave>>,
    heard: Res<crate::missions::Happenings>,
    time: Res<Time>,
) {
    // Touched only when it moves, so the panels and the count repaint only
    // on a change.
    if eco.notice.is_some() {
        let left = eco.notice.as_ref().map_or(0.0, |n| n.1) - time.delta_secs();
        if left <= 0.0 {
            eco.notice = None;
        } else if let Some(n) = eco.bypass_change_detection().notice.as_mut() {
            n.1 = left;
        }
    }
    let Some(mut save) = save else {
        return;
    };
    if !eco.joined(&save) {
        eco.join(&save);
    }
    if heard.0.is_empty() || eco.jobs.taken.is_empty() {
        return;
    }
    let mut changed = false;
    let mut done = Vec::new();
    for happened in &heard.0 {
        let (moved, d) = eco.jobs.record(happened);
        changed |= moved;
        done.extend(d);
    }
    if !done.is_empty() {
        eco.notice = Some((tr("jobs.done_notice"), NOTICE_S));
    }
    if changed {
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &eco.jobs);
        if save.store(&Author::Player(0), vec![record]).is_some() {
            save.note_record_kinds(&[(JOBS_KIND, JOBS_SCHEMA)]);
        } else {
            warn!("town jobs not saved: the save writer has failed");
        }
    }
}

/// A taken job's line, as the board and the talk panel show it: what to do,
/// how far along, and the reward.
pub fn job_line(job: &Job, counts: &[u32], towns: &Towns) -> Localized {
    let o = &job.objectives[0];
    let have = counts.first().copied().unwrap_or(0);
    let thing = job_thing(job);
    let to = job.to.map(|t| town_name(towns, t)).unwrap_or_default();
    let key = format!("jobs.kind.{}", job.kind.code());
    tr_with(
        &key,
        &[
            ("count", &o.count),
            ("have", &have),
            ("thing", &thing),
            ("town", &to),
        ],
    )
}

/// What a job asks for, named: a good, or a choice of goods.
pub fn job_thing(job: &Job) -> Localized {
    let first = job.thing.split('|').next().unwrap_or_default();
    if first.is_empty() {
        return Localized::empty();
    }
    pbd_core::economy::values::item_of(first, &[]).map_or_else(Localized::empty, Item::name)
}

/// The economy's config, state and counting.
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<EconomyConfig>() {
            app.insert_resource(EconomyConfig::load());
        }
        app.init_resource::<Economy>().add_systems(
            PostUpdate,
            count_jobs
                .after(crate::missions::MissionsSet)
                .run_if(crate::world_open),
        );
    }
}

#[cfg(test)]
mod tests;
