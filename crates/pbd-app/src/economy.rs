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
use pbd_core::economy::derive::{Deriver, makings};
use pbd_core::economy::goodwill::{GOODWILL_KIND, GOODWILL_SCHEMA, Goodwill};
use pbd_core::economy::jobs::{
    Board, JOBS_ID, JOBS_KIND, JOBS_SCHEMA, Job, JobKind, JobRefused, JobsConfig, Keepers,
    TakenJobs, Town, scoped,
};
use pbd_core::economy::market::{
    WORLD_MARKET_ID, WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA, WorldMarket, balance,
};
use pbd_core::economy::trade::{
    Barter, Basket, Deal, MARKET_KIND, MARKET_SCHEMA, Market, Refused, TradeConfig, Trader,
};
use pbd_core::economy::values::Values;
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots};
use pbd_core::locale::tr_with;
use pbd_core::missions::{Happened, Kind};
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::sites::SiteKind;

/// The values, the trade rules and the job rules, read at startup.
/// `values` are the derived values at rest (`townsfolk` design 37a): the
/// typed rows of raw materials, every made good worked out from the recipe
/// book; `deriver` prices them again with the market's index.
#[derive(Resource, Clone, Debug)]
pub struct EconomyConfig {
    pub values: Values,
    pub deriver: Deriver,
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

    /// Parse the three files, deriving the values from the game's recipe
    /// book (`recipes.ron` and `cooking.ron`, [`crate::config::recipe_book`]).
    pub fn parse(values: &str, trade: &str, jobs: &str) -> Result<Self, String> {
        let (book, _) = crate::config::recipe_book();
        Self::parse_with_book(values, trade, jobs, &book.0)
    }

