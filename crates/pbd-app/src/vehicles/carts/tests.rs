//! `land-transport` tasks 5.3 to 5.5: a cart and its hold through a save,
//! letting go and taking up again, hitching and unhitching, the horse
//! bought and kept, and saves from before the carts unchanged.

use super::*;
use crate::saves::WorldSave;
use pbd_core::inventory::{Slots, Stack};
use pbd_core::vehicle::hold::{Place, shift};

const R: f64 = 4800.0;

fn at() -> DVec3 {
    DVec3::new(0.3, 0.9, 0.2).normalize() * (R + 12.0)
}

fn world_with(save: WorldSave) -> World {
    let mut world = World::new();
    world.insert_resource(save);
    world.insert_resource(Fleet::new(Default::default()));
    world.insert_resource(Carts::default());
    world.insert_resource(Pace::default());
    world.insert_resource(PlanetRenderFrame::default());
    world.insert_resource(Hotbar(Slots::new()));
    world.insert_resource(Assets::<StandardMaterial>::default());
    world.insert_resource(Messages::<Did>::default());
    world.insert_resource(crate::config::HusbandryConfig(Default::default()));
    world
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("pbd-carts-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

/// Task 5.3: a horse cart set down with goods in its hold, and a horse in
/// its shafts, is where it was, loaded, with the same horse, when the world
/// is opened again; the good left the pack in the same line.
#[test]
fn a_loaded_cart_is_where_it_was_on_reopening() {
    let root = temp_root("reload");
    let slot = crate::saves::create(&root, "Carts", 5).unwrap();
    let log = Item::Good(Good::LOG);
    let mut c = Cart::new(41, CartKind::HorseCart, at(), DVec3::X);
    c.horse = Some(7);
    {
        let mut save = WorldSave::open(root.clone(), slot.clone());
        let mut slots = Slots::new();
        slots.set(12, Some(Stack::new(log, 40)));
        assert!(save.store_with_hands(c.record(), CART_SCHEMA, &slots));
        let mut hold = super::super::hold::read_sized(&save, c.id, 24);
        assert_eq!(hold.slots.len(), 24);
        shift(
            &mut slots,
            &mut hold,
            Place::Carried(12),
            Place::Hold(20),
            40,
        );
        assert!(super::super::hold::store(&mut save, c.id, &hold, &slots));
        save.drain();
    }
    let listed = crate::saves::list(&root);
    let save = WorldSave::open(root.clone(), listed[0].clone());
    let back = cart::carts(&save.records);
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].id, 41);
    assert_eq!(back[0].kind, CartKind::HorseCart);
    assert!(back[0].at.distance(c.at) < 1e-6, "where it was left");
    assert_eq!(back[0].horse, Some(7), "the same horse in its shafts");
    let hold = super::super::hold::read_sized(&save, 41, 24);
    assert_eq!(hold.get(20), Some(Stack::new(log, 40)), "loaded as it was");
    assert_eq!(save.carried.as_ref().and_then(|s| s.get(12)), None);
    let _ = std::fs::remove_dir_all(&root);
}

/// Task 5.4: letting go of a hand cart and stepping down from a horse cart
/// change who is with it, never whether it is: the cart is in the world
/// and in the save where it stood, and is taken up again as before.
#[test]
fn letting_go_keeps_the_cart_and_it_is_taken_again() {
    let mut world = world_with(WorldSave::memory_only());
    world.resource_mut::<Carts>().loaded = Some(0);
    for kind in CartKind::ALL {
        let id = 60 + kind as u64;
        let e = spawn(&mut world, Cart::new(id, kind, at(), DVec3::Z));
        let (take, leave): (fn(&mut World, Entity), fn(&mut World, Entity)) = match kind {
            CartKind::HandCart => (take_handles, let_go),
            CartKind::HorseCart => (take_seat, step_down),
        };
        for _ in 0..2 {
            take(&mut world, e);
            let c = world.resource::<Carts>();
            assert!(c.holding == Some(e) || c.seated == Some(e));
            leave(&mut world, e);
            let c = world.resource::<Carts>();
            assert!(c.holding.is_none() && c.seated.is_none());
            assert_eq!(
                world.resource::<Pace>().cap_mps,
                None,
                "the walker's own pace"
            );
            assert!(world.get::<CartThing>(e).is_some(), "still in the world");
            let saved = cart::carts(&world.resource::<WorldSave>().records);
            let it = saved.iter().find(|s| s.id == id).expect("in the save");
            assert!(it.at.distance(at()) < 1e-6);
            assert_eq!(it.kind, kind);
        }
    }
}

