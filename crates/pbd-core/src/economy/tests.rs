use super::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA, Purse, Reason, Refused as PurseRefused};
use super::follow::{self, GAP_M};
use super::goodwill::{GOODWILL_KIND, GOODWILL_SCHEMA, Goodwill};
use super::jobs::{Board, Job, JobKind, JobRefused, JobsConfig, TakenJobs, Town, scoped};
use super::trade::{MARKET_KIND, MARKET_SCHEMA, Market, Refused, TradeConfig, Trader};
use super::values::{Values, item_of, known_codes, trade_code};
use crate::daylight::DAY_S;
use crate::goods::Good;
use crate::inventory::{CARRIED, Item, Slots};
use crate::missions::{Happened, Kind};
use crate::records::Record;
use crate::sites::SiteKind;
use glam::DVec3;
use std::collections::BTreeSet;

const VALUES: &str = include_str!("../../../../assets/config/values.ron");
const TRADE: &str = include_str!("../../../../assets/config/trade.ron");
const JOBS: &str = include_str!("../../../../assets/config/jobs.ron");

fn values() -> Values {
    Values::parse(VALUES).unwrap()
}

fn trade() -> TradeConfig {
    TradeConfig::parse(TRADE).unwrap()
}

fn jobs() -> JobsConfig {
    JobsConfig::parse(JOBS).unwrap()
}

/// The home body's fish ids, as `fauna.ron` lists them.
fn fish_ids() -> Vec<String> {
    let fauna: crate::fauna::FaunaSettings =
        ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap();
    fauna
        .roster(crate::fauna::HOME_BODY)
        .iter()
        .map(|s| s.id.clone())
        .collect()
}

