//! Survival in the game: the server's bars through the save, the dropped
//! pack, wear and the worn slots, each across a reload where it is saved.

use super::*;
use crate::saves::{self, WorldSave};
use pbd_core::inventory::{Item, Stack, Tier};
use pbd_core::survival::temperature::{Exposure, feel};
use pbd_core::survival::water::sip;
use pbd_core::survival::{Note, WorldMode};
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
