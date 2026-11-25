//! Derived values, the world's market and the basket deal (`townsfolk`
//! design 37 and 37a, tasks 11.1 to 11.3 and 12.3).

use super::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA, Purse};
use super::derive::{Deriver, makings};
use super::market::{
    WORLD_MARKET_ID, WORLD_MARKET_KIND, WORLD_MARKET_SCHEMA, WorldMarket, balance,
};
use super::trade::{Basket, Market, Refused, TradeConfig, Trader};
use super::values::{Values, item_of};
use crate::inventory::Slots;
use crate::records::Record;
use crate::sites::SiteKind;
use crate::station::{RecipeBook, RecipeFile};

const VALUES: &str = include_str!("../../../../assets/config/values.ron");
const TRADE: &str = include_str!("../../../../assets/config/trade.ron");
const RECIPES: &str = include_str!("../../../../assets/config/recipes.ron");
const COOKING: &str = include_str!("../../../../assets/config/cooking.ron");

fn values() -> Values {
    Values::parse(VALUES).unwrap()
}

fn trade() -> TradeConfig {
    TradeConfig::parse(TRADE).unwrap()
}

/// The one book the game builds: `recipes.ron`, then the kitchen's.
fn book() -> RecipeBook {
    let file: RecipeFile = ron::from_str(RECIPES).unwrap();
    let mut book = file.resolve().unwrap();
    let kitchen: crate::cooking::CookingFile = ron::from_str(COOKING).unwrap();
    book.recipes.extend(kitchen.resolve().unwrap().recipes);
    book
}

fn deriver_with(values: Values) -> Deriver {
    let cfg = trade();
    let overrides = values.overrides().to_vec();
    Deriver::new(
        values,
        makings(&book()),
        cfg.labour_per_s,
        cfg.craft_margin,
        &overrides,
    )
    .unwrap()
}

fn fish_ids() -> Vec<String> {
    let fauna: crate::fauna::FaunaSettings =
        ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap();
    fauna
        .roster(crate::fauna::HOME_BODY)
        .iter()
        .map(|s| s.id.clone())
        .collect()
}

/// Task 11.1: every made good is worth more than the inputs of the recipe
/// that prices it (its outputs together above its inputs together), and a
/// raw material keeps its row.
#[test]
fn every_made_good_is_worth_more_than_its_inputs() {
    let base = values();
    let deriver = deriver_with(base.clone());
    let derived = deriver.derive();
    let v = &derived.values;
    let worth = |code: &str| -> u64 {
        if code == "fish" {
            v.codes()
                .filter(|c| c.starts_with("fish:"))
                .filter_map(|c| v.get(c))
                .map(|x| u64::from(x.coins))
                .min()
                .unwrap()
        } else {
            u64::from(v.get(code).unwrap().coins)
        }
    };
    assert!(!derived.source.is_empty());
    for (code, &at) in &derived.source {
        let m = &deriver.makings()[at];
        let inputs: u64 = m.inputs.iter().map(|(c, n)| worth(c) * u64::from(*n)).sum();
        let outputs: u64 = m
            .outputs
            .iter()
            .map(|(c, n)| worth(c) * u64::from(*n))
            .sum();
        assert!(
            outputs > inputs,
            "{code} by {}: outputs {outputs} not above inputs {inputs}",
            m.recipe
        );
    }
    for code in ["stone", "log", "fibre", "wheat", "egg", "coal"] {
        assert!(deriver.is_raw(code), "{code} is raw");
        assert_eq!(v.get(code), base.get(code), "{code} keeps its row");
    }
    for code in [
        "planks",
        "rope",
        "bronze-ingot",
        "grilled-fish",
        "flatbread",
    ] {
        assert!(!deriver.is_raw(code), "{code} is made");
        assert!(derived.source.contains_key(code), "{code} is priced");
    }
    assert_eq!(
        v.get("tool:pickaxe"),
        base.get("tool:pickaxe"),
        "a tool keeps its row"
    );
    assert!(
        worth("bronze-ingot") > worth("copper-ingot"),
        "bronze costs its copper, its tin and the forge's time"
    );
}