/// Task 5.5: hitching writes one record, the cart's, naming the horse, so
/// the save is never half-hitched; unhitching takes it out with its animal
/// record alive and its home beside the cart.
#[test]
fn a_horse_is_hitched_and_unhitched_and_never_lost() {
    let mut world = world_with(WorldSave::memory_only());
    let horse = AnimalBody {
        sp: 3,
        via: animal::Via::Bought,
        ..AnimalBody::default()
    };
    world
        .resource_mut::<WorldSave>()
        .store(&Author::Player(0), vec![horse.record(9)]);
    let e = spawn(
        &mut world,
        Cart::new(70, CartKind::HorseCart, at(), DVec3::X),
    );
    let lines = world.resource::<WorldSave>().records.len();
    assert!(hitch_horse(&mut world, e, 9));
    let save = world.resource::<WorldSave>();
    assert_eq!(save.records.len(), lines + 1, "one record: the cart's");
    assert!(cart::hitched(&save.records).contains(&9));
    unhitch(&mut world, e);
    let save = world.resource::<WorldSave>();
    assert!(cart::hitched(&save.records).is_empty());
    let back: AnimalBody = save.records.get(animal::ANIMAL, 9).unwrap().read().unwrap();
    assert_eq!(back.state, animal::Life::Alive, "the horse lives");
    assert_eq!(back.via, animal::Via::Bought);
    let beside = roads::nearest_key(at().normalize().as_vec3()).unwrap();
    let home = pbd_core::cell_key::key_direction(back.home).unwrap();
    let near = pbd_core::cell_key::key_direction(beside).unwrap();
    assert!(home.angle_between(near) * 4800.0 < 8.0, "beside the cart");
    assert!(world.get::<CartThing>(e).unwrap().cart.horse.is_none());
}

/// Task 5.5: a horse bought is a kept animal of the horse species, bought,
/// alive, and it is there when the world is opened again.
#[test]
fn a_bought_horse_is_kept_through_a_reload() {
    let root = temp_root("horse");
    let slot = crate::saves::create(&root, "Horse", 9).unwrap();
    let id = {
        let mut world = world_with(WorldSave::open(root.clone(), slot.clone()));
        let id = buy_horse(&mut world, at(), DVec3::X, false).expect("bought");
        world.resource::<WorldSave>().drain();
        id
    };
    let save = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
    let body: AnimalBody = save
        .records
        .get(animal::ANIMAL, id)
        .unwrap()
        .read()
        .unwrap();
    let roster = crate::config::HusbandryConfig(Default::default());
    let sp = &roster.0.roster(pbd_core::fauna::HOME_BODY)[body.sp as usize];
    assert_eq!(sp.id, "horse");
    assert_eq!(body.via, animal::Via::Bought);
    assert_eq!(body.state, animal::Life::Alive);
    let _ = std::fs::remove_dir_all(&root);
}

/// Old saves load unchanged: a raft's ten-slot hold reads as it did, a
/// world with no cart records has no carts and no hitched animal, and an
/// animal record written before horses (no `via: Bought`) reads as before.
#[test]
fn old_saves_load_unchanged() {
    let mut save = WorldSave::memory_only();
    let log = Item::Good(Good::LOG);
    let mut slots = Slots::new();
    slots.set(3, Some(Stack::new(log, 5)));
    let mut hold = super::super::hold::read(&save, 4);
    assert_eq!(hold.slots.len(), 10, "a boat's hold is ten");
    shift(&mut slots, &mut hold, Place::Carried(3), Place::Hold(9), 5);
    assert!(super::super::hold::store(&mut save, 4, &hold, &slots));
    let text = save.records.get("hold", 4).unwrap().body.clone();
    assert_eq!(ron::from_str::<Vec<String>>(&text).unwrap().len(), 10);
    assert_eq!(
        super::super::hold::read(&save, 4).get(9),
        Some(Stack::new(log, 5))
    );
    assert!(cart::carts(&save.records).is_empty());
    assert!(cart::hitched(&save.records).is_empty());
    let old: AnimalBody = ron::from_str("(sp: 2, via: Fed, home: 12)").unwrap();
    assert_eq!(old.via, animal::Via::Fed);
    assert_eq!(old.sp, 2);
}

/// The drawn root stands up where the cart does and faces its heading.
#[test]
fn a_cart_is_drawn_facing_its_heading() {
    let c = Cart::new(1, CartKind::HandCart, at(), DVec3::Z);
    let q = facing(&c);
    assert!((q * DVec3::Y).dot(c.up()) > 0.999);
    assert!((q * DVec3::NEG_Z).dot(c.heading) > 0.999);
}

