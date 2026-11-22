use super::*;
use crate::config::{FaunaConfig, HusbandryConfig};
use crate::missions::{Did, MissionChain, MissionsPlugin};
use pbd_core::economy::jobs::Board;
use pbd_core::missions::{Chain, Kind};

fn config() -> EconomyConfig {
    EconomyConfig::parse(
        include_str!("../../../../assets/config/values.ron"),
        include_str!("../../../../assets/config/trade.ron"),
        include_str!("../../../../assets/config/jobs.ron"),
    )
    .unwrap()
}

fn fauna() -> FaunaConfig {
    FaunaConfig(ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap())
}

fn harbour(site: u32) -> TownInfo {
    TownInfo {
        site,
        kind: SiteKind::Harbour,
        name: "Saltmere".into(),
        neighbours: vec![site + 1],
    }
}

/// An app counting the missions and the town jobs into a memory-only world:
/// what the game runs, without a window.
fn app() -> App {
    let (recipes, _) = crate::config::recipe_book();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(recipes)
        .insert_resource(fauna())
        .insert_resource(HusbandryConfig(Default::default()))
        .insert_resource(MissionChain(Chain::default()))
        .insert_resource(WorldSave::memory_only())
        .insert_resource(crate::hotbar::Hotbar::default())
        .insert_resource(config())
        .add_plugins((MissionsPlugin, EconomyPlugin));
    app.update();
    app
}

/// Make a deal in the app's world: the shop over its resources.
fn shop<R>(app: &mut App, f: impl FnOnce(&mut Shop) -> R) -> R {
    let world = app.world_mut();
    let ids = fish_ids(world.resource::<FaunaConfig>(), pbd_core::fauna::HOME_BODY);
    let cfg = world.resource::<EconomyConfig>().clone();
    let mut save = world.remove_resource::<WorldSave>().unwrap();
    let mut hotbar = world.remove_resource::<crate::hotbar::Hotbar>().unwrap();
    let mut eco = world.remove_resource::<Economy>().unwrap();
    let out = {
        let mut shop = Shop {
            save: &mut save,
            slots: &mut hotbar.0,
            eco: &mut eco,
            cfg: &cfg,
            fish_ids: &ids,
            now_s: 1000.0,
        };
        f(&mut shop)
    };
    world.insert_resource(save);
    world.insert_resource(hotbar);
    world.insert_resource(eco);
    out
}

/// The bot of task 6.3 and 7.4: takes a harbour's fishing job, lands the
/// fish (the fishing's own `Did` messages, counted by the starter missions'
/// function), hands it in and is paid; buys a loaf with the pay and sells
/// three fish. The purse and the pack agree with the save after each step,
/// and a reload reads the same purse.
#[test]
fn a_bot_takes_a_job_is_paid_buys_a_loaf_and_sells_three_fish() {
    let mut app = app();
    let town = harbour(21);
    let job = shop(&mut app, |s| {
        let posted = s.board(&town);
        assert!((3..=5).contains(&posted.len()));
        let fish = posted
            .into_iter()
            .find(|j| j.kind == JobKind::Fish)
            .expect("a harbour posts fishing");
        s.take(fish.clone()).unwrap();
        fish
    });
    let wanted = job.objectives[0].count;
    // Hand in too soon: refused.
    assert_eq!(
        shop(&mut app, |s| s.hand_in(&town, job.id).map(|_| ())),
        Err(Why::Job(JobRefused::NotDone))
    );
    for _ in 0..wanted {
        app.world_mut()
            .write_message(Did::new(Kind::Catch, "banded perch", 1));
        app.update();
    }
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
    assert!(app.world().resource::<Economy>().notice.is_some());
    // The counts are in the save.
    assert!(jobs_of(app.world().resource::<WorldSave>()).is_done(job.id));
    // Paid at the board.
    shop(&mut app, |s| s.hand_in(&town, job.id)).unwrap();
    let paid = app.world().resource::<Economy>().purse.coins;
    assert_eq!(paid, job.reward);
    assert!(goodwill_of(app.world().resource::<WorldSave>(), town.site).points > 0);
    // A loaf.
    let loaf = shop(&mut app, |s| s.buy(&town, "flatbread")).unwrap();
    let bread = Item::Good(Good::by_code("flatbread").unwrap());
    // Three fish, landed into the pack.
    let perch = Item::Fish(2);
    app.world_mut()
        .resource_mut::<crate::hotbar::Hotbar>()
        .0
        .give(perch, 3);
    let mut sold = 0;
    for _ in 0..3 {
        sold += shop(&mut app, |s| s.sell(&town, "fish:perch"))
            .unwrap()
            .coins;
    }
    let world = app.world();
    let purse = world.resource::<Economy>().purse.coins;
    assert_eq!(purse, paid - loaf.coins + sold);
    let slots = &world.resource::<crate::hotbar::Hotbar>().0;
    assert_eq!(slots.count(bread), 1);
    assert_eq!(slots.count(perch), 0);
    let save = world.resource::<WorldSave>();
    assert_eq!(purse_of(save).coins, purse, "the purse is saved");
    assert!(save.records.get(MARKET_KIND, town.site as u64).is_some());
    // A world opened again reads the same purse.
    let mut fresh = Economy::default();
    fresh.join(save);
    assert_eq!(fresh.purse.coins, purse);
    assert!(fresh.jobs.taken.is_empty());
    assert!(fresh.jobs.handed_in.contains(&job.id));
}

