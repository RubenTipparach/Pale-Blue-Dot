use super::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn roster() -> Vec<LandSpecies> {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/husbandry.ron"
    ))
    .unwrap();
    let settings: pbd_core::husbandry::HusbandrySettings = ron::from_str(&text).unwrap();
    settings.roster("home").to_vec()
}

fn survival_save() -> WorldSave {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    save
}

#[test]
fn each_weapon_names_its_swing() {
    let w = |g: Good| in_hand(Some(Item::Good(g)), None, false);
    assert_eq!(w(Good::STONE_SPEAR).left, Some(SwingKind::Thrust));
    assert_eq!(w(Good::STONE_SPEAR).right, Some(SwingKind::Throw));
    assert_eq!(w(Good::IRON_HUNTING_AXE).left, Some(SwingKind::Chop));
    assert_eq!(w(Good::BRONZE_SWORD).left, Some(SwingKind::Slash));
    assert_eq!(w(Good::BOW).left, Some(SwingKind::Bow));
    assert_eq!(w(Good::STONE_ARROW).left, None);
    assert_eq!(
        w(Good::BRONZE_SWORD).held,
        Held::Weapon("bronze-sword".into())
    );
    assert_eq!(in_hand(None, None, false).held, Held::Hand);
    assert_eq!(
        blade_of(&Held::Weapon("iron-hunting-axe".into())),
        Some(Blade::HuntingAxe)
    );
    assert_eq!(blade_of(&Held::Hand), None);
}

/// Task 4.5: a struck villager lowers the town's standing and brings the
/// guards near; the far guard stays at its post; the struck one flees.
#[test]
fn a_struck_villager_brings_the_guards_and_lowers_the_towns_standing() {
    let cfg = crate::survival::rules();
    let radius = f64::from(crate::planet::PLANET_RADIUS);
    let r = crate::planet::PLANET_RADIUS;
    let at = |east_m: f32| Vec3::new(east_m / r, 1.0, 0.0).normalize();
    let mut folk = Townsfolk::default();
    folk.stand_for_test(
        9,
        vec![
            crate::townsfolk::test_person(9, 0, at(0.0), false),
            crate::townsfolk::test_person(9, 1, at(12.0), true),
            crate::townsfolk::test_person(9, 2, at(90.0), true),
        ],
        radius,
    );
    let mut save = survival_save();
    assert_eq!(standing_of(&save, 9), 0);
    let villager = PersonId { site: 9, index: 0 };
    assert_eq!(
        strike_person(cfg, &mut save, WorldMode::Survival, 9),
        Some(-10)
    );
    assert_eq!(standing_of(&save, 9), -10);
    let mut combat = Combat::default();
    start_fight(&mut combat, &folk, cfg, villager);
    assert_eq!(
        combat.guards(),
        1,
        "the near guard comes, the far one stays"
    );
    // The guards walk: the fight's system drives them toward the player.
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(combat)
        .insert_resource(folk)
        .insert_resource(save)
        .insert_resource(crate::survival::Survival::default());
    app.world_mut().spawn((
        Walker,
        Position(at(4.0) * (r + crate::walking::HALF_HEIGHT)),
    ));
    app.world_mut().run_system_once(town_fight).unwrap();
    let folk = app.world().resource::<Townsfolk>();
    assert!(
        folk.driven(PersonId { site: 9, index: 1 }),
        "the near guard comes"
    );
    assert!(!folk.driven(PersonId { site: 9, index: 2 }));
    assert!(folk.driven(villager), "the struck villager flees");
}

/// Task 4.5: in Explorer a swing at a townsperson does nothing.
#[test]
fn a_swing_at_a_townsperson_in_explorer_does_nothing() {
    let cfg = crate::survival::rules();
    let mut save = WorldSave::memory_only();
    assert_eq!(strike_person(cfg, &mut save, WorldMode::Explorer, 9), None);
    assert_eq!(standing_of(&save, 9), 0);
    assert!(save.records.get(standing::KIND, 9).is_none());
}

/// Task 4.7's rule half: a swing never lands on another player; the aim
/// only ever names the yard's animals and the townsfolk.
#[test]
fn a_swing_at_another_player_does_nothing() {
    use pbd_core::survival::melee;
    assert!(!melee::lands(melee::Target::Player));
}

