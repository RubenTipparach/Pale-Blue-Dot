use super::*;
use crate::config::{FaunaConfig, HusbandryConfig};
use crate::economy::{EconomyConfig, EconomyPlugin, Shop, Talker, TownInfo, fish_ids, purse_of};
use crate::missions::{Did, MissionChain, MissionsPlugin};
use crate::saves::{create, list};
use pbd_core::economy::coins::Purse;
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::missions::MISSIONS_KIND;
use pbd_core::missions::town::paid_key;
use pbd_core::sites::SiteKind;

fn config() -> EconomyConfig {
    EconomyConfig::parse(
        include_str!("../../../../assets/config/values.ron"),
        include_str!("../../../../assets/config/trade.ron"),
        include_str!("../../../../assets/config/jobs.ron"),
    )
    .unwrap()
}

fn harbour(site: u32) -> TownInfo {
    TownInfo {
        site,
        kind: SiteKind::Harbour,
        name: "Saltmere".into(),
        neighbours: vec![site + 1],
    }
}

fn temporary(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-town-missions-{name}-{}-{}",
        std::process::id(),
        crate::saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// An app counting both trees and the town jobs into `save`, with the
/// shipped chains: what the game runs, without a window.
fn app(save: WorldSave) -> App {
    let (recipes, _) = crate::config::recipe_book();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(recipes)
        .insert_resource(FaunaConfig(
            ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap(),
        ))
        .insert_resource(HusbandryConfig(Default::default()))
        .insert_resource(save)
        .insert_resource(Hotbar::default())
        .insert_resource(config())
        .add_plugins((MissionsPlugin, EconomyPlugin));
    app.update();
    app
}

/// Do something in the app's town: the shop over its resources.
fn shop<R>(app: &mut App, f: impl FnOnce(&mut Shop) -> R) -> R {
    let world = app.world_mut();
    let ids = fish_ids(world.resource::<FaunaConfig>(), pbd_core::fauna::HOME_BODY);
    let cfg = world.resource::<EconomyConfig>().clone();
    let mut save = world.remove_resource::<WorldSave>().unwrap();
    let mut hotbar = world.remove_resource::<Hotbar>().unwrap();
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

fn talker(site: u32, index: u32) -> Talker {
    Talker {
        site,
        index,
        role: "farmer".into(),
    }
}

fn town(app: &App) -> &TownMissions {
    app.world().resource::<TownMissions>()
}

fn reach_town(app: &mut App) {
    app.world_mut()
        .write_message(Did::new(Kind::Reach, "town", 1));
    app.update();
}

/// Tasks 2.2 and 3: there is no record and no tree before a town is
/// reached; the first `reach town` writes the record and opens "Meet the
/// townsfolk", and the starter tree's record is not changed by it.
#[test]
fn the_tree_opens_on_reaching_a_town_and_the_starter_record_stays() {
    let mut app = app(WorldSave::memory_only());
    let save = app.world().resource::<WorldSave>();
    assert!(record_of(save).is_none());
    assert!(!town(&app).is_open());
    let starter = save.records.get(MISSIONS_KIND, 0).map(|r| r.body.clone());
    reach_town(&mut app);
    let save = app.world().resource::<WorldSave>();
    let record = record_of(save).expect("the record is written");
    assert!(record.progress.done.is_empty());
    let chain = &app.world().resource::<TownChain>().0;
    assert_eq!(
        record.progress.current(chain).map(|m| m.id.as_str()),
        Some("meet")
    );
    assert_eq!(
        save.records.get(MISSIONS_KIND, 0).map(|r| r.body.clone()),
        starter,
        "the starter record is the starter tree's alone"
    );
}

/// Task 2.4 in the game: talking, trading and a job handed in count from
/// where the game does them, a sale counts for the tree and for a taken
/// trade job at once, and every mission's coins land in the purse.
#[test]
fn talking_and_trading_advance_the_town_tree() {
    let mut app = app(WorldSave::memory_only());
    reach_town(&mut app);
    let here = harbour(8);
    for index in [0, 0, 1, 2] {
        shop(&mut app, |s| s.talked(&talker(here.site, index)));
        app.update();
    }
    assert!(town(&app).progress().unwrap().is_done("meet"));
    assert_eq!(app.world().resource::<Economy>().purse.coins, 5);
    // A trade job taken, then five sales: both count each one.
    let cfg = config();
    let board = pbd_core::economy::jobs::Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 1,
    };
    let job = board
        .make(
            here.as_town(),
            pbd_core::economy::jobs::JobKind::Trade,
            5,
            0,
        )
        .unwrap();
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    app.world_mut()
        .resource_mut::<Hotbar>()
        .0
        .give(Item::Good(Good::WHEAT), 20);
    for _ in 0..5 {
        shop(&mut app, |s| s.sell(&here, "wheat")).unwrap();
        app.update();
    }
    let chain = app.world().resource::<TownChain>().0.clone();
    let p = town(&app).progress().unwrap().clone();
    assert_eq!(p.counts(chain.get("first-trade").unwrap()), [5, 0]);
    shop(&mut app, |s| s.buy(&here, "flatbread")).unwrap();
    app.update();
    assert!(town(&app).progress().unwrap().is_done("first-trade"));
    // Hand the trade job in, if the five sales finished it.
    for _ in 0..job.objectives[0].count {
        shop(&mut app, |s| s.sell(&here, "wheat")).ok();
        app.update();
    }
    shop(&mut app, |s| s.hand_in(&here, job.id)).unwrap();
    app.update();
    assert!(town(&app).progress().unwrap().is_done("first-job"));
    let save = app.world().resource::<WorldSave>();
    let purse = purse_of(save);
    assert_eq!(purse, app.world().resource::<Economy>().purse);
    for id in ["meet", "first-trade", "first-job"] {
        assert!(purse.paid.contains(&paid_key(id)), "{id} is paid");
    }
    assert_eq!(
        record_of(save).unwrap().progress,
        *town(&app).progress().unwrap()
    );
}

/// Task 2.3: a mission's coins are paid once across a quit and reload;
/// and a write torn after the purse's line (the mission paid, its done mark
/// lost) finishes it again on the next count without paying it again.
#[test]
fn a_mission_pays_once_across_a_save_and_reload() {
    let root = temporary("paid-once");
    let slot = create(&root, "Town", 9).unwrap();
    {
        let mut app = app(WorldSave::open(root.clone(), slot.clone()));
        reach_town(&mut app);
        for index in 0..3 {
            shop(&mut app, |s| s.talked(&talker(4, index)));
            app.update();
        }
        assert_eq!(app.world().resource::<Economy>().purse.coins, 5);
        app.world().resource::<WorldSave>().drain();
    }
    let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
    assert_eq!(purse_of(&reopened).coins, 5);
    assert!(record_of(&reopened).unwrap().progress.is_done("meet"));
    let mut app = app(reopened);
    for index in 3..6 {
        shop(&mut app, |s| s.talked(&talker(4, index)));
        app.update();
    }
    assert_eq!(
        app.world().resource::<Economy>().purse.coins,
        5,
        "not twice"
    );
    // The torn write: the purse holds the coins and the paid mark, the
    // record lost "meet". Talking again finishes it and pays nothing.
    let mut save = WorldSave::memory_only();
    let mut purse = Purse::new(5);
    purse.paid.insert(paid_key("meet"));
    let lines = vec![StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse)];
    save.store(&Author::Player(0), lines).unwrap();
    let opened = TownRecord::default();
    let line = StoredRecord::of(TOWN_MISSIONS_KIND, 0, TOWN_MISSIONS_SCHEMA, &opened);
    save.store(&Author::Player(0), vec![line]).unwrap();
    let mut app = self::app(save);
    for index in 0..3 {
        shop(&mut app, |s| s.talked(&talker(4, index)));
        app.update();
    }
    assert!(town(&app).progress().unwrap().is_done("meet"));
    assert_eq!(app.world().resource::<Economy>().purse.coins, 5);
    let _ = std::fs::remove_dir_all(&root);
}