/// The plodder's index in the home roster.
fn plodder_sp(world: &World) -> u16 {
    species_index(world, "plodder").expect("a plodder in the roster")
}

/// Design 13: a kept plodder is hitched by the same one-record hitch as a
/// horse, its name in the message, and the cart then runs at its pace.
#[test]
fn a_plodder_is_hitched_and_pulls_at_its_own_pace() {
    let mut world = world_with(WorldSave::memory_only());
    let plodder = AnimalBody {
        sp: plodder_sp(&world),
        ..AnimalBody::default()
    };
    world
        .resource_mut::<WorldSave>()
        .store(&Author::Player(0), vec![plodder.record(12)]);
    let e = spawn(
        &mut world,
        Cart::new(71, CartKind::HorseCart, at(), DVec3::X),
    );
    let lines = world.resource::<WorldSave>().records.len();
    assert!(hitch_horse(&mut world, e, 12));
    assert_eq!(
        world.resource::<WorldSave>().records.len(),
        lines + 1,
        "one record: the cart's"
    );
    assert!(cart::hitched(&world.resource::<WorldSave>().records).contains(&12));
    assert_eq!(animal_name(&world, 12).into_string(), "Plodder");
    let specs = world.resource::<Fleet>().specs.carts;
    assert_eq!(pulled_spec(&world, Some(12)).road_mps, specs.plodder_mps);
    assert!(pulled_spec(&world, Some(12)).road_mps < specs.horse_cart.road_mps);
}

/// Design 13: a plodder in the shafts is the same plodder after a reload,
/// and still pulls at its own pace.
#[test]
fn a_hitched_plodder_is_there_after_a_reload() {
    let root = temp_root("plodder");
    let slot = crate::saves::create(&root, "Plodder", 11).unwrap();
    {
        let mut world = world_with(WorldSave::open(root.clone(), slot.clone()));
        let id =
            keep_animal(&mut world, "plodder", at(), DVec3::X, false, Vec::new()).expect("kept");
        let e = spawn(
            &mut world,
            Cart::new(72, CartKind::HorseCart, at(), DVec3::X),
        );
        assert!(hitch_horse(&mut world, e, id));
        world.resource::<WorldSave>().drain();
    }
    let save = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
    let back = cart::carts(&save.records);
    let id = back[0].horse.expect("an animal in the shafts");
    let mut world = world_with(save);
    let body = animal_body(&world, id).expect("its record");
    assert_eq!(body.sp, plodder_sp(&world));
    assert_eq!(
        pulled_spec(&world, Some(id)).road_mps,
        world.resource::<Fleet>().specs.carts.plodder_mps
    );
    let _ = &mut world;
    let _ = std::fs::remove_dir_all(&root);
}

/// A world with a walker standing at `at()` and the farmyard.
/// `at()`'s direction on the ground the rides stand on.
fn on_ground(world: &World) -> DVec3 {
    let up = at().normalize();
    up * ground_at(world, up * 4800.0)
}

/// The farmyard's local point for a body-local one.
fn yard_local(world: &World, p: DVec3) -> Vec3 {
    (world.resource::<PlanetRenderFrame>().physics_center() + p).as_vec3()
}

fn riding_world(save: WorldSave) -> World {
    let mut world = world_with(save);
    world.insert_resource(Farmyard::default());
    world.insert_resource(Time::<()>::default());
    let feet = on_ground(&world);
    let at = yard_local(&world, feet + feet.normalize() * f64::from(HALF_HEIGHT));
    world.spawn((Walker, Position(at), LinearVelocity::default()));
    world
}

/// A kept plodder beside the walker, in the yard.
fn plodder_beside(world: &mut World) -> u64 {
    let feet = on_ground(world);
    let up = feet.normalize();
    let beside = feet + up.any_orthonormal_vector() * 1.0;
    let id = keep_animal(world, "plodder", beside, DVec3::X, false, Vec::new()).expect("kept");
    let body = animal_body(world, id).unwrap();
    let local = yard_local(world, beside);
    world
        .resource_mut::<Farmyard>()
        .take_back(id, &body, local, up.as_vec3(), Vec3::X, 0.0);
    id
}