    pub fn parse_with_book(
        values: &str,
        trade: &str,
        jobs: &str,
        book: &pbd_core::station::RecipeBook,
    ) -> Result<Self, String> {
        let base = Values::parse(values)?;
        let trade = TradeConfig::parse(trade)?;
        let overrides = base.overrides().to_vec();
        let deriver = Deriver::new(
            base,
            makings(book),
            trade.labour_per_s,
            trade.craft_margin,
            &overrides,
        )?;
        Ok(Self {
            values: deriver.derive().values,
            deriver,
            trade,
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
    /// What the town did this frame that only the mission trees count
    /// (`town-missions-tree` design 4): a talk, a sale or buy, a job handed
    /// in, a parcel handed over, a town's goodwill reaching a step. The
    /// taken jobs counted these already; [`crate::town_missions`] takes them.
    pub raised: Vec<Happened>,
    /// Lost villagers whose job was abandoned, to stand back at their posts
    /// (`town-missions-tree` design 9): what `special::lost_villagers` lets go.
    pub let_go: Vec<pbd_core::townsfolk::PersonId>,
    /// The world's market as last settled (`townsfolk` design 37a), and the
    /// world opening and game day it was settled for.
    pub market: WorldMarket,
    market_for: Option<(u64, u64)>,
    /// Every town's kind, what the market's daily making and using reads;
    /// kept by [`settle_market`] from the towns the world holds.
    pub town_kinds: Vec<SiteKind>,
    /// The derived values with the market's index folded in: what a trader
    /// prices by. `None` until the market is first settled.
    pub prices: Option<Values>,
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
        self.market_for = None;
        self.prices = None;
    }

    /// Settle the world's market to `today` (design 37a): read from the save
    /// when this opening has not been settled, stepped by the days since,
    /// and the prices worked out again. The record to write comes back when
    /// a day moved a row that the save holds or would now hold.
    pub fn settle_market(
        &mut self,
        save: &WorldSave,
        cfg: &EconomyConfig,
        today: u64,
    ) -> Option<StoredRecord> {
        let key = (save.serial(), today);
        if self.market_for == Some(key) {
            return None;
        }
        let stored = if self.market_for.is_some_and(|(s, _)| s == save.serial()) {
            self.market.clone()
        } else {
            world_market_of(save, today)
        };
        let kinds = &self.town_kinds;
        let settled = stored.settled(today, &cfg.trade.market, &cfg.values, |c| {
            balance(&cfg.trade, &cfg.values, kinds, c)
        });
        let write = settled.moved_from(&stored).then(|| {
            StoredRecord::of(
                WORLD_MARKET_KIND,
                WORLD_MARKET_ID,
                WORLD_MARKET_SCHEMA,
                &settled,
            )
        });
        self.market = settled;
        self.market_for = Some(key);
        self.reprice(cfg);
        write
    }

    /// Work the prices out again from the market's index.
    pub fn reprice(&mut self, cfg: &EconomyConfig) {
        let market = &self.market;
        self.prices = Some(cfg.deriver.priced(|c| market.index(c)).values);
    }
}

/// Who writes a day's settle of the market: the world, never the player
/// (`world-persistence`: every change names its author).
pub fn market_author() -> Author {
    Author::World {
        process: "market".into(),
        kind: WORLD_MARKET_KIND.into(),
        id: WORLD_MARKET_ID,
    }
}

/// The world's market in `save`; a world with none reads at rest, settled
/// to `today` (an old save opens with every index at 1.0).
pub fn world_market_of(save: &WorldSave, today: u64) -> WorldMarket {
    save.records
        .get(WORLD_MARKET_KIND, WORLD_MARKET_ID)
        .and_then(|r| r.read())
        .unwrap_or_else(|| WorldMarket::fresh(today))
}

impl Economy {
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
    /// The towns a road of the world's joins to this one (`townsfolk`
    /// design 46): where a road job goes. Filled from the save by
    /// [`roads_to`] where a board is made.
    pub roads_to: Vec<u32>,
}

impl TownInfo {
    pub fn as_town(&self) -> Town<'_> {
        Town {
            site: self.site,
            kind: self.kind,
            neighbours: &self.neighbours,
            roads_to: &self.roads_to,
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
        roads_to: Vec::new(),
    })
}

/// The towns a stored road of the world's joins to `site`, by site id, in
/// the roads' order (`townsfolk` design 46).
pub fn roads_to(save: &WorldSave, site: u32) -> Vec<u32> {
    let mut out = Vec::new();
    for road in pbd_core::roads::stored(&save.records) {
        let other = if road.from_site == site {
            road.to_site
        } else if road.to_site == site {
            road.from_site
        } else {
            continue;
        };
        if !road.cells.is_empty() && !out.contains(&other) {
            out.push(other);
        }
    }
    out
}

/// A town's name, by its site id: what a delivery's line names.
/// Where a town's job giver keeps their post, as a direction from the
/// planet's centre: what the compass and the map mark as the place a job is
/// taken (`job-waypoints` design 7). `None` while its people are not laid.
pub fn giver_direction(
    cfg: &EconomyConfig,
    folk: &crate::townsfolk::Townsfolk,
    site: u32,
) -> Option<Vec3> {
    let mut people: Vec<_> = folk.people(site).collect();
    people.sort_by_key(|p| p.id.index);
    let roles: Vec<&str> = people.iter().map(|p| p.role.as_str()).collect();
    let k = cfg.jobs.keepers(&roles)?;
    people.get(k.giver).and_then(|p| p.at.try_normalize())
}

/// The nearest held town within `within_m` of `up`, and its job giver's
/// direction (`job-waypoints` design 7).
pub fn near_giver(
    cfg: &EconomyConfig,
    folk: &crate::townsfolk::Townsfolk,
    towns: &Towns,
    up: Vec3,
    within_m: f32,
) -> Option<Vec3> {
    let site = towns
        .held
        .iter()
        .map(|h| {
            (
                h.site.id,
                h.site.direction.angle_between(up) * crate::planet::PLANET_RADIUS,
            )
        })
        .filter(|(_, d)| *d <= within_m)
        .min_by(|a, b| a.1.total_cmp(&b.1))?
        .0;
    giver_direction(cfg, folk, site)
}

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
            Why::Trade(Refused::Empty) => "trade.empty",
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
    /// Today's game day.
    fn today(&self) -> u64 {
        pbd_core::daylight::Clock {
            seconds: self.now_s,
        }
        .day()
    }

