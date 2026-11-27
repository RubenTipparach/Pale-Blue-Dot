//! `wildfire` 4.1, 4.3 and 4.4 in the game: a torch on dry grass in a
//! Survival world, the server's tick through the save, torches in the rain,
//! and the danger to the living.

use super::*;
use crate::planet::lod;
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::edits::Edit;
use pbd_core::fire::blaze::Lighting;
use pbd_core::survival::WorldMode;

/// A place on fields with grass: the test ground's anchor.
fn anchor() -> Vec3 {
    Vec3::new(0.31, 0.8, 0.51).normalize()
}

fn survival_save() -> WorldSave {
    let mut save = WorldSave::memory_only();
    save.identity.mode = Some(WorldMode::Survival);
    save
}

/// The weather after two dry weeks of late summer everywhere: warm, dry
/// air, dry ground, no rain, no wind.
fn dry_summer() -> Atmosphere {
    let mut a = Atmosphere::new(
        terrain_config(),
        AtmosphereSettings {
            level: 3,
            ..Default::default()
        },
        7,
    );
    for i in 0..a.grid.len() {
        let e = a.surface.elevation[i];
        a.air_k[i] = 30.0 + a.settings.lapse_k_per_m * e;
        a.vapour[i] = 0.0;
        a.ground_wet[i] = 0.0;
        a.rain_rate[i] = 0.0;
        a.rain_seen[i] = 0.0;
        a.wind[i] = Vec3::ZERO;
    }
    a
}

fn spring() -> Atmosphere {
    let mut a = dry_summer();
    for i in 0..a.grid.len() {
        let e = a.surface.elevation[i];
        a.air_k[i] = 16.0 + a.settings.lapse_k_per_m * e;
        a.vapour[i] = 0.6 * a.saturation(16.0);
        a.ground_wet[i] = 0.5;
    }
    a
}

/// The year fraction of late summer, and of spring, where the anchor is.
fn late_summer() -> f64 {
    if anchor().y >= 0.0 { 0.10 } else { 0.60 }
}
fn in_spring() -> f64 {
    if anchor().y >= 0.0 { 0.75 } else { 0.25 }
}

/// An open grass cell on the tier, near the anchor, dry in late summer.
fn grass_cell(set: &lod::FineSet, cfg: &FireConfig, a: &Atmosphere) -> u32 {
    let marked = BTreeMap::new();
    let ground = TierGround {
        set,
        odds: odds_of(None),
        air: air_at(a, anchor()),
        marked: &marked,
    };
    let records = pbd_core::records::Records::new();
    let yields = pbd_core::records::YieldSet::new();
    let world = blaze::World {
        cfg,
        seed: 1,
        ground: &ground,
        records: &records,
        yields: &yields,
        now_s: 0.0,
        year_fraction: late_summer(),
    };
    let mut best: Option<(f32, u32)> = None;
    for r in set.finest_records() {
        let key = r.key();
        if ground.land(key).is_some_and(|l| l.plant == Plant::Grass) {
            let d = world.plant_dry(key);
            let off = Vec3::from_slice(&r.direction_height[..3])
                .normalize()
                .angle_between(anchor());
            if d >= 0.6 && best.is_none_or(|(o, _)| off < o) {
                best = Some((off, key));
            }
        }
    }
    best.expect("dry grass on the tier").1
}

