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

pub(super) fn harbour(site: u32) -> TownInfo {
    TownInfo {
        site,
        kind: SiteKind::Harbour,
        name: "Saltmere".into(),
        neighbours: vec![site + 1],
        roads_to: vec![],
    }
}

/// An app counting the missions and the town jobs into a memory-only world:
/// what the game runs, without a window.
pub(super) fn app() -> App {
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
pub(super) fn shop<R>(app: &mut App, f: impl FnOnce(&mut Shop) -> R) -> R {
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
        // Three to five places, less a lamps job, shown only from dusk.
        assert!((2..=5).contains(&posted.len()), "{}", posted.len());
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

/// `townsfolk` task 7.5: a lost villager taken as a job is found by talking
/// to them, follows the player at their pace, waits at deep water, and at
/// the town's wall is home: the job counts it, its record is cleared, and
/// handing it in pays.
#[test]
fn a_lost_villager_follows_waits_at_deep_water_and_counts_at_the_wall() {
    use bevy::math::DVec3;
    use special::{Walked, find_lost, lost_of, lost_record, walk};
    let mut app = app();
    let cfg = config();
    let town = harbour(40);
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 3,
    };
    assert!(cfg.jobs.posted.contains(&JobKind::Lost), "posted on boards");
    let job = board.make(town.as_town(), JobKind::Lost, 77, 9).unwrap();
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    // Stood out of town; not yet following.
    let mut lost = pbd_core::economy::follow::Lost {
        site: town.site,
        at: [0.0, 0.0, 0.0],
        following: false,
        person: 3,
    };
    {
        let world = app.world_mut();
        let mut save = world.remove_resource::<WorldSave>().unwrap();
        save.store(&Author::Player(0), vec![lost_record(job.id, &lost)])
            .unwrap();
        let eco = world.resource::<Economy>().clone();
        assert!(!find_lost(&mut save, &eco, town.site, 2), "someone else");
        assert!(find_lost(&mut save, &eco, town.site, 3), "found");
        assert!(!find_lost(&mut save, &eco, town.site, 3), "found once");
        lost = lost_of(&save, job.id).unwrap();
        assert!(lost.following);
        world.insert_resource(save);
    }
    // Deep water from x = 20 m, the town's wall inside x = -30 m.
    let dry = |p: DVec3| p.x < 20.0;
    let home = |p: DVec3| p.x < -30.0;
    let flat = |p: DVec3| p;
    // The player walks off at 1.5 m/s; the villager keeps behind.
    let mut player = DVec3::new(6.0, 0.0, 0.0);
    let mut moved = 0.0;
    for _ in 0..20 {
        player.x += 0.15;
        if let Walked::Moved(m) = walk(&mut lost, player, 1.5, 0.1, dry, flat, home) {
            moved += m;
        }
    }
    assert!(moved > 2.0, "it followed {moved} m");
    assert!(player.x - lost.at[0] >= pbd_core::economy::follow::GAP_M - 1e-6);
    // Across deep water: it waits at the edge.
    player = DVec3::new(45.0, 0.0, 0.0);
    for _ in 0..200 {
        walk(&mut lost, player, 0.0, 0.1, dry, flat, home);
    }
    assert!(lost.at[0] < 20.0 && lost.at[0] > 18.0, "{:?}", lost.at);
    assert_eq!(
        walk(&mut lost, player, 0.0, 0.1, dry, flat, home),
        Walked::Still
    );
    // Back over the wall: home.
    player = DVec3::new(-40.0, 0.0, 0.0);
    let mut came = false;
    for _ in 0..600 {
        if walk(&mut lost, player, 4.0, 0.1, dry, flat, home) == Walked::Home {
            came = true;
            break;
        }
    }
    assert!(came, "home at {:?}", lost.at);
    // What the game writes on the way in: the job counts it.
    app.world_mut().write_message(Did::new(
        Kind::Escort,
        pbd_core::economy::jobs::scoped(town.site, "villager"),
        1,
    ));
    app.update();
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
    let paid = shop(&mut app, |s| s.hand_in(&town, job.id)).unwrap();
    assert_eq!(app.world().resource::<Economy>().purse.coins, paid.reward);
}

/// A one-cell house with a door on side 0, and a second the same.
fn small_town(site: u32) -> pbd_core::settlement::record::Town {
    let house = |name: &str| -> pbd_core::settlement::record::Building {
        serde_json::from_value(serde_json::json!({
            "name": name, "kit": "timber", "cells": [[0, 0]], "floor": 0,
            "storeys": 1, "tall": 1, "doors": [[0, 0, 0, 0]], "windows": [],
            "roof": "gable", "pitch": 1.0, "chimney": null, "stair_cells": [],
        }))
        .unwrap()
    };
    pbd_core::settlement::record::Town {
        site,
        template: "village".into(),
        layout: 1,
        terrace: 10,
        anchor: (0, 0),
        cells: Vec::new(),
        levels: Vec::new(),
        buildings: vec![house("a"), house("b")],
        over_sea: Vec::new(),
    }
}

/// `townsfolk` task 7.6: the world breaks a wall of a town building for a
/// repair job only where the player has changed nothing of it (so never a
/// piece the player placed or touched), and placing the planks back mends
/// it and counts the job.
#[test]
fn a_repair_breaks_no_players_work_and_placing_planks_counts() {
    use pbd_core::settlement::record::{BUILDING_RECORD, building_id, to_records};
    use special::{break_wall, mend, repair_of};
    let mut app = app();
    let cfg = config();
    let site = 50;
    let town_info = harbour(site);
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 3,
    };
    assert!(
        cfg.jobs.posted.contains(&JobKind::Repair),
        "posted on boards"
    );
    let job = board
        .make(town_info.as_town(), JobKind::Repair, 91, 4)
        .unwrap();
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    let mut town = small_town(site);
    assert_eq!(town.buildings[0].breakable().len(), 5, "six sides, a door");
    let world = app.world_mut();
    let mut save = world.remove_resource::<WorldSave>().unwrap();
    save.store(&Author::Creation, to_records(&town)).unwrap();
    // The player has changed both buildings: nothing is broken.
    let mut touched = town.clone();
    for (b, building) in touched.buildings.iter_mut().enumerate() {
        building.windows.push([0, 0, 2, 0]);
        save.store(
            &Author::Player(0),
            vec![pbd_core::records::Record::of(
                BUILDING_RECORD,
                building_id(site, b as u32),
                pbd_core::settlement::record::RECORD_SCHEMA,
                building,
            )],
        )
        .unwrap();
    }
    let mut held = touched.clone();
    assert_eq!(break_wall(&mut save, &job, &mut held), None);
    assert_eq!(
        held, touched,
        "the player's buildings are as they left them"
    );
    assert!(repair_of(&save, job.id).is_none());
    // A town the player has not touched: one wall is broken, the world's.
    let other = 51;
    let mut fresh = small_town(other);
    save.store(&Author::Creation, to_records(&fresh)).unwrap();
    let repair = break_wall(&mut save, &job, &mut fresh).expect("a wall to break");
    let building = &fresh.buildings[repair.building as usize];
    assert_eq!(building.broken, vec![repair.wall]);
    assert!(!building.doors.contains(&repair.wall), "never the doorway");
    let stored: pbd_core::settlement::record::Building = save
        .records
        .get(BUILDING_RECORD, building_id(other, repair.building))
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(stored.broken, vec![repair.wall], "stored as the world's");
    town = fresh;
    // Mended: two planks, the wall back, the job's record mended.
    let mut slots = Slots::new();
    assert!(!mend(&mut save, &mut slots, job.id, &mut town), "no planks");
    slots.give(Item::Good(Good::PLANKS), 3);
    assert!(mend(&mut save, &mut slots, job.id, &mut town));
    assert_eq!(slots.count(Item::Good(Good::PLANKS)), 1);
    assert!(town.buildings[repair.building as usize].broken.is_empty());
    assert!(repair_of(&save, job.id).unwrap().mended);
    assert!(
        !mend(&mut save, &mut slots, job.id, &mut town),
        "mended once"
    );
    world.insert_resource(save);
    // Placing it counts the job, as the game writes it.
    app.world_mut().write_message(Did::new(
        Kind::Repair,
        pbd_core::economy::jobs::scoped(other, "piece"),
        1,
    ));
    app.update();
    // Its town is the one the repair was in: here the job's town is 50
    // and the wall in 51, so it counts only for a piece in 50.
    assert!(!app.world().resource::<Economy>().jobs.is_done(job.id));
    app.world_mut().write_message(Did::new(
        Kind::Repair,
        pbd_core::economy::jobs::scoped(site, "piece"),
        1,
    ));
    app.update();
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
}