/// Task 2.6: an old world from before the town tree, with a purse and no
/// record, opens the tree at its root with nothing done and nothing paid,
/// and no record it held changes. One with no sign of a town waits.
#[test]
fn an_old_world_with_a_purse_opens_the_tree_with_nothing_paid() {
    let root = temporary("old-world");
    let slot = create(&root, "Old", 11).unwrap();
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let purse = Purse::new(42);
        let lines = vec![StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse)];
        save.store(&Author::Player(0), lines).unwrap();
        save.note_record_kinds(&[(PURSE_KIND, PURSE_SCHEMA)]);
        save.drain();
    }
    let old = WorldSave::open(root.clone(), list(&root)[0].clone());
    let before: Vec<(String, u64, String)> = old
        .records
        .iter()
        .map(|r| (r.kind.clone(), r.id, r.body.clone()))
        .collect();
    let app = app(old);
    let save = app.world().resource::<WorldSave>();
    let record = record_of(save).expect("opened on load");
    assert!(record.progress.done.is_empty());
    assert_eq!(purse_of(save).coins, 42, "nothing paid");
    for (kind, id, body) in &before {
        assert_eq!(
            save.records.get(kind, *id).map(|r| &r.body),
            Some(body),
            "{kind} {id}"
        );
    }
    let chain = &app.world().resource::<TownChain>().0;
    assert_eq!(
        record.progress.current(chain).map(|m| m.id.as_str()),
        Some("meet")
    );
    let _ = std::fs::remove_dir_all(&root);
    // A world with no sign of a town has no tree.
    let fresh = app_fresh();
    assert!(!town(&fresh).is_open());
}