#[test]
fn a_torch_on_late_summer_grass_starts_a_blaze_and_the_server_spreads_it() {
    let (fine, _) = crate::planet::test_ground(anchor());
    let cfg = FireConfig::shipped();
    let air = dry_summer();
    let mut save = survival_save();
    let cell = grass_cell(&fine.set, &cfg, &air);
    let marked = BTreeMap::new();
    let lit = light(
        &cfg,
        &mut save,
        &fine.set,
        &air,
        &marked,
        &odds_of(None),
        cell,
        100.0,
        late_summer(),
    );
    assert!(matches!(lit, Lighting::Blaze(_)), "{lit:?}");
    assert_eq!(blaze::live(&save.records).len(), 1);
    assert!(
        save.records.get(blaze::IGNITION_KIND, 1).is_some(),
        "the player's act"
    );
    // The server's ticks: it burns, and what burnt is in the save.
    for s in 0..400 {
        run_once(
            &cfg,
            &mut save,
            &fine.set,
            &air,
            &marked,
            &odds_of(None),
            101.0 + f64::from(s),
            late_summer(),
        );
    }
    let burnt = save.records.of_kind(blaze::BURNT_KIND).count();
    assert!(burnt >= 1, "the lit cell burnt");
    assert!(burnt <= cfg.limits.cells as usize);
    // The tier shows it: ash on the burnt cells' words.
    let (_, want, _, _) = shown(&cfg, &save, &fine.set, 500.0);
    let mut set = (*fine.set).clone();
    let mut scars = BTreeMap::new();
    assert!(apply_scars(&mut set, &want, &mut scars) >= 1);
    let i = set.finest_index_of(cell).unwrap();
    assert_eq!(
        set.finest_records()[i].metadata[1] & ASH,
        ASH,
        "ash on the lit cell"
    );
    assert!(!scars_differ(&set, &want, &scars));
    // Sixty days on the scar is gone from the words.
    let day = f64::from(pbd_core::daylight::DAY_S);
    let (_, later, _, snags) = shown(&cfg, &save, &set, 500.0 + 61.0 * day);
    assert!(later.is_empty() && snags.is_empty());
    apply_scars(&mut set, &later, &mut scars);
    assert_eq!(
        set.finest_records()[i].metadata[1],
        fine.set.finest_records()[i].metadata[1],
        "the word as the generator made it"
    );
}

#[test]
fn a_torch_on_spring_grass_makes_a_damp_flame_and_explorer_none() {
    let (fine, _) = crate::planet::test_ground(anchor());
    let cfg = FireConfig::shipped();
    let cell = grass_cell(&fine.set, &cfg, &dry_summer());
    let marked = BTreeMap::new();
    let mut save = survival_save();
    let lit = light(
        &cfg,
        &mut save,
        &fine.set,
        &spring(),
        &marked,
        &odds_of(None),
        cell,
        100.0,
        in_spring(),
    );
    assert_eq!(lit, Lighting::Damp);
    assert!(
        save.records.of_kind(blaze::BLAZE_KIND).next().is_none(),
        "nothing saved"
    );
    let mut explorer = WorldSave::memory_only();
    let lit = light(
        &cfg,
        &mut explorer,
        &fine.set,
        &dry_summer(),
        &marked,
        &odds_of(None),
        cell,
        100.0,
        late_summer(),
    );
    assert_eq!(lit, Lighting::Nothing, "an Explorer world makes no fire");
    assert!(explorer.records.of_kind(blaze::BLAZE_KIND).next().is_none());
}