/// Every fish id of every body.
fn all_fish_ids() -> Vec<String> {
    let fauna: crate::fauna::FaunaSettings =
        ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap();
    let mut ids: Vec<String> = fauna
        .bodies
        .values()
        .flatten()
        .map(|s| s.id.clone())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

// ---- coins (5.1)

#[test]
fn a_spend_past_the_purse_is_refused_and_changes_nothing() {
    let mut purse = Purse::new(4);
    assert_eq!(
        purse.change(Reason::Bought, 6),
        Err(PurseRefused::Short { have: 4, need: 6 })
    );
    assert_eq!(purse.coins, 4);
    let entry = purse.change(Reason::Bought, 4).unwrap();
    assert_eq!((entry.delta, entry.after, purse.coins), (-4, 0, 0));
    assert!(
        purse.change(Reason::Bought, 1).is_err(),
        "never below nought"
    );
}

#[test]
fn the_purse_survives_a_save_and_load() {
    let mut purse = Purse::default();
    purse.change(Reason::Mission, 37).unwrap();
    let record = Record::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
    let line = record.line(&crate::records::Author::Player(0));
    let (_, back) = Record::parse_line(&line).unwrap();
    assert_eq!(back.read::<Purse>(), Some(Purse::new(37)));
}

#[test]
fn every_reason_is_logged() {
    let mut purse = Purse::new(10);
    for reason in Reason::ALL {
        let entry = purse.change(reason, 2).unwrap();
        assert_eq!(entry.reason, reason);
        let line = entry.log_line();
        assert!(line.contains(reason.code()), "{line}");
        assert!(line.contains(&entry.after.to_string()), "{line}");
    }
}

#[test]
fn a_full_pack_still_takes_coins() {
    let mut slots = Slots::new();
    for _ in 0..CARRIED {
        slots.give(Item::Good(Good::LOG), 99);
    }
    assert_eq!(
        slots.give(Item::Good(Good::WHEAT), 1),
        1,
        "the pack is full"
    );
    let mut purse = Purse::default();
    purse.change(Reason::Mission, 20).unwrap();
    assert_eq!(purse.coins, 20);
}

// ---- values (5.3)

/// The codes `values.ron` shipped with (2026-11-21). The file only grows:
/// every one of these keeps its row.
const SHIPPED: &[&str] = &[
    "wheat-seed",
    "wheat",
    "turnip-seed",
    "turnip",
    "frost-kale-seed",
    "frost-kale",
    "chili-seed",
    "chili",
    "coal",
    "tin-ore",
    "copper-ingot",
    "iron-ingot",
    "bronze-ingot",
    "fence",
    "gate",
    "egg",
    "wool",
    "milk",
    "fibre",
    "stone",
    "log",
    "planks",
    "rope",
    "grilled-fish",
    "chili-fish",
    "flatbread",
    "roast-turnip",
    "kale-skewer",
    "fried-egg",
    "roast-meat",
    "bucket",
    "water-bucket",
    "fertile-egg",
    "incubator",
    "apple",
    "pear",
    "plum",
    "cherry",
    "mango",
    "papaya",
    "marula",
    "baobab",
    "pine-cone",
    "bait",
    "inner-tube",
    "raft",
    "rowboat",
    "glowing-mushroom",
    "leather",
    "block:stone",
    "block:soil",
    "block:grass",
    "block:ore",
    "block:sand",
    "block:dry grass",
    "block:jungle grass",
    "block:snow",
    "block:rock",
    "block:dirt",
    "block:torch",
    "block:street lantern",
    "block:wall lantern",
    "block:hanging lantern",
    "block:brazier",
    "block:candle",
    "block:tilled soil",
    "block:forge",
    "block:workbench",
    "block:campfire",
    "block:reef rock",
    "block:sea ice",
    "block:basalt",
    "block:mussel bed",
    "block:sticks",
    "block:desert sand",
    "tool:rod",
    "tool:shovel",
    "tool:pickaxe",
    "tool:axe",
    "fish:minnow",
    "fish:silverfin",
    "fish:perch",
    "fish:ray",
    "fish:eel",
    "fish:reef",
    "fish:deepback",
    "fish:serpent",
    "fish:shore_crab",
    "fish:lobster",
    "fish:giant_clam",
    "fish:reef_turtle",
    "fish:sea_otter",
    "fish:reed_dabbler",
    "fish:pond_diver",
    "fish:octopus",
    "fish:tube_worm",
    "fish:seal",
    "fish:beaver",
    "fish:river_otter",
];

#[test]
fn every_item_the_game_knows_has_a_value() {
    let values = values();
    let missing: Vec<String> = known_codes(&all_fish_ids())
        .into_iter()
        .filter(|c| values.get(c).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "give these a row in values.ron: {missing:?}"
    );
}

#[test]
fn the_values_file_only_grows() {
    let values = values();
    let codes: Vec<&str> = values.codes().collect();
    for (i, code) in SHIPPED.iter().enumerate() {
        assert_eq!(
            codes.get(i),
            Some(code),
            "row {i} was {code}: never drop or move a row"
        );
    }
}

#[test]
fn a_trade_code_names_its_item_back() {
    let ids = fish_ids();
    for code in known_codes(&ids) {
        let item = item_of(&code, &ids).unwrap_or_else(|| panic!("{code}"));
        assert_eq!(trade_code(item, &ids).as_deref(), Some(code.as_str()));
    }
}

// ---- trade (6.2)

fn trader<'a>(
    cfg: &'a TradeConfig,
    values: &'a Values,
    kind: SiteKind,
    ids: &'a [String],
) -> Trader<'a> {
    Trader {
        cfg,
        values,
        kind,
        goodwill: 0,
        fish_ids: ids,
    }
}

