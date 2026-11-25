//! The basket deal and the world's market in the game (`townsfolk` design
//! 37a, tasks 11.2 and 11.3).

use super::tests::{app, harbour, shop};
use super::*;
use pbd_core::economy::market::WORLD_MARKET_KIND;

/// Task 11.3: a deal paid half in goods. The player takes two loaves and
/// gives fish for part of them, paying the rest in coins; the pack, the
/// purse, the town's market and the world's market move together and are in
/// the save, and the town counts a buy and a sell.
#[test]
fn a_deal_paid_half_in_goods() {
    let mut app = app();
    let town = harbour(31);
    let perch = Item::Fish(2);
    let bread = Item::Good(Good::by_code("flatbread").unwrap());
    app.world_mut()
        .resource_mut::<crate::hotbar::Hotbar>()
        .0
        .give(perch, 2);
    let mut basket = Basket::default();
    basket.add("flatbread", true, 2);
    basket.add("fish:perch", false, 2);
    let tally = shop(&mut app, |s| s.tally(&town, &basket));
    assert!(tally.give > 0, "the fish are worth something: {tally:?}");
    assert!(tally.take > tally.give, "and less than the bread");
    let owed = tally.balance() as u64;
    // Too few coins for the rest: nothing moves.
    let before = app.world().resource::<WorldSave>().records.len();
    assert_eq!(
        shop(&mut app, |s| s.barter(&town, &basket)).map(|_| ()),
        Err(Why::Trade(Refused::Short))
    );
    assert_eq!(app.world().resource::<WorldSave>().records.len(), before);
    assert_eq!(
        app.world()
            .resource::<crate::hotbar::Hotbar>()
            .0
            .count(perch),
        2
    );
    // With the coins: half in goods, the rest in coins, all at once.
    app.world_mut().resource_mut::<Economy>().purse = Purse::new(owed + 3);
    let deal = shop(&mut app, |s| s.barter(&town, &basket)).unwrap();
    assert_eq!(deal.tally, tally);
    let world = app.world();
    let eco = world.resource::<Economy>();
    assert_eq!(eco.purse.coins, 3);
    let slots = &world.resource::<crate::hotbar::Hotbar>().0;
    assert_eq!((slots.count(bread), slots.count(perch)), (2, 0));
    assert!(eco.market.index("fish:perch") < 1.0, "fish sold: cheaper");
    assert!(eco.market.index("flatbread") > 1.0, "bread bought: dearer");
    let kinds: Vec<Kind> = eco.raised.iter().map(|h| h.kind).collect();
    assert!(kinds.contains(&Kind::Buy) && kinds.contains(&Kind::Sell));
    let save = world.resource::<WorldSave>();
    assert_eq!(purse_of(save).coins, 3, "the purse is saved");
    assert!(save.records.get(MARKET_KIND, town.site as u64).is_some());
    let stored = world_market_of(save, 0);
    assert_eq!(stored.index, eco.market.index, "the world market is saved");
    // A world opened again reads them back.
    let mut fresh = Economy::default();
    fresh.join(save);
    assert_eq!(fresh.purse.coins, 3);
    // An empty basket is refused with its words.
    assert_eq!(
        shop(&mut app, |s| s.barter(&town, &Basket::default())).map(|_| ()),
        Err(Why::Trade(Refused::Empty))
    );
}

/// Task 11.2: a world saved before the market has no record; it opens with
/// every index at 1.0, prices at the derived values, and writes nothing
/// until a trade.
#[test]
fn a_world_from_before_the_market_opens_at_rest_and_writes_nothing() {
    let mut app = app();
    let cfg = app.world().resource::<EconomyConfig>().clone();
    let world = app.world_mut();
    let save = world.resource::<WorldSave>();
    assert!(save.records.get(WORLD_MARKET_KIND, 0).is_none());
    let mut eco = Economy::default();
    eco.join(save);
    let line = eco.settle_market(save, &cfg, 12);
    assert!(line.is_none(), "nothing moved, nothing written");
    for code in cfg.values.codes() {
        assert_eq!(eco.market.index(code), 1.0);
    }
    assert_eq!(eco.prices.as_ref(), Some(&cfg.values));
    // Days later with towns that lack wood: a row moves and is written.
    eco.town_kinds = vec![SiteKind::Desert, SiteKind::Tundra];
    let line = eco.settle_market(save, &cfg, 20);
    assert!(line.is_some());
    assert!(eco.market.index("log") > 1.0);
}