/// A stored road of the world's from `from` to `to`, `n` cells east of the
/// shipped start, on ground at 30 m.
fn a_road(from: u32, to: u32, n: usize) -> pbd_core::roads::Road {
    let start = Vec3::new(0.31, 0.62, 0.72).normalize();
    let east = Vec3::Y.cross(start).normalize();
    let mut cells: Vec<pbd_core::roads::RoadCell> = Vec::new();
    let mut k = 0.0;
    while cells.len() < n {
        let d = (start + east * (k / 4800.0)).normalize();
        k += 1.0;
        let key = pbd_core::roads::nearest_key(d).unwrap();
        if cells.last().is_some_and(|c| c.key == key) {
            continue;
        }
        cells.push(pbd_core::roads::RoadCell {
            key,
            base_m: 30,
            kind: pbd_core::roads::Kind::Ground,
            raised: false,
            cut_fill: 0,
            bed: Default::default(),
            up: None,
            from: None,
        });
    }
    pbd_core::roads::Road {
        from_site: from,
        to_site: to,
        surface: pbd_core::roads::Surface::Gravel,
        cells,
    }
}

/// `world/roads`, jobs on the roads (task 8.5, design 46): a town a road
/// leaves posts a rockfall; taking it has the world drop rock on the road
/// through the yield check; the dig's own edit (the rock back to air)
/// counts each cell once, the road is drawn again there, and the job is
/// handed in and paid. A town no road leaves posts none.
#[test]
fn a_rockfall_is_dropped_on_the_road_dug_clear_and_paid() {
    use bevy::ecs::system::RunSystemOnce;
    use road_jobs::{drop_rockfall, dug_clear, rockfall_of};
    let mut app = app();
    let cfg = config();
    let site = 60;
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 3,
    };
    assert!(cfg.jobs.posted.contains(&JobKind::Rockfall));
    assert!(
        board
            .make(harbour(site).as_town(), JobKind::Rockfall, 5, 4)
            .is_none(),
        "no road, no rockfall"
    );
    let road = a_road(site, site + 1, 20);
    {
        let mut save = app.world_mut().resource_mut::<WorldSave>();
        save.store(
            &crate::roads::author(),
            vec![pbd_core::roads::to_record(&road)],
        )
        .unwrap();
    }
    let town = TownInfo {
        roads_to: roads_to(app.world().resource::<WorldSave>(), site),
        ..harbour(site)
    };
    assert_eq!(town.roads_to, vec![site + 1]);
    let job = board.make(town.as_town(), JobKind::Rockfall, 5, 4).unwrap();
    let wanted = job.wanted();
    assert!((2..=4).contains(&wanted));
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    // The world drops it, once.
    let laid = {
        let mut save = app.world_mut().resource_mut::<WorldSave>();
        drop_rockfall(&mut save, &job).expect("rock on the road")
    };
    assert_eq!(laid.len(), wanted as usize);
    let fall = rockfall_of(app.world().resource::<WorldSave>(), job.id).unwrap();
    assert!(
        fall.cells.iter().all(
            |(c, _)| road.cells[pbd_core::economy::road_jobs::ROCKFALL_FROM..]
                .iter()
                .any(|r| r.key == *c)
        ),
        "past the town's own cells, on the road"
    );
    assert!(dug_clear(app.world().resource::<WorldSave>()).is_empty());
    // The player digs each with the dig's own edit; each counts once.
    let carried = pbd_core::inventory::Slots::new();
    for (n, &(cell, layer)) in fall.cells.iter().enumerate() {
        let dig = pbd_core::edits::Edit {
            cell,
            layer,
            material: pbd_core::terrain::Material::Air,
        };
        assert!(
            app.world_mut()
                .resource_mut::<WorldSave>()
                .accept(dig, None, &carried)
        );
        app.world_mut()
            .run_system_once(road_jobs::rockfalls)
            .unwrap();
        app.world_mut()
            .run_system_once(road_jobs::rockfalls)
            .unwrap();
        app.update();
        let counts = app.world().resource::<Economy>().jobs.counts(job.id);
        assert_eq!(counts, vec![n as u32 + 1], "counted once");
    }
    // The road is the road again where its rock is gone.
    assert_eq!(
        dug_clear(app.world().resource::<WorldSave>()).len(),
        fall.cells.len()
    );
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
    let paid = shop(&mut app, |s| s.hand_in(&town, job.id)).unwrap();
    assert_eq!(app.world().resource::<Economy>().purse.coins, paid.reward);
}