#[test]
fn a_harbour_sells_fish_below_what_a_mountain_town_pays() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let harbour = trader(&cfg, &values, SiteKind::Harbour, &ids);
    let cliff = trader(&cfg, &values, SiteKind::Cliff, &ids);
    let rest = Market::default();
    for code in ["fish:perch", "fish:minnow", "fish:eel"] {
        let sells = harbour
            .quote(&rest, code)
            .sell
            .expect("the harbour sells fish");
        let pays = cliff
            .quote(&rest, code)
            .buy
            .expect("the mountain town buys fish");
        assert!(
            sells < pays,
            "{code}: harbour sells {sells}, mountain pays {pays}"
        );
    }
    assert!(
        cliff.quote(&rest, "fish:perch").sell.is_none(),
        "it makes none"
    );
}

#[test]
fn buying_and_selling_back_loses_coins() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    for kind in SiteKind::ALL {
        let t = trader(&cfg, &values, kind, &ids);
        for code in t.stock_list() {
            let mut market = Market::default();
            let mut purse = Purse::new(1000);
            let mut slots = Slots::new();
            t.buy(&mut market, &mut purse, &mut slots, &code, 0.0)
                .unwrap();
            if t.sell(&mut market, &mut purse, &mut slots, &code, 0.0)
                .is_ok()
            {
                assert!(purse.coins < 1000, "{kind:?} {code}");
            }
        }
    }
}

#[test]
fn ten_sales_each_pay_less_and_a_day_brings_the_price_back() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let t = trader(&cfg, &values, SiteKind::Cliff, &ids);
    let code = "fish:deepback";
    let fish = item_of(code, &ids).unwrap();
    let mut market = Market::default();
    let mut purse = Purse::default();
    let mut slots = Slots::new();
    slots.give(fish, 10);
    let at_rest = t.quote(&market, code).buy.unwrap();
    let mut paid = Vec::new();
    for _ in 0..10 {
        paid.push(
            t.sell(&mut market, &mut purse, &mut slots, code, 100.0)
                .unwrap()
                .coins,
        );
    }
    assert_eq!(paid[0], at_rest);
    assert!(paid.windows(2).all(|w| w[1] <= w[0]), "{paid:?}");
    assert!(paid[9] < paid[0], "{paid:?}");
    assert_eq!(slots.count(fish), 0);
    assert_eq!(purse.coins, paid.iter().sum::<u64>());
    let next = t.quote(&market, code).buy.unwrap();
    assert!(next < paid[9], "an eleventh would pay less again");
    let later = market.settled(100.0 + DAY_S as f64, &cfg);
    assert!(later.lots.is_empty(), "back at rest");
    assert_eq!(t.quote(&later, code).buy, Some(at_rest));
    let half = market.settled(100.0 + DAY_S as f64 / 2.0, &cfg);
    let mid = t.quote(&half, code).buy.unwrap();
    assert!(mid > next && mid <= at_rest, "half a day: {mid}");
}

#[test]
fn too_little_to_buy_leaves_purse_and_pack() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let t = trader(&cfg, &values, SiteKind::Village, &ids);
    let mut market = Market::default();
    let price = t.quote(&market, "flatbread").sell.unwrap();
    let mut purse = Purse::new(price - 1);
    let mut slots = Slots::new();
    assert_eq!(
        t.buy(&mut market, &mut purse, &mut slots, "flatbread", 0.0),
        Err(Refused::Short)
    );
    assert_eq!(purse.coins, price - 1);
    assert_eq!(slots, Slots::new());
    assert_eq!(market, Market::default());
}

#[test]
fn stock_runs_out_and_want_is_met() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let t = trader(&cfg, &values, SiteKind::Village, &ids);
    let mut market = Market::default();
    let mut purse = Purse::new(100_000);
    let mut slots = Slots::new();
    let stock = t.quote(&market, "flatbread").stock;
    assert!(stock > 0);
    for _ in 0..stock {
        t.buy(&mut market, &mut purse, &mut slots, "flatbread", 0.0)
            .unwrap();
    }
    assert_eq!(
        t.buy(&mut market, &mut purse, &mut slots, "flatbread", 0.0),
        Err(Refused::SoldOut)
    );
    assert_eq!(
        t.sell(&mut market, &mut purse, &mut slots, "wheat", 0.0),
        Err(Refused::NotHeld)
    );
}