/// Task 11.1: a dearer input makes a dearer good, whether its row is
/// retuned or the market's index on it rises; task 12.3: labour is priced
/// at 0.1 coin a second.
#[test]
fn a_dearer_input_makes_a_dearer_good() {
    let base = values();
    let rest = deriver_with(base.clone()).derive().values;
    let text = VALUES.replace("(\"log\", 3, wood)", "(\"log\", 30, wood)");
    assert_ne!(text, VALUES, "the log row is where the test expects it");
    let dear = deriver_with(Values::parse(&text).unwrap()).derive().values;
    for code in ["planks", "block:workbench", "raft"] {
        assert!(
            dear.get(code).unwrap().coins > rest.get(code).unwrap().coins,
            "dear logs make dear {code}"
        );
    }
    let deriver = deriver_with(base);
    let tin = deriver
        .priced(|c| if c == "tin-ore" { 2.0 } else { 1.0 })
        .values;
    assert!(
        tin.get("bronze-ingot").unwrap().coins > rest.get("bronze-ingot").unwrap().coins,
        "dear tin makes dear bronze"
    );
    assert_eq!(trade().labour_per_s, 0.1);
}

/// `values.ron`'s overrides keep a made good's row; one naming no row is
/// refused.
#[test]
fn an_override_keeps_a_made_goods_row() {
    let text = VALUES.replace(
        "    values: [",
        "    overrides: [\"planks\"],\n    values: [",
    );
    let base = Values::parse(&text).unwrap();
    let d = deriver_with(base.clone());
    assert!(d.is_raw("planks"));
    assert_eq!(d.derive().values.get("planks"), base.get("planks"));
    let bad = VALUES.replace(
        "    values: [",
        "    overrides: [\"no-such\"],\n    values: [",
    );
    assert!(Values::parse(&bad).is_err());
}

/// Task 11.2: selling a flood of fish lowers its index, and it recovers
/// over days.
#[test]
fn a_flood_of_fish_lowers_the_index_and_it_recovers() {
    let cfg = trade();
    let v = values();
    let fish = format!("fish:{}", fish_ids()[0]);
    let mut m = WorldMarket::fresh(10);
    assert_eq!(m.index(&fish), 1.0);
    m.traded(&fish, 60, true, &cfg.market);
    let low = m.index(&fish);
    assert!(low < 0.8, "sixty sold: {low}");
    let none = |_: &str| 0.0;
    let mut last = low;
    for day in 11..=20 {
        let next = m.settled(day, &cfg.market, &v, none);
        let now = next.index(&fish);
        assert!(now > last && now <= 1.0, "day {day}: {now} after {last}");
        last = now;
        m = next;
    }
    let later = m.settled(200, &cfg.market, &v, none);
    assert_eq!(later.index(&fish), 1.0, "back at rest, and no row");
    assert!(later.index.is_empty());
    // The same whether read day by day or at once.
    let mut a = WorldMarket::fresh(0);
    a.traded(&fish, 30, true, &cfg.market);
    let at_once = a.settled(7, &cfg.market, &v, none);
    let mut daily = a.clone();
    for day in 1..=7 {
        daily = daily.settled(day, &cfg.market, &v, none);
    }
    assert_eq!(at_once, daily);
}

/// Task 11.2: the towns' making and using pull a good most towns lack up,
/// and one most make down, to a steady point near 1.0.
#[test]
fn the_towns_pull_a_lacked_good_up_and_a_made_one_down() {
    let cfg = trade();
    let v = values();
    let towns = [SiteKind::Harbour, SiteKind::Harbour, SiteKind::Tundra];
    let fish = format!("fish:{}", fish_ids()[0]);
    assert!(balance(&cfg, &v, &towns, &fish) < 0.0, "harbours make fish");
    assert!(balance(&cfg, &v, &towns, "log") > 0.0, "they lack wood");
    assert_eq!(balance(&cfg, &v, &[], "log"), 0.0);
    let m = WorldMarket::fresh(0).settled(100, &cfg.market, &v, |c| balance(&cfg, &v, &towns, c));
    assert!(m.index(&fish) < 1.0 && m.index(&fish) > 0.8);
    assert!(m.index("log") > 1.0 && m.index("log") < 1.25);
}

