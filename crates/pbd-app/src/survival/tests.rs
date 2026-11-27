//! Survival in the game: the server's bars through the save, the dropped
//! pack, wear and the worn slots, each across a reload where it is saved.

use super::*;
use crate::saves::{self, WorldSave};
use pbd_core::inventory::{Item, Stack, Tier};
use pbd_core::survival::temperature::{Exposure, feel};
use pbd_core::survival::water::sip;
use pbd_core::survival::{Note, WaterSource, WorldMode};
use pbd_core::terrain::Material;
use std::path::PathBuf;

fn temporary(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pbd-survival-{name}-{}-{}",
        std::process::id(),
        saves::now_unix_s()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// A Survival world on disk, and the way back into it.
fn survival_world(name: &str) -> (PathBuf, saves::Slot) {
    let root = temporary(name);
    let slot = saves::create_in(&root, name, 4242, WorldMode::Survival).unwrap();
    (root, slot)
}

fn reopen(root: &std::path::Path) -> WorldSave {
    let listed = saves::list(root);
    WorldSave::open(root.to_path_buf(), listed[0].clone())
}

fn dive() -> Conditions {
    Conditions {
        head_in_water: true,
        in_deep_water: true,
        exposure: Exposure {
            air_c: 20.0,
            in_water: true,
            ..Exposure::default()
        },
        ..Conditions::default()
    }
}

#[test]
fn an_old_save_opens_as_explorer_and_has_no_bars() {
    let root = temporary("old");
    // `create` makes an Explorer world, written as before modes.
    let slot = saves::create(&root, "Old", 4242).unwrap();
    assert_eq!(slot.identity.as_ref().unwrap().mode, None);
    let mut save = WorldSave::open(root.clone(), slot);
    assert_eq!(save.mode(), WorldMode::Explorer);
    assert!(save.survive(&Conditions::default(), 1.0).is_none());
    assert!(save.vitals().is_none());
    assert!(
        save.survival_act(Act::Hit(Cause::Fall, 50.0)).is_none(),
        "nothing hurts in Explorer"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_long_dive_drains_health_and_surfacing_refills_breath() {
    let (root, slot) = survival_world("dive");
    let mut save = WorldSave::open(root.clone(), slot);
    assert_eq!(save.mode(), WorldMode::Survival);
    assert_eq!(save.vitals().unwrap().health, rules().health.max_pts);
    for _ in 0..400 {
        save.survive(&dive(), 0.1).unwrap();
    }
    let v = save.vitals().unwrap();
    assert_eq!(v.breath.breath_s, 0.0);
    assert!((v.health - 50.0).abs() < 1.0, "{}", v.health);
    let surfaced = Conditions {
        head_in_water: false,
        ..dive()
    };
    for _ in 0..40 {
        save.survive(&surfaced, 0.1).unwrap();
    }
    let v = save.vitals().unwrap();
    assert!((v.breath.breath_s - rules().breath.under_s).abs() < 1e-3);
    // The bars come back with the world, at most VITALS_EVERY_S behind.
    save.drain();
    let reopened = reopen(&root);
    let back = reopened.vitals().unwrap();
    assert!(
        (back.health - v.health).abs() < 3.0,
        "{} {}",
        back.health,
        v.health
    );
    assert!(back.health < 60.0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_sea_sip_moves_nothing_and_a_river_sip_fills() {
    let save = &mut WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    let cfg = rules();
    save.survive(&Conditions::default(), 0.0);
    {
        let v = save.vitals().unwrap();
        assert_eq!(v.water as f32, cfg.water.max);
    }
    // Down to 40 by a thirsty walk.
    let mut v = save.vitals().unwrap();
    v.water = 40.0;
    let record = StoredRecord::of(pbd_server::survival::VITALS_KIND, 0, 1, &v);
    save.store(&pbd_core::records::Author::Player(0), vec![record]);
    save.reset_server();
    let sea = sip(cfg, WaterSource::Sea, 40.0);
    assert_eq!(sea.note, Some(Note::SaltWater));
    assert!(!sea.counts);
    save.survival_act(Act::Drink(sea));
    assert_eq!(
        save.vitals().unwrap().water,
        40.0,
        "salt water gives nothing"
    );
    assert_eq!(pbd_core::missions::drink_name(WaterSource::Sea), None);
    let river = sip(cfg, WaterSource::Fresh, 40.0);
    assert!(river.counts);
    save.survival_act(Act::Drink(river));
    assert!(save.vitals().unwrap().water > 40.0);
    assert_eq!(
        pbd_core::missions::drink_name(WaterSource::Fresh),
        Some("fresh")
    );
}

#[test]
fn a_sprint_drains_the_bar_and_rest_refills_it() {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    let sprint = Conditions {
        effort: Effort::Sprint,
        ..Conditions::default()
    };
    for _ in 0..300 {
        save.survive(&sprint, 0.1);
    }
    let spent = save.vitals().unwrap().stamina.pts;
    assert!((spent - 70.0).abs() < 1.0, "{spent}");
    for _ in 0..100 {
        save.survive(&Conditions::default(), 0.1);
    }
    assert_eq!(
        save.vitals().unwrap().stamina.pts,
        rules().stamina.max_pts,
        "rested to full"
    );
}

#[test]
fn a_winded_swimmer_floats_and_cannot_sprint_or_climb_out() {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    let swim = Conditions {
        effort: Effort::SwimSprint,
        in_deep_water: true,
        ..Conditions::default()
    };
    let mut winded = false;
    for _ in 0..1000 {
        let tick = save.survive(&swim, 0.1).unwrap();
        winded |= tick.events.contains(&Event::Winded);
    }
    assert!(winded);
    let v = save.vitals().unwrap();
    let (stamina, control) = v.limits(rules(), true);
    let limits = WalkerLimits { stamina, control };
    assert!(limits.floats(), "a winded swimmer floats");
    assert!(!limits.can_sprint());
    assert!(!limits.stamina.can_climb_out);
    assert!(limits.jump_speed_mult() < 1.0);
    // Free in Explorer.
    assert!(WalkerLimits::default().can_sprint());
    assert!(!WalkerLimits::default().floats());
}

#[test]
fn a_hat_and_a_cloak_change_feel_and_are_worn_from_the_hand() {
    let cfg = &rules().body;
    let desert = Exposure {
        air_c: 38.0,
        sunlight_wm2: 900.0,
        humidity: 0.15,
        wind_mps: 2.0,
        ..Exposure::default()
    };
    let mut hatted = desert.clone();
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    let mut carried = Slots::new();
    carried.give(
        Item::Good(pbd_core::goods::Good::by_code("straw-hat").unwrap()),
        1,
    );
    assert!(wear_held(&mut save, &mut carried));
    assert!(carried.held().is_none(), "the hat left the hand");
    hatted.worn = WornRecord::of(&save).worn();
    assert!(
        feel(cfg, &hatted) < feel(cfg, &desert),
        "a hat halves the sun"
    );
    let tundra = Exposure {
        air_c: -18.0,
        wind_mps: 6.0,
        ..Exposure::default()
    };
    carried.give(
        Item::Good(pbd_core::goods::Good::by_code("wool-cloak").unwrap()),
        1,
    );
    assert!(wear_held(&mut save, &mut carried));
    let mut cloaked = tundra.clone();
    cloaked.worn = WornRecord::of(&save).worn();
    assert!(feel(cfg, &cloaked) > feel(cfg, &tundra), "a cloak warms");
    // Not worn in Explorer.
    let mut explorer = WorldSave::memory_only();
    carried.give(
        Item::Good(pbd_core::goods::Good::by_code("straw-hat").unwrap()),
        1,
    );
    assert!(!wear_held(&mut explorer, &mut carried));
}

#[test]
fn a_death_drops_every_item_in_one_bag_that_survives_a_reload_and_is_picked_up_whole() {
    let (root, slot) = survival_world("bag");
    let mut carried = Slots::new();
    carried.give(Item::Block(Material::Dirt), 12);
    carried.give(
        Item::Good(pbd_core::goods::Good::by_code("hide").unwrap()),
        2,
    );
    carried.give(Item::Fish(3), 1);
    let before = carried.clone();
    let at = WorldPos::new(
        crate::sim_origin::HOME,
        bevy::math::DVec3::new(1.0, 2.0, 3.0),
    );
    {
        let mut save = WorldSave::open(root.clone(), slot);
        save.survive(&Conditions::default(), 0.0);
        let died = save.survival_act(Act::Hit(Cause::Fall, 500.0)).unwrap();
        assert!(died.events.contains(&Event::Died(Cause::Fall)));
        let id = drop_pack(&mut save, &mut carried, at).expect("a bag");
        assert_eq!(
            carried,
            Slots::new(),
            "the pack, the hotbar included, is empty"
        );
        assert_eq!(bags(&save).len(), 1, "one bag");
        assert_eq!(bags(&save)[0].0, id);
        save.survival_act(Act::WakeUp);
        save.drain();
    }
    let mut save = reopen(&root);
    assert_eq!(save.carried.as_ref(), Some(&Slots::new()));
    let lying = bags(&save);
    assert_eq!(lying.len(), 1, "the bag is still there");
    assert_eq!(lying[0].1.at, at);
    let v = save.vitals().unwrap();
    assert_eq!(v.health, rules().health.max_pts, "woke at full health");
    assert_eq!(v.food_fp as f32, rules().food.wake_fp, "and half food");
    let mut hands = Slots::new();
    assert!(pick_bag(&mut save, &mut hands, lying[0].0));
    for item in [
        Item::Block(Material::Dirt),
        Item::Good(pbd_core::goods::Good::by_code("hide").unwrap()),
        Item::Fish(3),
    ] {
        assert_eq!(hands.count(item), before.count(item), "{item:?}");
    }
    assert!(bags(&save).is_empty(), "picked up whole");
    save.drain();
    assert!(bags(&reopen(&root)).is_empty(), "and it stays picked up");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn what_does_not_fit_stays_in_the_bag() {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    let mut carried = Slots::new();
    carried.give(Item::Block(Material::Dirt), 5);
    let at = WorldPos::new(crate::sim_origin::HOME, bevy::math::DVec3::ZERO);
    let id = drop_pack(&mut save, &mut carried, at).unwrap();
    // Hands full of stone: nothing fits.
    let mut full = Slots::new();
    for _ in 0..CARRIED {
        full.give(Item::Block(Material::Stone), 9999);
    }
    let held = full.clone();
    let moved = pick_bag(&mut save, &mut full, id);
    assert_eq!(moved, full != held);
    if !moved {
        assert_eq!(bags(&save).len(), 1, "the bag waits");
    }
}

#[test]
fn a_death_wakes_at_a_laid_bedroll() {
    let mut save = WorldSave::memory_only();
    assert_eq!(spawn_point(&save), None, "the world's spawn");
    let at = WorldPos::new(
        crate::sim_origin::HOME,
        bevy::math::DVec3::new(0.0, 300.0, 0.0),
    );
    assert!(set_spawn(&mut save, at));
    assert_eq!(spawn_point(&save), Some(at));
}

#[test]
fn a_tool_wears_breaks_survives_a_reload_and_is_repaired() {
    let (root, slot) = survival_world("wear");
    let tier = Tier::Stone;
    let cfg = rules();
    let wearer = pbd_core::survival::wear::Wearer::Tool(Tool::Pickaxe, tier);
    let full = wearer.max_uses(cfg);
    {
        let mut save = WorldSave::open(root.clone(), slot);
        assert!(
            !WearBook::of(&save).broken(Tool::Pickaxe, tier),
            "an old save reads full"
        );
        let mut broke = false;
        for _ in 0..full {
            broke |= spend_wear(
                &mut save,
                Tool::Pickaxe,
                tier,
                pbd_core::survival::wear::Action::BlockBroken { block_tier: None },
            ) == Spend::Broke;
        }
        assert!(broke);
        assert!(WearBook::of(&save).broken(Tool::Pickaxe, tier));
        save.drain();
    }
    let mut save = reopen(&root);
    assert!(
        WearBook::of(&save).broken(Tool::Pickaxe, tier),
        "broken across a reload"
    );
    // At the workbench: half the head's materials, taken from the hands.
    let inputs = pbd_core::survival::wear::repair_inputs(cfg, &[("block:stone".into(), 3)]);
    assert_eq!(inputs, vec![("block:stone".to_string(), 2)]);
    // Three stone in hand, the two the repair takes gone: one left.
    let mut carried = Slots::new();
    carried.give(Item::Block(Material::Stone), 1);
    assert!(repair_tool(&mut save, Tool::Pickaxe, tier, &carried));
    assert!(!WearBook::of(&save).broken(Tool::Pickaxe, tier));
    assert_eq!(
        WearBook::of(&save).wear(Tool::Pickaxe, tier).uses_left,
        full
    );
    assert_eq!(
        save.carried.as_ref(),
        Some(&carried),
        "saved with the hands"
    );
    // An Explorer world never wears.
    let mut explorer = WorldSave::memory_only();
    assert_eq!(
        spend_wear(
            &mut explorer,
            Tool::Pickaxe,
            tier,
            pbd_core::survival::wear::Action::TreeFelled
        ),
        Spend::None
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_smith_repairs_for_the_share_worn() {
    let cfg = rules();
    let tier = Tier::Stone;
    let wearer = pbd_core::survival::wear::Wearer::Tool(Tool::Axe, tier);
    let mut wear = pbd_core::survival::wear::Wear::full(cfg, &wearer);
    let half = wearer.max_uses(cfg) / 2;
    for _ in 0..half {
        wear.spend(
            cfg,
            WorldMode::Survival,
            &wearer,
            pbd_core::survival::wear::Action::TreeFelled,
        );
    }
    let coins = pbd_core::survival::wear::repair_coins(cfg, 40, wear, &wearer);
    assert!((19..=21).contains(&coins), "{coins}");
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    assert!(repair_tool(&mut save, Tool::Axe, tier, &Slots::new()));
    assert_eq!(
        WearBook::of(&save).wear(Tool::Axe, tier),
        pbd_core::survival::wear::Wear::full(cfg, &wearer)
    );
    let _ = Stack::new(Item::Block(Material::Dirt), 1);
}

// ------------------------------------------------ water and places (6.3, 6.6)

fn survival_memory() -> WorldSave {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    save.survive(&Conditions::default(), 0.0);
    save
}

fn held(good: &str) -> Slots {
    let mut carried = Slots::new();
    carried.give(Item::Good(pbd_core::goods::Good::by_code(good).unwrap()), 1);
    carried
}

#[test]
fn the_still_makes_fresh_water_only_beside_a_lit_fire() {
    let mut save = survival_memory();
    let cfg = rules();
    let at = (77, 12);
    let mut carried = held("sea-bucket");
    assert_eq!(
        places::use_still(&mut save, &mut carried, at, 40.0),
        Some(places::StillUse::Poured)
    );
    assert_eq!(
        carried.held().map(|s| s.item),
        Some(Item::Good(pbd_core::goods::Good::BUCKET)),
        "the bucket comes back empty"
    );
    assert_eq!(places::still_at(&save, at.0, at.1).salt_sips, 4);
    // Nothing to drink yet, and no fire: it says so.
    let mut hands = Slots::new();
    assert_eq!(
        places::use_still(&mut save, &mut hands, at, 40.0),
        Some(places::StillUse::NeedsFire)
    );
    // Long without a fire boils nothing.
    let mut still = places::still_at(&save, at.0, at.1);
    for _ in 0..600 {
        places::step_still(&mut save, at, &mut still, false, false, 1.0);
    }
    assert_eq!(places::still_at(&save, at.0, at.1).fresh_sips, 0);
    // Beside a lit fire, one sip a boil.
    let boil = cfg.still.boil_s_per_sip as usize;
    for _ in 0..boil {
        places::step_still(&mut save, at, &mut still, true, false, 1.0);
    }
    assert_eq!(places::still_at(&save, at.0, at.1).fresh_sips, 1);
    match places::use_still(&mut save, &mut hands, at, 40.0) {
        Some(places::StillUse::Drank(sipped)) => {
            assert!(sipped.water > 0.0 && sipped.counts && sipped.note.is_none());
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(places::still_at(&save, at.0, at.1).fresh_sips, 0);
    // Explorer has no still to use.
    let mut explorer = WorldSave::memory_only();
    assert_eq!(
        places::use_still(&mut explorer, &mut held("sea-bucket"), at, 40.0),
        None
    );
}

#[test]
fn a_bucket_filled_at_the_sea_says_salt_water_and_a_fresh_one_empties_in_four_sips() {
    let mut save = survival_memory();
    let mut sea = held("sea-bucket");
    let sipped = places::drink_bucket(&mut save, &mut sea, 40.0).unwrap();
    assert_eq!(sipped.water, 0.0);
    assert_eq!(sipped.note, Some(Note::SaltWater));
    assert!(!sipped.counts);
    let mut fresh = held("water-bucket");
    let sips = rules().drinks.bucket.sips;
    for k in 0..sips {
        let sipped = places::drink_bucket(&mut save, &mut fresh, 40.0).unwrap();
        assert!(sipped.water > 0.0 && sipped.counts, "sip {k}");
    }
    assert_eq!(
        fresh.held().map(|s| s.item),
        Some(Item::Good(pbd_core::goods::Good::BUCKET)),
        "the last sip leaves an empty bucket"
    );
    assert_eq!(save.carried.as_ref(), Some(&fresh), "saved with the hands");
}

#[test]
fn a_cactus_gives_three_sips_through_a_straw_and_is_dry_after() {
    let mut save = survival_memory();
    let mut carried = Slots::new();
    carried.give(Item::Good(pbd_core::goods::Good::STRAW), 3);
    for k in 0..rules().drinks.cactus.sips {
        let (sipped, broke) = places::sip_cactus(&mut save, &mut carried, 9, 2.0, 40.0).unwrap();
        assert!(sipped.water > 0.0 && sipped.counts, "sip {k}");
        assert!(!broke);
    }
    let (dry, _) = places::sip_cactus(&mut save, &mut carried, 9, 2.0, 40.0).unwrap();
    assert_eq!(dry.note, Some(Note::Dry));
    assert_eq!(dry.water, 0.0);
    // A day on, a sip is back.
    let (back, _) = places::sip_cactus(&mut save, &mut carried, 9, 3.1, 40.0).unwrap();
    assert!(back.water > 0.0);
    // No straw, no sip.
    assert!(places::sip_cactus(&mut save, &mut Slots::new(), 9, 9.0, 40.0).is_none());
    // A straw wears a use a sip.
    let worn = WearBook::of(&save)
        .tools
        .get(places::STRAW_WEAR)
        .copied()
        .unwrap();
    assert_eq!(worn.uses_left, rules().wear.straw_uses - 4);
}

#[test]
fn a_death_wakes_the_player_at_a_laid_bedroll() {
    let (root, slot) = survival_world("bedroll");
    let at = WorldPos::new(
        crate::sim_origin::HOME,
        bevy::math::DVec3::new(0.0, 4801.0, 0.0),
    );
    {
        let mut save = WorldSave::open(root.clone(), slot);
        save.survive(&Conditions::default(), 0.0);
        assert_eq!(
            spawn_point(&save),
            None,
            "the world's spawn until one is laid"
        );
        let mut carried = held("bedroll");
        assert!(places::lay_bedroll(&mut save, &mut carried, at, Vec3::X));
        assert!(carried.held().is_none(), "the bedroll left the hand");
        assert_eq!(spawn_point(&save), Some(at));
        // A death: the wake reads the bedroll.
        let died = save.survival_act(Act::Hit(Cause::Fall, 500.0)).unwrap();
        assert!(died.events.contains(&Event::Died(Cause::Fall)));
        save.survival_act(Act::WakeUp);
        save.drain();
    }
    let mut save = reopen(&root);
    assert_eq!(spawn_point(&save), Some(at), "laid across a reload");
    assert_eq!(save.vitals().unwrap().health, rules().health.max_pts);
    // Taken up, the wake is the world's spawn again.
    let mut hands = Slots::new();
    assert!(places::take_up_bedroll(&mut save, &mut hands));
    assert_eq!(hands.count(Item::Good(pbd_core::goods::Good::BEDROLL)), 1);
    assert_eq!(spawn_point(&save), None);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_new_survival_start_goes_near_fresh_water_and_a_watered_one_stays() {
    let r = 4800.0;
    let start = Vec3::Y;
    // Water 600 m off along x: the start moves to land within 150 m of it.
    let water = (Vec3::Y * (600.0f32 / r).cos() + Vec3::X * (600.0f32 / r).sin()).normalize();
    let fresh = |d: Vec3| d.angle_between(water) * r < 20.0;
    let moved = places::start_near_fresh_water(
        start,
        r,
        fresh,
        |d| (d + Vec3::Z * (60.0 / r)).normalize(),
        |d| d,
    );
    assert!(moved != start);
    assert!(moved.angle_between(water) * r <= places::FRESH_WITHIN_M);
    // Water already in reach: the start stands.
    let near = (Vec3::Y + Vec3::X * (100.0 / r)).normalize();
    let kept = places::start_near_fresh_water(
        start,
        r,
        |d| d.angle_between(near) * r < 20.0,
        |d| d,
        |d| d,
    );
    assert_eq!(kept, start);
    // No fresh water anywhere: the start stands.
    assert_eq!(
        places::start_near_fresh_water(start, r, |_| false, |d| d, |d| d),
        start
    );
}

#[test]
fn a_river_is_brackish_only_within_six_cells_of_the_sea_along_its_flow() {
    let cfg = crate::planet::terrain_config();
    let settings = cfg
        .rivers
        .as_ref()
        .expect("the shipped generator has rivers");
    let map = pbd_core::rivers::map(cfg, settings);
    // A river that reaches the sea: walk up from its sea cell's centre
    // toward its last river cell, a metre at a time, to the mouth.
    let mut checked = 0;
    for k in map.rivers().step_by(97) {
        let (centre, ..) = map.describe(k);
        let course = map.course(centre, 0);
        let (Some(sea), Some(last)) = (
            course.iter().rev().find(|s| !s.river),
            course.iter().rev().find(|s| s.river),
        ) else {
            continue;
        };
        let gap = sea.centre.angle_between(last.centre) * cfg.radius_m;
        let mouth = (0..gap as i32).find_map(|m| {
            let t = m as f32 / gap;
            let d = sea.centre.lerp(last.centre, t).normalize();
            (pbd_core::planet_gen::fresh_water_at(cfg, d) && pbd_core::rivers::at(cfg, d).is_some())
                .then_some(d)
        });
        let Some(mouth) = mouth else { continue };
        assert!(
            places::brackish_at(cfg, mouth),
            "the river's first fresh metre from the sea is brackish"
        );
        // Far up the river (its source end), the water is fresh.
        let up = course[0].centre;
        if pbd_core::planet_gen::fresh_water_at(cfg, up)
            && up.angle_between(mouth) * cfg.radius_m > 100.0
        {
            assert!(!places::brackish_at(cfg, up), "a river's spring is fresh");
            assert_eq!(
                places::water_source(up * (cfg.radius_m + 30.0), 30.0),
                WaterSource::Fresh
            );
        }
        checked += 1;
        if checked >= 3 {
            break;
        }
    }
    assert!(checked > 0, "no river mouth found to check");
}

#[test]
fn a_town_repair_takes_coins_and_restores_the_tool() {
    let mut save = survival_memory();
    let tier = Tier::Stone;
    for _ in 0..30 {
        spend_wear(
            &mut save,
            Tool::Axe,
            tier,
            pbd_core::survival::wear::Action::TreeFelled,
        );
    }
    let coins = places::repair_coins_for(&save, Tool::Axe, tier, 12);
    assert!(coins > 0);
    let mut poor = pbd_core::economy::coins::Purse::new(u64::from(coins) - 1);
    assert!(places::repair_for_coins(&mut save, &mut poor, Tool::Axe, tier, coins).is_err());
    assert!(WearBook::of(&save).wear(Tool::Axe, tier).uses_left < full_uses(Tool::Axe, tier));
    let mut purse = pbd_core::economy::coins::Purse::new(50);
    places::repair_for_coins(&mut save, &mut purse, Tool::Axe, tier, coins).unwrap();
    assert_eq!(purse.coins, 50 - u64::from(coins));
    assert_eq!(
        WearBook::of(&save).wear(Tool::Axe, tier).uses_left,
        full_uses(Tool::Axe, tier)
    );
    let saved: pbd_core::economy::coins::Purse = save
        .records
        .get(pbd_core::economy::coins::PURSE_KIND, 0)
        .and_then(|r| r.read())
        .unwrap();
    assert_eq!(saved, purse, "the purse is saved with the repair");
}

fn full_uses(tool: Tool, tier: Tier) -> u32 {
    pbd_core::survival::wear::Wearer::Tool(tool, tier).max_uses(rules())
}

#[test]
fn a_workbench_repair_takes_half_the_head_of_the_tools_recipe() {
    let file: pbd_core::station::RecipeFile =
        ron::from_str(include_str!("../../../../assets/config/recipes.ron")).unwrap();
    let book = file.resolve().unwrap().for_mode(true);
    let price = places::repair_price(&book, rules(), Tool::Pickaxe, Tier::Stone).unwrap();
    assert!(!price.is_empty(), "a stone pickaxe's head has a price");
    for &(item, _) in &price {
        if let Item::Good(g) = item {
            assert!(
                !rules().wear.handle_goods.iter().any(|h| h == g.code()),
                "no handle in the price: {}",
                g.code()
            );
        }
    }
    let mut save = survival_memory();
    let full = full_uses(Tool::Pickaxe, Tier::Stone);
    for _ in 0..full {
        spend_wear(
            &mut save,
            Tool::Pickaxe,
            Tier::Stone,
            pbd_core::survival::wear::Action::BlockBroken { block_tier: None },
        );
    }
    assert!(WearBook::of(&save).broken(Tool::Pickaxe, Tier::Stone));
    let mut short = Slots::new();
    assert!(!places::repair_at_bench(
        &mut save,
        &mut short,
        Tool::Pickaxe,
        Tier::Stone,
        &price
    ));
    let mut carried = Slots::new();
    for &(item, n) in &price {
        carried.give(item, n);
    }
    assert!(places::repair_at_bench(
        &mut save,
        &mut carried,
        Tool::Pickaxe,
        Tier::Stone,
        &price
    ));
    assert_eq!(carried, Slots::new(), "the price left the hands");
    assert!(!WearBook::of(&save).broken(Tool::Pickaxe, Tier::Stone));
}

/// Wear `code` down to broken by landed hits.
fn break_weapon(save: &mut WorldSave, code: &str) {
    for _ in 0..10_000 {
        if crate::combat::spend_weapon_wear(save, code, pbd_core::survival::wear::Action::HitLanded)
            == pbd_core::survival::wear::Spend::Broke
        {
            return;
        }
    }
    panic!("{code} never broke");
}

#[test]
fn a_broken_weapon_is_repaired_at_the_workbench_for_half_its_head() {
    use places::Mend;
    let file: pbd_core::station::RecipeFile =
        ron::from_str(include_str!("../../../../assets/config/recipes.ron")).unwrap();
    let book = file.resolve().unwrap().for_mode(true);
    let sword = pbd_core::goods::Good::by_code("bronze-sword").unwrap();
    let mut save = survival_memory();
    break_weapon(&mut save, "bronze-sword");
    assert!(crate::combat::WeaponWearBook::of(&save).broken("bronze-sword"));
    // The sword in hand is what is mended, before any tool in the slot.
    let (mend, share) = places::worn_in_hand(
        &save,
        Some(Item::Good(sword)),
        Some((Tool::Pickaxe, Tier::Stone)),
    )
    .unwrap();
    assert_eq!(mend, Mend::Weapon(sword));
    assert_eq!(share, 0.0);
    // Half the head: two of the sword's three bronze ingots, no handle.
    let price = places::mend_price(&book, rules(), mend).unwrap();
    assert_eq!(
        price,
        vec![(
            Item::Good(pbd_core::goods::Good::by_code("bronze-ingot").unwrap()),
            2
        )]
    );
    let mut short = Slots::new();
    assert!(!places::mend_at_bench(&mut save, &mut short, mend, &price));
    assert!(crate::combat::WeaponWearBook::of(&save).broken("bronze-sword"));
    let mut carried = Slots::new();
    for &(item, n) in &price {
        carried.give(item, n);
    }
    assert!(places::mend_at_bench(&mut save, &mut carried, mend, &price));
    assert_eq!(carried, Slots::new(), "the price left the hands");
    let wear = crate::combat::WeaponWearBook::of(&save).wear("bronze-sword");
    assert_eq!(
        wear.uses_left,
        pbd_core::survival::wear::Wearer::Weapon("bronze-sword".into()).max_uses(rules())
    );
    // Full now: nothing to mend.
    assert!(places::worn_in_hand(&save, Some(Item::Good(sword)), None).is_none());
    // The bow's recipe is all handle: half of everything, never free.
    let bow = places::mend_price(&book, rules(), Mend::Weapon(pbd_core::goods::Good::BOW)).unwrap();
    assert!(!bow.is_empty(), "a bow's repair is never free");
}

#[test]
fn a_town_repairs_a_weapon_for_coins_and_the_repair_survives_a_reload() {
    use places::Mend;
    let (dir, slot) = survival_world("weapon-repair");
    let spear = pbd_core::goods::Good::by_code("stone-spear").unwrap();
    let mend = Mend::Weapon(spear);
    let mut purse = pbd_core::economy::coins::Purse::new(50);
    {
        let mut save = WorldSave::open(dir.clone(), slot);
        for _ in 0..30 {
            crate::combat::spend_weapon_wear(
                &mut save,
                "stone-spear",
                pbd_core::survival::wear::Action::HitLanded,
            );
        }
        let coins = places::mend_coins_for(&save, mend, 10);
        assert!(coins > 0 && coins <= 10, "{coins}");
        let mut poor = pbd_core::economy::coins::Purse::new(u64::from(coins) - 1);
        assert!(places::mend_for_coins(&mut save, &mut poor, mend, coins).is_err());
        assert!(mend.share(&save) < 1.0, "a short purse mends nothing");
        places::mend_for_coins(&mut save, &mut purse, mend, coins).unwrap();
        assert_eq!(purse.coins, 50 - u64::from(coins));
        assert_eq!(mend.share(&save), 1.0);
        assert_eq!(places::mend_coins_for(&save, mend, 10), 0);
    }
    let save = reopen(&dir);
    assert_eq!(mend.share(&save), 1.0, "the repair survives a reload");
    let saved: pbd_core::economy::coins::Purse = save
        .records
        .get(pbd_core::economy::coins::PURSE_KIND, 0)
        .and_then(|r| r.read())
        .unwrap();
    assert_eq!(saved, purse, "the purse is saved with the repair");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_explorer_weapon_is_never_offered_a_repair() {
    let save = WorldSave::memory_only();
    let sword = pbd_core::goods::Good::by_code("iron-sword").unwrap();
    assert!(places::worn_in_hand(&save, Some(Item::Good(sword)), None).is_none());
}

#[test]
fn sea_water_poured_above_the_sea_stays_salt() {
    let cfg = crate::planet::terrain_config();
    // Land well above the sea that the generator made no water on: water
    // gathered there reads fresh.
    let dry = (0..4000)
        .map(|k| {
            let t = k as f32 * 0.37;
            Vec3::new(t.cos(), (k as f32 * 0.011).sin() * 0.6, t.sin()).normalize()
        })
        .find(|&d| {
            !pbd_core::planet_gen::fresh_water_at(cfg, d)
                && pbd_core::planet_gen::surface_altitude(cfg, d) > cfg.sea_level_m + 5.0
        })
        .expect("dry land above the sea");
    let top = pbd_core::planet_gen::surface_altitude(cfg, dry) + 0.5;
    let point = dry * (cfg.radius_m + top);
    let mut save = survival_memory();
    assert_eq!(
        places::water_source_in(&save, point, top),
        WaterSource::Fresh
    );
    // A sea bucket poured there: that water, and the water it runs into a
    // cell on, is the sea's.
    assert!(places::note_salt_pour(
        &mut save,
        (5, 9),
        WorldPos::new(crate::sim_origin::HOME, point.as_dvec3())
    ));
    assert_eq!(places::water_source_in(&save, point, top), WaterSource::Sea);
    let beside = point + point.normalize().any_orthonormal_vector() * places::CELL_M;
    assert_eq!(
        places::water_source_in(&save, beside, top),
        WaterSource::Sea
    );
    let sipped = sip(rules(), places::water_source_in(&save, point, top), 40.0);
    assert_eq!(sipped.note, Some(Note::SaltWater));
    // Far from the pour, gathered water is fresh as before.
    let far = point + point.normalize().any_orthonormal_vector() * 40.0;
    assert_eq!(places::water_source_in(&save, far, top), WaterSource::Fresh);
}