#[test]
fn a_market_survives_a_save_and_load() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let t = trader(&cfg, &values, SiteKind::Harbour, &ids);
    let mut market = Market::default();
    let mut purse = Purse::new(500);
    let mut slots = Slots::new();
    t.buy(&mut market, &mut purse, &mut slots, "fish:perch", 30.0)
        .unwrap();
    let record = Record::of(MARKET_KIND, 7, MARKET_SCHEMA, &market);
    assert_eq!(record.read::<Market>(), Some(market));
}

// ---- goodwill (7.7)

#[test]
fn the_price_at_each_goodwill_step() {
    let (cfg, values, ids) = (trade(), values(), fish_ids());
    let market = Market::default();
    let code = "fish:serpent";
    let mut last: Option<(u64, u64)> = None;
    for (points, bonus) in [
        (0, 0.0),
        (20, 0.02),
        (40, 0.04),
        (60, 0.06),
        (80, 0.08),
        (100, 0.10),
        (500, 0.10),
    ] {
        assert!(
            (cfg.goodwill.bonus(points) - bonus).abs() < 1e-6,
            "{points}"
        );
        let mut t = trader(&cfg, &values, SiteKind::Harbour, &ids);
        t.goodwill = points;
        let rest = t.rest_price(code).unwrap();
        let q = t.quote(&market, code);
        let (sell, buy) = (q.sell.unwrap(), q.buy.unwrap());
        assert_eq!(sell, (rest * (1.0 - bonus)).ceil() as u64, "{points}");
        assert_eq!(buy, (rest * 0.6 * (1.0 + bonus)).floor() as u64, "{points}");
        if let Some((s, b)) = last {
            assert!(sell <= s && buy >= b, "better at {points}");
        }
        last = Some((sell, buy));
    }
    let mut g = Goodwill::default();
    assert!(g.raise(150, &cfg.goodwill));
    assert_eq!(g.points, 100);
    assert!(!g.raise(1, &cfg.goodwill));
    let record = Record::of(GOODWILL_KIND, 3, GOODWILL_SCHEMA, &g);
    assert_eq!(record.read::<Goodwill>(), Some(g));
}

// ---- boards and jobs (7.2, 7.3)

fn board<'a>(jobs: &'a JobsConfig, trade: &'a TradeConfig, values: &'a Values) -> Board<'a> {
    Board {
        jobs,
        trade,
        values,
        seed: 42,
    }
}

#[test]
fn a_harbours_board_asks_for_fish() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let none = BTreeSet::new();
    for site in 0..40 {
        let town = Town {
            site,
            kind: SiteKind::Harbour,
            neighbours: &[900, 901],
        };
        for day in 0..12 {
            let posted = b.posted(town, day, 0, &none);
            assert!(
                posted.iter().any(|j| j.kind == JobKind::Fish),
                "site {site} day {day}"
            );
        }
    }
}

#[test]
fn the_same_day_gives_the_same_board_and_three_to_five_jobs() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let none = BTreeSet::new();
    for kind in SiteKind::ALL {
        let town = Town {
            site: 11,
            kind,
            neighbours: &[5, 6],
        };
        for day in [0, 1, 7, 300] {
            let one = b.posted(town, day, 0, &none);
            assert_eq!(one, b.posted(town, day, 0, &none));
            assert_eq!(one.len(), 3, "{kind:?}");
            assert_eq!(
                b.posted(town, day, 100, &none).len(),
                5,
                "{kind:?}: more with goodwill"
            );
            assert_eq!(b.posted(town, day, 40, &none).len(), 4);
            for job in &one {
                assert!(job.reward > 0 && job.id != 0);
            }
        }
    }
}