/// Too few coins: refused, and neither the purse, the pack nor the save's
/// market changes.
#[test]
fn too_few_coins_change_nothing() {
    let mut app = app();
    let town = harbour(3);
    let before = app.world().resource::<WorldSave>().records.len();
    let why = shop(&mut app, |s| s.buy(&town, "flatbread"));
    assert_eq!(why, Err(Why::Trade(Refused::Short)));
    assert_eq!(app.world().resource::<Economy>().purse.coins, 0);
    assert_eq!(app.world().resource::<WorldSave>().records.len(), before);
    assert_eq!(
        app.world().resource::<crate::hotbar::Hotbar>().0,
        Slots::new()
    );
}

/// A delivery: the parcel is handed over on taking the job, given to the
/// other town's people, and paid back at the giver.
#[test]
fn a_delivery_hands_its_parcel_to_the_other_town() {
    let mut app = app();
    let cfg = config();
    let town = harbour(30);
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 1,
    };
    let job = board.make(town.as_town(), JobKind::Deliver, 99, 5).unwrap();
    let to = job.to.unwrap();
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    let parcel = Item::Good(Good::PARCEL);
    assert_eq!(
        app.world()
            .resource::<crate::hotbar::Hotbar>()
            .0
            .count(parcel),
        1
    );
    // Not counted in the wrong town.
    assert_eq!(shop(&mut app, |s| s.deliver(&harbour(77))), Ok(0));
    let there = TownInfo {
        site: to,
        ..harbour(to)
    };
    assert_eq!(shop(&mut app, |s| s.deliver(&there)), Ok(1));
    assert_eq!(
        app.world()
            .resource::<crate::hotbar::Hotbar>()
            .0
            .count(parcel),
        0
    );
    assert_eq!(
        shop(&mut app, |s| s.hand_in(&there, job.id).map(|_| ())),
        Err(Why::Job(JobRefused::WrongTown))
    );
    let paid = shop(&mut app, |s| s.hand_in(&town, job.id)).unwrap();
    assert_eq!(app.world().resource::<Economy>().purse.coins, paid.reward);
}

/// A sale counts toward a trade job in that town, by the same rule.
#[test]
fn a_sale_counts_toward_a_trade_job() {
    let mut app = app();
    let cfg = config();
    let town = harbour(8);
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 1,
    };
    let job = board.make(town.as_town(), JobKind::Trade, 5, 0).unwrap();
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    app.world_mut()
        .resource_mut::<crate::hotbar::Hotbar>()
        .0
        .give(Item::Good(Good::WHEAT), 20);
    for _ in 0..job.objectives[0].count {
        shop(&mut app, |s| s.sell(&town, "wheat")).unwrap();
    }
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
}

/// Every job kind and every role the townsfolk carry has its words.
#[test]
fn every_job_kind_and_role_has_its_words() {
    let locale = pbd_core::locale::Locale::english();
    for kind in JobKind::ALL {
        let key = format!("jobs.kind.{}", kind.code());
        assert!(locale.get(&key).is_some(), "{key}");
    }
    let cfg = config();
    for role in cfg.jobs.trader_roles.iter().chain(&cfg.jobs.giver_roles) {
        let key = format!("talk.role.{role}");
        assert!(locale.get(&key).is_some(), "{key}");
    }
    let dir = crate::paths::assets_dir().join("settlements/v1");
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let text = std::fs::read_to_string(entry.path()).unwrap();
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        for person in json["people"].as_array().into_iter().flatten() {
            if let Some(role) = person["role"].as_str() {
                let key = format!("talk.role.{role}");
                assert!(locale.get(&key).is_some(), "{key} ({:?})", entry.path());
            }
        }
    }
}

#[test]
fn a_town_with_no_people_laid_keeps_trade_by_its_drawn_talkers() {
    let cfg = config();
    let t = |site, index, role: &str| Talker {
        site,
        index,
        role: role.into(),
    };
    let drawn = [
        t(7, 2, "smith"),
        t(9, 0, "merchant"),
        t(7, 0, "villager"),
        t(7, 1, "trader"),
    ];
    let refs: Vec<&Talker> = drawn.iter().collect();
    let people = roster(None, &refs, 7);
    assert_eq!(people.len(), 3, "only the town's own talkers");
    let all: Vec<&Talker> = people.iter().collect();
    let k = keepers(&cfg.jobs, &all, 7).expect("a town with people has keepers");
    assert_eq!(
        (k.trader, k.giver),
        (1, 2),
        "the trader trades, the smith gives jobs"
    );
    // An empty laid roster is no roster: the drawn talkers stand in.
    let folk = crate::townsfolk::Townsfolk::default();
    assert_eq!(roster(Some(&folk), &refs, 7), people);
}