/// Design 14: F beside a kept plodder mounts it (its record says ridden,
/// the walker is on its back), and F again gets off it: its record is
/// unridden, it is back in the yard, and nothing about it is lost. Riding
/// is not hitching.
#[test]
fn a_plodder_is_mounted_and_got_off() {
    let mut world = riding_world(WorldSave::memory_only());
    let id = plodder_beside(&mut world);
    let (_, feet) = walker_feet(&mut world).unwrap();
    let (_, found) = riding::mountable(&world, feet).expect("a plodder to ride");
    assert_eq!(found, id);
    assert!(riding::mount(&mut world, id));
    assert!(
        animal_body(&world, id).unwrap().ridden,
        "its record says so"
    );
    assert_eq!(
        world.resource::<Carts>().riding.as_ref().unwrap().animal,
        id
    );
    assert!(cart::hitched(&world.resource::<WorldSave>().records).is_empty());
    assert!(
        riding::mountable(&world, feet).is_none(),
        "nothing else is mounted"
    );
    riding::get_off(&mut world);
    let (_, feet) = walker_feet(&mut world).unwrap();
    let body = animal_body(&world, id).unwrap();
    assert!(!body.ridden);
    assert_eq!(body.state, animal::Life::Alive);
    assert!(world.resource::<Carts>().riding.is_none());
    assert_eq!(world.resource::<Pace>().cap_mps, None);
    assert!(
        world
            .resource::<Farmyard>()
            .kept_pose(id, yard_local(&world, feet), 4.0)
            .is_some(),
        "back in the yard beside the walker"
    );
}

/// Design 14: an animal in a cart's shafts is not mounted until it is
/// unhitched.
#[test]
fn a_hitched_animal_is_not_ridden() {
    let mut world = riding_world(WorldSave::memory_only());
    let id = plodder_beside(&mut world);
    let e = spawn(
        &mut world,
        Cart::new(73, CartKind::HorseCart, at(), DVec3::X),
    );
    assert!(hitch_horse(&mut world, e, id));
    let (_, feet) = walker_feet(&mut world).unwrap();
    assert!(riding::mountable(&world, feet).is_none());
    assert!(!riding::mount(&mut world, id));
}

/// Design 14: a save closed while riding opens with the walker on foot and
/// the animal unridden beside them.
#[test]
fn a_reload_while_riding_comes_back_beside_it() {
    let root = temp_root("ride");
    let slot = crate::saves::create(&root, "Ride", 13).unwrap();
    let id = {
        let mut world = riding_world(WorldSave::open(root.clone(), slot.clone()));
        let id = plodder_beside(&mut world);
        assert!(riding::mount(&mut world, id));
        world.resource::<WorldSave>().drain();
        id
    };
    let save = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
    let serial = save.serial();
    let mut world = riding_world(save);
    assert!(animal_body(&world, id).unwrap().ridden, "saved ridden");
    world.resource_mut::<Carts>().loaded = Some(serial);
    riding::restore(&mut world);
    assert!(!animal_body(&world, id).unwrap().ridden);
    assert!(world.resource::<Carts>().riding.is_none(), "on foot");
    let (_, feet) = walker_feet(&mut world).unwrap();
    assert!(
        world
            .resource::<Farmyard>()
            .kept_pose(id, yard_local(&world, feet), 3.0)
            .is_some(),
        "beside the walker"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Design 14: an animal record from before riding reads unridden, and one
/// written unridden holds no new field, so old saves read and write as
/// they did.
#[test]
fn old_animal_records_are_unchanged_by_riding() {
    let old: AnimalBody = ron::from_str("(sp: 2, via: Fed, home: 12)").unwrap();
    assert!(!old.ridden);
    let written = old.record(5).body;
    assert!(!written.contains("ridden"), "{written}");
}

/// `town-missions-tree` task 4.1, the carts' half: a horse hitched raises
/// `hitch` (`horse`), and metres hauled raise `haul` (`cart`) and nothing
/// for none.
#[test]
fn the_carts_raise_hitch_and_haul() {
    use pbd_core::missions::Kind;
    let mut world = world_with(WorldSave::memory_only());
    let horse = AnimalBody {
        sp: 3,
        via: animal::Via::Bought,
        ..AnimalBody::default()
    };
    world
        .resource_mut::<WorldSave>()
        .store(&Author::Player(0), vec![horse.record(9)]);
    let e = spawn(
        &mut world,
        Cart::new(71, CartKind::HorseCart, at(), DVec3::X),
    );
    assert!(hitch_horse(&mut world, e, 9));
    let raised: Vec<(Kind, String, u32)> = world
        .resource_mut::<Messages<Did>>()
        .drain()
        .map(|d| (d.0.kind, d.0.what, d.0.count))
        .collect();
    assert_eq!(raised, vec![(Kind::Hitch, "horse".to_string(), 1)]);
    let haul = missions::hauled(12).unwrap();
    assert_eq!(
        (haul.kind, haul.what.as_str(), haul.count),
        (Kind::Haul, "cart", 12)
    );
    assert!(missions::hauled(0).is_none());
}