#[test]
fn a_job_not_taken_is_replaced_after_three_days() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let town = Town {
        site: 3,
        kind: SiteKind::Village,
        neighbours: &[4],
    };
    for place in 0..3 {
        let first = b.job_at(town, 0, place).unwrap();
        let days: Vec<u64> = (0..12)
            .filter(|d| b.job_at(town, *d, place).unwrap().id == first.id)
            .collect();
        assert!(
            !days.is_empty() && days.len() <= 3,
            "place {place}: {days:?}"
        );
        let gone = (0..20)
            .find(|d| b.job_at(town, *d, place).unwrap().id != first.id)
            .unwrap();
        assert!(gone <= 3, "replaced by day 3, place {place}");
    }
}

#[test]
fn a_taken_job_never_expires() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let town = Town {
        site: 9,
        kind: SiteKind::Harbour,
        neighbours: &[1],
    };
    let job = b.posted(town, 0, 0, &BTreeSet::new())[0].clone();
    let mut taken = TakenJobs::default();
    taken.take(job.clone()).unwrap();
    assert_eq!(taken.take(job.clone()), Err(JobRefused::Taken));
    for day in 0..400 {
        assert!(
            !b.posted(town, day, 0, &taken.gone())
                .iter()
                .any(|p| p.id == job.id)
        );
    }
    let record = Record::of(super::jobs::JOBS_KIND, 0, super::jobs::JOBS_SCHEMA, &taken);
    let back: TakenJobs = record.read().unwrap();
    assert_eq!(back.taken, vec![job]);
}

/// Each new objective kind, counted from its `Happened`, by the starter
/// missions' function, only in the job's own town.
#[test]
fn each_new_objective_kind_counts_from_its_event() {
    let site = 12;
    let cases: [(Kind, String, String); 7] = [
        (Kind::Deliver, scoped(40, "parcel"), scoped(41, "parcel")),
        (
            Kind::Escort,
            scoped(site, "villager"),
            scoped(13, "villager"),
        ),
        (Kind::Repair, scoped(site, "piece"), scoped(13, "piece")),
        (
            Kind::Sell,
            format!("{}*", scoped(site, "")),
            scoped(13, "wheat"),
        ),
        (
            Kind::Buy,
            scoped(site, "flatbread"),
            scoped(13, "flatbread"),
        ),
        (Kind::Light, scoped(site, "lamp"), scoped(13, "lamp")),
        (Kind::Clear, scoped(site, "road"), scoped(13, "road")),
    ];
    for (n, (kind, what, elsewhere)) in cases.into_iter().enumerate() {
        let job = Job {
            id: 100 + n as u64,
            site,
            kind: JobKind::Trade,
            objectives: vec![crate::missions::ObjectiveEntry {
                kind,
                what: what.clone(),
                count: 2,
                label: None,
            }],
            reward: 10,
            to: None,
            thing: String::new(),
        };
        let mut taken = TakenJobs::default();
        taken.take(job.clone()).unwrap();
        let hit = if what.ends_with('*') {
            scoped(site, "wheat")
        } else {
            what.clone()
        };
        assert!(
            taken
                .record(&Happened::new(kind, elsewhere, 1))
                .1
                .is_empty()
        );
        assert_eq!(
            taken.counts(job.id),
            vec![0],
            "{kind:?}: another town's does not count"
        );
        assert!(
            taken
                .record(&Happened::new(Kind::Gather, hit.clone(), 1))
                .1
                .is_empty()
        );
        taken.record(&Happened::new(kind, hit.clone(), 1));
        assert_eq!(taken.counts(job.id), vec![1], "{kind:?}");
        assert_eq!(taken.hand_in(job.id, site), Err(JobRefused::NotDone));
        let (_, done) = taken.record(&Happened::new(kind, hit, 1));
        assert_eq!(done, vec![job.id], "{kind:?}");
        assert_eq!(taken.hand_in(job.id, site + 1), Err(JobRefused::WrongTown));
        assert_eq!(taken.hand_in(job.id, site).unwrap().reward, 10);
        assert!(taken.taken.is_empty() && taken.handed_in.contains(&job.id));
    }
}