/// Task 11.2: a world from before the market has no record and reads every
/// index as 1.0; the record reads back as it was written.
#[test]
fn an_old_save_reads_every_index_at_one_and_the_record_round_trips() {
    let m = WorldMarket::fresh(3);
    for code in values().codes() {
        assert_eq!(m.index(code), 1.0);
    }
    let mut m = m;
    m.traded("wheat", 5, false, &trade().market);
    let record = Record::of(WORLD_MARKET_KIND, WORLD_MARKET_ID, WORLD_MARKET_SCHEMA, &m);
    let back: WorldMarket = record.read().unwrap();
    assert_eq!(back, m);
    assert!(m.index("wheat") > 1.0);
    assert!(m.moved_from(&WorldMarket::fresh(3)));
    assert!(!m.moved_from(&m.clone()));
}

/// Task 11.3: a deal paid half in goods: the player takes two flatbread
/// and gives fish worth about half, paying the rest in coins, all at once;
/// a short purse refuses the whole of it.
#[test]
fn a_basket_deal_paid_half_in_goods() {
    let cfg = trade();
    let v = deriver_with(values()).derive().values;
    let ids = fish_ids();
    let trader = Trader {
        cfg: &cfg,
        values: &v,
        kind: SiteKind::Village,
        goodwill: 0,
        fish_ids: &ids,
    };
    let fish = format!("fish:{}", ids[0]);
    let mut slots = Slots::new();
    slots.give(item_of(&fish, &ids).unwrap(), 3);
    let mut basket = Basket::default();
    basket.add("flatbread", true, 2);
    basket.add(&fish, false, 1);
    let market = Market::default();
    let tally = trader.tally(&market, &basket);
    assert!(tally.give > 0 && tally.take > tally.give, "{tally:?}");
    let owed = tally.balance() as u64;
    // Short: nothing changes.
    let mut purse = Purse::new(owed - 1);
    let (mut m, mut s) = (market.clone(), slots.clone());
    assert_eq!(
        trader.deal(&mut m, &mut purse, &mut s, &basket, 0.0),
        Err(Refused::Short)
    );
    assert_eq!((purse.coins, &s, &m), (owed - 1, &slots, &market));
    // Covered: coins and goods together.
    let mut purse = Purse::new(owed + 5);
    let deal = trader
        .deal(&mut m, &mut purse, &mut s, &basket, 0.0)
        .unwrap();
    assert_eq!(deal.tally, tally);
    assert_eq!(purse.coins, 5);
    assert_eq!(s.count(item_of(&fish, &ids).unwrap()), 2);
    assert_eq!(s.count(item_of("flatbread", &ids).unwrap()), 2);
    assert!(m.lots["flatbread"].sold == 2.0 && m.lots[&fish].bought == 1.0);
    let record = Record::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse);
    assert_eq!(record.read::<Purse>().unwrap(), purse);
    // Pure barter: the two sides even, no coins and no entry.
    let mut even = Basket::default();
    even.add(&fish, false, 1);
    assert_eq!(
        trader.deal(&mut m, &mut purse, &mut s, &Basket::default(), 0.0),
        Err(Refused::Empty)
    );
    let paid = trader.tally(&m, &even).give;
    let before = purse.coins;
    let deal = trader.deal(&mut m, &mut purse, &mut s, &even, 0.0).unwrap();
    assert_eq!(purse.coins, before + paid);
    assert!(deal.entry.is_some() || paid == 0);
}