/// A placed torch in forced rain under the open sky goes out after 20 s,
/// a roof keeps one lit, the out torch stays out across a reload of the
/// records, the player relights it, and the field goes dark and lit again.
#[test]
fn a_placed_torch_goes_out_in_forced_rain_and_a_roof_keeps_one_lit() {
    let cfg = FireConfig::shipped();
    let (fine, _) = crate::planet::test_ground(anchor());
    let cell = fine.set.finest_records()[fine
        .set
        .finest_index_of(grass_cell(&fine.set, &cfg, &dry_summer()))
        .unwrap()]
    .key();
    let record = fine.set.finest_index_of(cell).unwrap();
    let column = fine.set.columns.column(record).unwrap();
    let top = column.surface().unwrap() as u16;
    // A second torch beside it, under a roof four layers up.
    let roofed = pbd_core::roads::neighbours(cell)[0];
    let roofed_top = fine
        .set
        .columns
        .column(fine.set.finest_index_of(roofed).unwrap())
        .unwrap()
        .surface()
        .unwrap() as u16;
    let mut edits = pbd_core::edits::Edits::new();
    for e in [
        Edit {
            cell,
            layer: top + 1,
            material: Material::Torch,
        },
        Edit {
            cell: roofed,
            layer: roofed_top + 1,
            material: Material::Torch,
        },
        Edit {
            cell: roofed,
            layer: roofed_top + 5,
            material: Material::Stone,
        },
    ] {
        edits.set(e);
    }
    let set = lod::generate_fine_live(
        anchor(),
        lod::BAND_M,
        lod::REGEN_DISTANCE_M,
        &crate::config::ColumnSettings::default(),
        &edits,
        4,
        None,
        0.0,
    );
    let mut save = survival_save();
    for e in edits.all() {
        assert!(save.accept(e, None, &Default::default()));
    }
    let mut rain = dry_summer();
    rain.rain_seen.iter_mut().for_each(|r| *r = 1.0);
    let mut soaking = Soaking::default();
    for _ in 0..19 {
        assert_eq!(
            soak_once(&cfg, &mut save, &set, &rain, &mut soaking, 1.0),
            0
        );
    }
    assert_eq!(
        soak_once(&cfg, &mut save, &set, &rain, &mut soaking, 1.0),
        1,
        "out at 20 s"
    );
    assert!(torch::is_out(&save.records, cell, top + 1));
    assert!(
        !torch::is_out(&save.records, roofed, roofed_top + 1),
        "the roof keeps it lit"
    );
    for _ in 0..60 {
        soak_once(&cfg, &mut save, &set, &rain, &mut soaking, 1.0);
    }
    assert!(!torch::is_out(&save.records, roofed, roofed_top + 1));
    // The field: the out torch gives no light.
    let (tier_fire, _, _, _) = shown(&cfg, &save, &set, 0.0);
    assert_eq!(tier_fire.out, vec![(cell, top + 1)]);
    let mut dark = set.clone();
    assert!(dark.columns.set_fire(tier_fire));
    assert!(!dark.columns.fire().out.is_empty());
    // Across a reload: the records as lines and back.
    let mut again = pbd_core::records::Records::new();
    for r in save.records.iter() {
        let line = r.line(&torch::author(cell));
        again.put(pbd_core::records::Record::parse_line(&line).unwrap().1);
    }
    assert!(
        torch::is_out(&again, cell, top + 1),
        "a reload keeps it out"
    );
    // Relit by the player.
    assert!(relight(&mut save, cell, top + 1));
    assert!(!torch::is_out(&save.records, cell, top + 1));
    // An Explorer world's torches never go out.
    let mut explorer = WorldSave::memory_only();
    for e in edits.all() {
        assert!(explorer.accept(e, None, &Default::default()));
    }
    // (The system returns before soaking in Explorer; its rule is `mode`.)
    assert!(!explorer.mode().is_survival());
}

/// Standing in fire hurts four a second, beside it one, and animals run.
#[test]
fn standing_in_fire_hurts_and_animals_flee() {
    let cfg = FireConfig::shipped();
    let at = anchor().as_dvec3() * f64::from(PLANET_RADIUS);
    let now = FireNow {
        flames: vec![Flame {
            key: 1,
            at,
            plant: Plant::Grass,
            crown: false,
            left: 1.0,
        }],
        ..Default::default()
    };
    let up = anchor().as_dvec3();
    let east = DVec3::Y.cross(up).normalize();
    assert_eq!(
        harm_at(&cfg, &now, at + up * 0.5),
        4.0,
        "in the burning cell"
    );
    assert_eq!(harm_at(&cfg, &now, at + east * 3.0), 1.0, "beside it");
    assert_eq!(harm_at(&cfg, &now, at + east * 12.0), 0.0, "well away");
    let animal = at + east * 20.0;
    let away = flee_from(&cfg, &now, animal).expect("an animal 20 m off runs");
    assert!(away.dot(east) > 0.9, "away from the fire");
    assert!(
        flee_from(&cfg, &now, at + east * 60.0).is_none(),
        "60 m off it does not"
    );
    assert!(flee_from(&cfg, &FireNow::default(), animal).is_none());
}