fn app_fresh() -> App {
    app(WorldSave::memory_only())
}

/// The town tree's unlocks are recipes the game has, or recipes it awaits
/// (data only). The paving's and the carts' are built (`townsfolk` design
/// 44, `land-transport` design 10): the grinder, crushed stone, the mixer,
/// gravel, survey stakes, the wheel and the two carts are locked until
/// their missions.
#[test]
fn the_town_tree_locks_its_recipes_until_their_missions() {
    let mut app = app(WorldSave::memory_only());
    reach_town(&mut app);
    app.update();
    let chain = app.world().resource::<TownChain>().0.clone();
    let recipes = app.world().resource::<Recipes>();
    let locks = app.world().resource::<crate::missions::RecipeLocks>();
    let mut built = Vec::new();
    for m in &chain.missions {
        for code in &m.unlocks {
            if recipes.0.get(code).is_some() {
                assert!(locks.0.contains_key(code), "{code} waits on {}", m.id);
                built.push(code.as_str());
            }
        }
    }
    built.sort_unstable();
    assert_eq!(
        built,
        [
            "crushed-stone",
            "grinder",
            "hand-cart",
            "horse-cart",
            "mixed-gravel",
            "mixer",
            "survey-stakes",
            "wheel",
        ]
    );
    let starter = &app.world().resource::<MissionChain>().0;
    for code in locks.0.keys() {
        assert!(
            starter.names(code) || built.contains(&code.as_str()),
            "{code} is a tree's lock"
        );
    }
    assert!(locks.0.contains_key("hand-cart") && locks.0.contains_key("horse-cart"));
    assert!(
        !locks.0.contains_key("wheel"),
        "open while paving is pending"
    );
}

/// Design 8 and 9: a lamp lit at dusk counts toward a taken lamps job; an
/// abandoned job leaves the taken list and shows on its board again, and an
/// abandoned lost villager's record is cleared and they are let go.
#[test]
fn a_lamp_counts_and_an_abandoned_job_goes_back_to_the_board() {
    use pbd_core::economy::jobs::{Board, JobKind};
    let mut app = app(WorldSave::memory_only());
    let cfg = config();
    let here = harbour(12);
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 1,
    };
    let lamps = board.make(here.as_town(), JobKind::Lamps, 40, 0).unwrap();
    shop(&mut app, |s| s.take(lamps.clone())).unwrap();
    app.world_mut().write_message(Did::new(
        Kind::Light,
        pbd_core::economy::jobs::scoped(12, "lamp"),
        1,
    ));
    app.update();
    let eco = app.world().resource::<Economy>();
    assert_eq!(eco.jobs.counts(lamps.id), [1]);
    shop(&mut app, |s| s.abandon(lamps.id)).unwrap();
    let eco = app.world().resource::<Economy>();
    assert!(eco.jobs.taken.is_empty());
    assert!(!eco.jobs.gone().contains(&lamps.id), "back on its board");
    // A lost villager's job, with its villager placed.
    let lost_job = board.make(here.as_town(), JobKind::Lost, 41, 0).unwrap();
    shop(&mut app, |s| s.take(lost_job.clone())).unwrap();
    let lost = pbd_core::economy::follow::Lost {
        site: 12,
        at: [1.0, 2.0, 3.0],
        following: true,
        person: 4,
    };
    app.world_mut()
        .resource_mut::<WorldSave>()
        .store(
            &Author::Player(0),
            vec![crate::economy::special::lost_record(lost_job.id, &lost)],
        )
        .unwrap();
    {
        let world = app.world();
        let dirs = crate::economy::special::lost_directions(
            world.resource::<Economy>(),
            world.resource::<WorldSave>(),
        );
        assert_eq!(dirs.len(), 1, "the map and compass mark them");
    }
    shop(&mut app, |s| s.abandon(lost_job.id)).unwrap();
    let world = app.world();
    assert!(crate::economy::special::lost_of(world.resource::<WorldSave>(), lost_job.id).is_none());
    assert_eq!(
        world.resource::<Economy>().let_go,
        [pbd_core::townsfolk::PersonId { site: 12, index: 4 }]
    );
    assert!(
        crate::economy::special::lost_directions(
            world.resource::<Economy>(),
            world.resource::<WorldSave>()
        )
        .is_empty()
    );
}
