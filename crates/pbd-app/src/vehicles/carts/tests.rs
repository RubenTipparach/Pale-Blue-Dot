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