/// `world/roads`, a trader walked to the next town (task 8.5, design 46):
/// placed on the road's first cell toward the job's town; waits to be met,
/// walks the road while the player keeps up and waits when they fall back;
/// reaching the other town counts the job, which is handed in where it was
/// taken.
#[test]
fn a_trader_leads_the_player_along_the_road_and_the_escort_is_paid() {
    use bevy::math::DVec3;
    use pbd_core::economy::road_jobs::lead;
    use road_jobs::{escort_of, escort_record, place_trader, trader_point};
    let mut app = app();
    let cfg = config();
    let site = 70;
    let road = a_road(site + 1, site, 30);
    {
        let mut save = app.world_mut().resource_mut::<WorldSave>();
        save.store(
            &crate::roads::author(),
            vec![pbd_core::roads::to_record(&road)],
        )
        .unwrap();
    }
    let town = TownInfo {
        roads_to: roads_to(app.world().resource::<WorldSave>(), site),
        ..harbour(site)
    };
    let board = Board {
        jobs: &cfg.jobs,
        trade: &cfg.trade,
        values: &cfg.values,
        seed: 3,
    };
    let job = board.make(town.as_town(), JobKind::Escort, 9, 2).unwrap();
    assert_eq!(job.to, Some(site + 1));
    shop(&mut app, |s| s.take(job.clone())).unwrap();
    let roles = ["merchant", "reeve", "farmer", "smith", "fisher"];
    let mut escort = place_trader(app.world().resource::<WorldSave>(), &job, &roles, &cfg)
        .expect("a trader to walk with");
    assert!(escort.person >= 2, "never the trader or the job giver");
    assert_eq!(
        escort.route[0],
        road.cells.last().unwrap().key,
        "from this town"
    );
    let config = *crate::planet::terrain_config();
    let cell_m = 2.83;
    let mut player = trader_point(&escort, &config).unwrap() + DVec3::new(30.0, 0.0, 0.0);
    // Not met yet: waits.
    let (a, s, _) = lead(
        escort.along,
        escort.started,
        escort.route.len(),
        cell_m,
        30.0,
        0.0,
        0.1,
    );
    assert_eq!((a, s), (0.0, false));
    // Met, and walked with at 1.5 m/s: on along the road.
    let mut moved_m = 0.0;
    for _ in 0..100 {
        let here = trader_point(&escort, &config).unwrap();
        player = here - (here - player).normalize() * 3.0;
        let (a, s, _) = lead(
            escort.along,
            escort.started,
            escort.route.len(),
            cell_m,
            (player - here).length(),
            1.5,
            0.1,
        );
        moved_m += (a - escort.along) * cell_m;
        (escort.along, escort.started) = (a, s);
    }
    assert!(escort.started && moved_m > 10.0, "led {moved_m} m");
    // Left behind: waits.
    let before = escort.along;
    let (a, _, m) = lead(
        escort.along,
        true,
        escort.route.len(),
        cell_m,
        20.0,
        0.0,
        1.0,
    );
    assert!(!m && a == before);
    // Stored, it reads back where it stopped.
    {
        let mut save = app.world_mut().resource_mut::<WorldSave>();
        save.store(
            &Author::Player(0),
            vec![escort_record(job.id, Some(&escort))],
        )
        .unwrap();
        assert_eq!(escort_of(&save, job.id), Some(escort.clone()));
    }
    // At the other town the game writes the count; the job is done and paid
    // where it was taken.
    app.world_mut().write_message(Did::new(
        Kind::Escort,
        pbd_core::economy::jobs::scoped(site + 1, "trader"),
        1,
    ));
    app.update();
    assert!(app.world().resource::<Economy>().jobs.is_done(job.id));
    let paid = shop(&mut app, |s| s.hand_in(&town, job.id)).unwrap();
    assert_eq!(paid.reward, 60, "40 coins times the board's 1.5");
}