/// Tasks 1.6 and 4.4: the hit lands at the swing's hit moment, on the fox
/// the aim meets then, for the weapon's table value; the drawing is told
/// the swing and never feeds it back.
#[test]
fn the_hit_lands_at_the_swings_hit_moment() {
    let roster = roster();
    let fox_sp = roster.iter().position(|s| s.id == "red-fox").unwrap() as u16;
    let r = crate::planet::PLANET_RADIUS;
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            10,
        )))
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<Combat>()
        .init_resource::<CombatClaim>()
        .init_resource::<SwingNow>()
        .init_resource::<crate::planet::PlanetRenderFrame>()
        .init_resource::<Drops>()
        .init_resource::<Hotbar>()
        .add_message::<Landed>()
        .add_message::<crate::missions::Did>()
        .insert_resource(survival_save())
        .insert_resource(crate::carried_light::HeldItem(Some(Item::Good(
            Good::BRONZE_HUNTING_AXE,
        ))))
        .insert_resource(crate::fish::Body("home".into()))
        .insert_resource(crate::config::HusbandryConfig({
            let text = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/config/husbandry.ron"
            ))
            .unwrap();
            ron::from_str(&text).unwrap()
        }))
        .add_systems(Update, swing);
    // The eye 1.6 m over the ground, looking north; the fox 1.4 m ahead.
    let eye = Vec3::new(0.0, r + 1.6, 0.0);
    app.world_mut().spawn((
        WalkingCamera,
        GlobalTransform::from(Transform::from_translation(eye).looking_to(Vec3::NEG_Z, Vec3::Y)),
    ));
    let mut yard = Farmyard::default();
    let fox = yard.place_wild(
        fox_sp,
        Vec3::new(0.0, r + 1.6 - 0.25, -1.4),
        Vec3::Y,
        Vec3::X,
        1,
    );
    yard.hold_still(fox, true);
    app.insert_resource(yard);
    let cfg = crate::survival::rules();
    let health = |app: &App| {
        app.world()
            .resource::<Farmyard>()
            .health_of(cfg, &roster, fox)
            .unwrap()
            .1
    };
    app.update();
    // The press.
    app.world_mut().resource_mut::<Combat>().scripted = Some((true, false));
    app.update();
    app.world_mut().resource_mut::<Combat>().scripted = Some((false, false));
    assert!(app.world().resource::<CombatClaim>().0);
    let hit_s = 0.6 * cfg.weapons.hit_share;
    let mut t = 0.0;
    let mut landed_at = None;
    let mut drawn = false;
    while t < 0.6 {
        app.update();
        t += 0.01;
        drawn |= matches!(
            app.world().resource::<SwingNow>().0,
            Some((SwingKind::Chop, Timing::Melee { .. }))
        );
        if landed_at.is_none() && health(&app) < 1.0 {
            landed_at = Some(t);
        }
    }
    let landed_at = landed_at.expect("the hit landed");
    assert!(
        (landed_at - hit_s).abs() <= 0.021,
        "landed {landed_at}, rule {hit_s}"
    );
    assert!((health(&app) - 6.0 / 30.0).abs() < 1e-5, "24 off 30");
    assert!(drawn, "the drawing was told the chop");
    assert_eq!(app.world().resource::<SwingNow>().0, None);
}

/// A thrown spear leaves the hand on release, flies, and lands as a drop.
#[test]
fn a_thrown_spear_lands_as_a_drop() {
    let cfg = crate::survival::rules();
    let from = WorldPos::new(
        pbd_core::world_frame::FrameId(0),
        DVec3::new(0.0, 9601.6, 0.0),
    );
    let mut p =
        Projectile::throw_spear(cfg, "stone-spear", from, DVec3::new(1.0, 0.3, 0.0)).unwrap();
    let mut out = Flight::Flying;
    for _ in 0..600 {
        out = p.step::<usize>(cfg, 9.8, 1.0 / 60.0, |_, b| {
            (b.length() < 9600.0).then_some(Struck::Ground(b))
        });
        if out != Flight::Flying {
            break;
        }
    }
    assert!(matches!(out, Flight::Landed { drop: Some(_) }), "{out:?}");
}

/// Every combat system's parameters are disjoint (Bevy's B0001).
#[test]
fn the_systems_parameters_do_not_conflict() {
    use bevy::ecs::system::IntoSystem;
    let mut world = World::new();
    fn init<M>(world: &mut World, s: impl IntoSystem<(), (), M>) {
        let mut sys = IntoSystem::into_system(s);
        sys.initialize(world);
    }
    init(&mut world, swing);
    init(&mut world, fly);
    init(&mut world, town_fight);
    init(&mut world, bites);
    init(&mut world, script);
}