    /// The world's market settled to today; the line to write if a day
    /// moved it.
    fn settle(&mut self) -> Option<StoredRecord> {
        let today = self.today();
        self.eco.settle_market(self.save, self.cfg, today)
    }

    /// Settle the market before a panel prices anything (design 37a).
    pub fn ready(&mut self) {
        if let Some(line) = self.settle()
            && self.save.store(&market_author(), vec![line]).is_some()
        {
            self.save
                .note_record_kinds(&[(WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA)]);
        }
    }

    /// The trader of `town`, at its goodwill, pricing by the market.
    pub fn trader<'b>(&'b self, town: &TownInfo) -> Trader<'b> {
        Trader {
            cfg: &self.cfg.trade,
            values: self.eco.prices.as_ref().unwrap_or(&self.cfg.values),
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
        self.ready();
        let mut market = market_of(self.save, town.site);
        let mut purse = self.eco.purse.clone();
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
        let stepped = self.raise_goodwill(&mut goodwill, self.cfg.trade.goodwill.per_trade);
        let mut jobs = self.eco.jobs.clone();
        let kind = if buying { Kind::Buy } else { Kind::Sell };
        let (counted, done) = jobs.record(&Happened::new(kind, scoped(town.site, code), 1));
        let mut world = self.eco.market.clone();
        world.traded(code, 1, !buying, &self.cfg.trade.market);
        let mut lines = vec![
            StoredRecord::of(MARKET_KIND, town.site as u64, MARKET_SCHEMA, &market),
            StoredRecord::of(GOODWILL_KIND, town.site as u64, GOODWILL_SCHEMA, &goodwill),
            StoredRecord::of(
                WORLD_MARKET_KIND,
                WORLD_MARKET_ID,
                WORLD_MARKET_SCHEMA,
                &world,
            ),
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
            (WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA),
            (JOBS_KIND, JOBS_SCHEMA),
        ]);
        self.eco.market = world;
        self.eco.reprice(self.cfg);
        let purse_line = StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
        if !self.save.store_with_hands(purse_line, PURSE_SCHEMA, &slots) {
            return Err(Why::Unsaved);
        }
        info!("{} at {}: {code}", deal.entry.log_line(), town.name);
        self.eco.purse = purse;
        self.eco.jobs = jobs;
        *self.slots = slots;
        self.notice_done(&done);
        self.eco
            .raised
            .push(Happened::new(kind, scoped(town.site, code), 1));
        self.note_step(town, stepped);
        Ok(deal)
    }

    /// What `basket` comes to at `town` now.
    pub fn tally(&mut self, town: &TownInfo, basket: &Basket) -> pbd_core::economy::trade::Tally {
        self.ready();
        let market = self.market(town);
        self.trader(town).tally(&market, basket)
    }

    /// Settle a basket deal at `town` (`townsfolk` design 37a, task 11.3):
    /// goods and coins both ways, all at once. Its lines are a single
    /// trade's: the town market, goodwill (one step a deal), the world
    /// market and the jobs, then the purse with the pack in one line.
    pub fn barter(&mut self, town: &TownInfo, basket: &Basket) -> Result<Barter, Why> {
        self.ready();
        let mut market = market_of(self.save, town.site);
        let mut purse = self.eco.purse.clone();
        let mut slots = self.slots.clone();
        let barter = self
            .trader(town)
            .deal(&mut market, &mut purse, &mut slots, basket, self.now_s)
            .map_err(Why::Trade)?;
        let mut goodwill = goodwill_of(self.save, town.site);
        let stepped = self.raise_goodwill(&mut goodwill, self.cfg.trade.goodwill.per_trade);
        let mut jobs = self.eco.jobs.clone();
        let mut world = self.eco.market.clone();
        let mut happened = Vec::new();
        let mut counted = false;
        let mut done = Vec::new();
        for (list, kind, sold) in [
            (&basket.take, Kind::Buy, false),
            (&basket.give, Kind::Sell, true),
        ] {
            for (code, n) in list.iter().filter(|(_, n)| *n > 0) {
                world.traded(code, *n, sold, &self.cfg.trade.market);
                let h = Happened::new(kind, scoped(town.site, code), *n);
                let (c, d) = jobs.record(&h);
                counted |= c;
                done.extend(d);
                happened.push(h);
            }
        }
        let mut lines = vec![
            StoredRecord::of(MARKET_KIND, town.site as u64, MARKET_SCHEMA, &market),
            StoredRecord::of(GOODWILL_KIND, town.site as u64, GOODWILL_SCHEMA, &goodwill),
            StoredRecord::of(
                WORLD_MARKET_KIND,
                WORLD_MARKET_ID,
                WORLD_MARKET_SCHEMA,
                &world,
            ),
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
            (WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA),
            (JOBS_KIND, JOBS_SCHEMA),
        ]);
        let purse_line = StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
        if !self.save.store_with_hands(purse_line, PURSE_SCHEMA, &slots) {
            return Err(Why::Unsaved);
        }
        match &barter.entry {
            Some(entry) => info!("{} at {}: a basket deal", entry.log_line(), town.name),
            None => info!("an even basket deal at {}", town.name),
        }
        self.eco.purse = purse;
        self.eco.jobs = jobs;
        self.eco.market = world;
        self.eco.reprice(self.cfg);
        *self.slots = slots;
        self.notice_done(&done);
        self.eco.raised.extend(happened);
        self.note_step(town, stepped);
        Ok(barter)
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
        let hour = pbd_core::daylight::Clock {
            seconds: self.now_s,
        }
        .hour();
        let town = TownInfo {
            roads_to: roads_to(self.save, town.site),
            ..town.clone()
        };
        board
            .posted(town.as_town(), day, goodwill, &self.eco.jobs.gone())
            .into_iter()
            .filter(|j| pbd_core::economy::jobs::shown_at(j, hour))
            .collect()
    }

    /// Give a taken job up (`town-missions-tree` design 9): off the taken
    /// list, back on its board, no coins and no goodwill. A delivery's
    /// parcel leaves the pack if it is there; a lost villager's record is
    /// cleared and they go back to their post. The jobs line first.
    pub fn abandon(&mut self, id: u64) -> Result<Job, Why> {
        let mut jobs = self.eco.jobs.clone();
        // A parcel still with the trader was never in the pack.
        let carried = !jobs.is_waiting(id);
        let job = jobs.abandon(id).map_err(Why::Job)?;
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
        let mut slots = self.slots.clone();
        if job.kind == JobKind::Deliver && carried && slots.take_item(Item::Good(Good::PARCEL), 1) {
            if !self.save.store_with_hands(record, JOBS_SCHEMA, &slots) {
                return Err(Why::Unsaved);
            }
        } else if self.save.store(&Author::Player(0), vec![record]).is_none() {
            return Err(Why::Unsaved);
        }
        if job.kind == JobKind::Lost
            && let Some(lost) = special::lost_of(self.save, job.id)
        {
            let home = special::home_record(job.id);
            if self.save.store(&Author::Player(0), vec![home]).is_some() {
                self.eco.let_go.push(pbd_core::townsfolk::PersonId {
                    site: lost.site,
                    index: lost.person,
                });
            }
        }
        // A road trader given up goes home (`townsfolk` design 46).
        if job.kind == JobKind::Escort
            && let Some(escort) = road_jobs::escort_of(self.save, job.id)
        {
            let home = road_jobs::escort_record(job.id, None);
            if self.save.store(&Author::Player(0), vec![home]).is_some() {
                self.eco.let_go.push(pbd_core::townsfolk::PersonId {
                    site: escort.site,
                    index: escort.person,
                });
            }
        }
        info!(
            "job {} abandoned at site {}: {:?}",
            job.id, job.site, job.kind
        );
        self.eco.jobs = jobs;
        *self.slots = slots;
        Ok(job)
    }

    /// Take a job from `town`'s board. A delivery's parcel waits with the
    /// town's trader until picked up (`townsfolk` design 47).
    pub fn take(&mut self, job: Job) -> Result<(), Why> {
        let mut jobs = self.eco.jobs.clone();
        jobs.take(job.clone()).map_err(Why::Job)?;
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
        if self.save.store(&Author::Player(0), vec![record]).is_some() {
            self.save.note_record_kinds(&[(JOBS_KIND, JOBS_SCHEMA)]);
        } else {
            return Err(Why::Unsaved);
        }
        info!("job {} taken at site {}: {:?}", job.id, job.site, job.kind);
        self.eco.jobs = jobs;
        Ok(())
    }

    /// The parcels waiting with `town`'s trader handed to the player, one
    /// pack slot each, in the same line as the jobs (`townsfolk` design 47).
    /// How many; a full pack leaves them waiting.
    pub fn pick_up(&mut self, town: &TownInfo) -> Result<u32, Why> {
        let waiting = self.eco.jobs.waiting_at(town.site);
        if waiting.is_empty() {
            return Ok(0);
        }
        let mut jobs = self.eco.jobs.clone();
        let mut slots = self.slots.clone();
        let mut given = 0;
        for id in waiting {
            if slots.give(Item::Good(Good::PARCEL), 1) != 0 {
                break;
            }
            jobs.pick_up(id);
            given += 1;
        }
        if given == 0 {
            return Err(Why::PackFull);
        }
        let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
        if !self.save.store_with_hands(record, JOBS_SCHEMA, &slots) {
            return Err(Why::Unsaved);
        }
        info!("{given} parcel(s) picked up at {}", town.name);
        self.eco.jobs = jobs;
        *self.slots = slots;
        Ok(given)
    }

    /// Hand a done job in at `town`: its coins into the purse and the town's
    /// goodwill up.
    pub fn hand_in(&mut self, town: &TownInfo, id: u64) -> Result<Job, Why> {
        let mut jobs = self.eco.jobs.clone();
        let job = jobs.hand_in(id, town.site).map_err(Why::Job)?;
        let mut purse = self.eco.purse.clone();
        let entry = purse
            .change(Reason::Mission, job.reward)
            .expect("an earning is never refused");
        let mut goodwill = goodwill_of(self.save, town.site);
        let stepped = self.raise_goodwill(&mut goodwill, self.cfg.trade.goodwill.per_job);
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
        self.eco
            .raised
            .push(Happened::new(Kind::Job, job.kind.code(), 1));
        self.note_step(town, stepped);
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
            .filter(|j| !self.eco.jobs.is_done(j.id) && !self.eco.jobs.is_waiting(j.id))
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
        self.eco.raised.push(Happened::new(
            Kind::Deliver,
            scoped(town.site, "parcel"),
            handed,
        ));
        Ok(handed)
    }

    /// A townsperson talked to: counted once a person by the mission trees.
    pub fn talked(&mut self, talker: &Talker) {
        self.eco.raised.push(Happened::new(
            Kind::Talk,
            scoped(talker.site, &talker.index.to_string()),
            1,
        ));
    }

    /// Raise a town's goodwill by `by`: whether it reached its next step.
    fn raise_goodwill(&self, goodwill: &mut Goodwill, by: u32) -> bool {
        let cfg = &self.cfg.trade.goodwill;
        let before = cfg.level(goodwill.points);
        goodwill.raise(by, cfg);
        cfg.level(goodwill.points) > before
    }

    /// A town's goodwill stepped up, for the mission trees.
    fn note_step(&mut self, town: &TownInfo, stepped: bool) {
        if stepped {
            self.eco
                .raised
                .push(Happened::new(Kind::Goodwill, scoped(town.site, "step"), 1));
        }
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

/// Make a taken job the active mission, or none (playtest-fixes design 2),
/// and store the jobs record. Whether it moved and was stored.
pub fn track_job(eco: &mut Economy, save: &mut WorldSave, mission: Option<String>) -> bool {
    if eco.jobs.progress.focus == mission {
        return false;
    }
    let mut jobs = eco.jobs.clone();
    jobs.progress.focus = mission;
    let record = StoredRecord::of(JOBS_KIND, JOBS_ID, JOBS_SCHEMA, &jobs);
    if save.store(&Author::Player(0), vec![record]).is_none() {
        warn!("the tracked job was not saved: the save writer has failed");
        return false;
    }
    save.note_record_kinds(&[(JOBS_KIND, JOBS_SCHEMA)]);
    eco.jobs = jobs;
    true
}

/// The tracked job, if a taken one is.
pub fn tracked_job(eco: &Economy) -> Option<&Job> {
    let focus = eco.jobs.progress.focus.as_deref()?;
    eco.jobs.taken.iter().find(|j| j.mission_id() == focus)
}

/// A taken job as the HUD tracker names it (playtest-fixes design 2): what
/// to do next, without counts. A delivery still with the trader is to be
/// picked up; a done job is to be handed in where it was taken.
pub fn job_title(job: &Job, jobs: &TakenJobs, towns: &Towns) -> Localized {
    if jobs.is_done(job.id) {
        return tr_with(
            "jobs.track_hand_in",
            &[("town", &town_name(towns, job.site))],
        );
    }
    if jobs.is_waiting(job.id) {
        return tr("jobs.track_pick_up");
    }
    let o = &job.objectives[0];
    let thing = job_thing(job);
    let to = job.to.map(|t| town_name(towns, t)).unwrap_or_default();
    tr_with(
        &format!("jobs.name.{}", job.kind.code()),
        &[("count", &o.count), ("thing", &thing), ("town", &to)],
    )
}

/// Where each picked-up parcel is bound, as directions from the planet's
/// centre: the job's town, what the compass and the map mark (`townsfolk`
/// design 47).
pub fn parcel_directions(eco: &Economy, towns: &Towns) -> Vec<Vec3> {
    eco.jobs
        .carried()
        .filter_map(|j| j.to)
        .filter_map(|to| towns.held.iter().find(|h| h.site.id == to))
        .map(|h| h.site.direction)
        .collect()
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

/// Keep the world's market settled to today (`townsfolk` design 37a): the
/// towns' kinds read from the towns the world holds, the market stepped at
/// each day boundary, and its record written when a day moved a row.
/// Waits for the towns, so a day's making and using reads every town.
pub fn settle_market(
    mut eco: ResMut<Economy>,
    save: Option<ResMut<WorldSave>>,
    towns: Res<Towns>,
    sun: Res<crate::sky::Sun>,
    cfg: Res<EconomyConfig>,
) {
    let Some(mut save) = save else {
        return;
    };
    if towns.held.is_empty() || !eco.joined(&save) {
        return;
    }
    if eco.town_kinds.len() != towns.held.len() {
        eco.town_kinds = towns.held.iter().map(|h| h.site.kind).collect();
    }
    let today = sun.clock.day();
    if eco.market_for == Some((save.serial(), today)) {
        return;
    }
    if let Some(line) = eco.settle_market(&save, &cfg, today) {
        if save.store(&market_author(), vec![line]).is_some() {
            save.note_record_kinds(&[(WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA)]);
        } else {
            warn!("the market was not saved: the save writer has failed");
        }
    }
}

/// The lost villager and repair jobs (`townsfolk` design 28 and 29).
pub mod road_jobs;
pub mod special;

/// The economy's config, state and counting.
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<EconomyConfig>() {
            app.insert_resource(EconomyConfig::load());
        }
        if !app.is_plugin_added::<crate::town_missions::TownMissionsPlugin>() {
            app.add_plugins(crate::town_missions::TownMissionsPlugin);
        }
        app.init_resource::<Economy>()
            .add_systems(
                PostUpdate,
                count_jobs
                    .after(crate::missions::MissionsSet)
                    .run_if(crate::world_open),
            )
            .add_systems(
                Update,
                (
                    settle_market.run_if(resource_exists::<crate::sky::Sun>),
                    special::lost_villagers,
                    special::repairs,
                    road_jobs::rockfalls,
                    road_jobs::escorts,
                    special::use_mend,
                    special::use_lamp,
                )
                    .run_if(crate::world_open)
                    .run_if(resource_exists::<Towns>),
            )
            .init_resource::<special::LitLamps>()
            .add_systems(
                PostUpdate,
                (special::offer_mend, special::offer_lamp)
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<Towns>),
            );
    }
}

#[cfg(test)]
mod barter_tests;
#[cfg(test)]
mod tests;