/// One job of every kind on the board, done from its own events and paid.
#[test]
fn every_job_kind_is_made_and_finished() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let town = Town {
        site: 5,
        kind: SiteKind::Village,
        neighbours: &[6],
    };
    for kind in JobKind::ALL {
        let job = b
            .make(town, kind, 77, 1234)
            .unwrap_or_else(|| panic!("{kind:?}"));
        assert!(job.reward >= 1);
        let mut taken = TakenJobs::default();
        taken.take(job.clone()).unwrap();
        let o = &job.objectives[0];
        let what = match super::jobs::want_of(&o.what) {
            crate::missions::Want::Any => "minnow".to_string(),
            crate::missions::Want::Prefix(p) => format!("{p}wheat"),
            crate::missions::Want::Names(n) => n[0].clone(),
            crate::missions::Want::Group(_) => unreachable!(),
        };
        let (_, done) = taken.record(&Happened::new(o.kind, what, o.count));
        assert_eq!(done, vec![job.id], "{kind:?}");
        assert!(taken.hand_in(job.id, 5).is_ok());
    }
}

#[test]
fn a_harbour_job_pays_one_and_a_half_times_its_value() {
    let (j, t, v) = (jobs(), trade(), values());
    let b = board(&j, &t, &v);
    let town = Town {
        site: 5,
        kind: SiteKind::Harbour,
        neighbours: &[],
    };
    let job = b.make(town, JobKind::Cook, 1, 0).unwrap();
    let value = v.get(&job.thing).unwrap().coins;
    let count = job.objectives[0].count;
    assert_eq!(
        job.reward,
        (count as f32 * value as f32 * 1.5).round() as u64
    );
    assert!(
        b.make(town, JobKind::Deliver, 1, 0).is_none(),
        "nowhere to deliver to"
    );
}

// ---- the lost villager (7.5)

#[test]
fn a_found_villager_follows_waits_at_water_and_is_home_at_the_wall() {
    let dry = |_: DVec3| true;
    let mut at = DVec3::new(0.0, 0.0, 0.0);
    let player = DVec3::new(10.0, 0.0, 0.0);
    for _ in 0..200 {
        at = follow::step(at, player, 1.4, 0.05, dry);
    }
    assert!((player - at).length() <= GAP_M + 1e-6, "caught up");
    assert!((player - at).length() >= GAP_M - 1e-6, "keeps its gap");
    let still = follow::step(at, player, 0.0, 0.05, dry);
    assert_eq!(still, at, "stops when you stop");
    let water = |p: DVec3| p.x < 3.0;
    let mut w = DVec3::ZERO;
    for _ in 0..200 {
        w = follow::step(w, player, 1.4, 0.05, water);
    }
    assert!(w.x < 3.0 && w.x > 2.5, "waits at the edge: {w}");
    let anchor = DVec3::new(0.0, 1000.0, 0.0);
    assert!(follow::home(
        anchor + DVec3::new(20.0, 3.0, 0.0),
        anchor,
        30.0
    ));
    assert!(!follow::home(
        anchor + DVec3::new(40.0, 0.0, 0.0),
        anchor,
        30.0
    ));
}

#[test]
fn every_town_has_a_trader_and_a_giver() {
    let j = jobs();
    let town = ["stallholder", "guard", "reeve", "merchant"];
    assert_eq!(j.keepers(&town).map(|k| (k.trader, k.giver)), Some((3, 2)));
    let village = ["villager", "smith", "farmer"];
    assert_eq!(
        j.keepers(&village).map(|k| (k.trader, k.giver)),
        Some((0, 1))
    );
    let one = ["lookout"];
    assert_eq!(j.keepers(&one).map(|k| (k.trader, k.giver)), Some((0, 0)));
    assert_eq!(j.keepers(&[]), None);
}
