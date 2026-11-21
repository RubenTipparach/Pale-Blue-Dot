//! The critters' motion pinned (`ocean-environments` B29): the file and its
//! defaults, the bodies and the rig, the gnaw's beat, planted feet and the
//! forelimb's limits over a seeded run.

#![allow(clippy::needless_range_loop)]

use super::actor::{Actor, Ground, Kind, gnaw_d, strike};
use super::motion::CreatureMotion;
use super::rig::{self, Animal, Limits};
use super::*;

const MOTION_RON: &str = include_str!("../../../../assets/config/creature_motion.ron");

/// Flat ground with a pond: the ground 0.6 m down where x < 0, its water at
/// nought, and dry ground at nought where x >= 0.
struct Pond;

impl Ground for Pond {
    fn ground(&self, x: f32, _z: f32) -> f32 {
        if x < 0.0 { -0.6 } else { 0.0 }
    }

    fn level(&self, x: f32, _z: f32) -> Option<f32> {
        (x < 0.0).then_some(0.0)
    }
}

/// Dry and flat everywhere.
struct Meadow;

impl Ground for Meadow {
    fn ground(&self, _x: f32, _z: f32) -> f32 {
        0.0
    }

    fn level(&self, _x: f32, _z: f32) -> Option<f32> {
        None
    }
}

#[test]
fn the_shipped_motion_is_the_defaults_and_validates() {
    let shipped: CreatureMotion = ron::from_str(MOTION_RON).unwrap();
    shipped.validate().unwrap();
    assert_eq!(shipped, CreatureMotion::default());
}

/// The page prints the whole block; what it prints pastes in, and a block
/// with values off the page's sliders is refused.
#[test]
fn a_block_tuned_on_the_page_pastes_in_and_a_bad_one_is_refused() {
    let tuned = MOTION_RON
        .replace("beat_s: 0.42", "beat_s: 0.5")
        .replace("elbow_max_deg: 80", "elbow_max_deg: 70");
    let m: CreatureMotion = ron::from_str(&tuned).unwrap();
    m.validate().unwrap();
    assert_eq!(m.beaver.gnaw.beat_s, 0.5);
    assert_eq!(m.beaver.forelimb.elbow_max_deg, 70.0);
    for bad in [
        MOTION_RON.replace("elbow_max_deg: 80", "elbow_max_deg: 175"),
        MOTION_RON.replace("stance: 0.62", "stance: 1.5"),
        MOTION_RON.replace("beat_s: 0.42", "beat_s: 0.0"),
        MOTION_RON.replace("cracks: 4", "cracks: 0"),
        MOTION_RON.replace("exaggerate: 1.0", "exaggerate: -1.0"),
    ] {
        let m: CreatureMotion = ron::from_str(&bad).unwrap();
        assert!(m.validate().is_err(), "accepted:\n{bad}");
    }
    assert!(
        ron::from_str::<CreatureMotion>(&MOTION_RON.replace("hz: 7.0", "hz: 7.0, bogus: 1.0"))
            .is_err(),
        "an unknown key is refused"
    );
}

/// The bodies are the approved mockup's, skinned round the rig's own
/// joints: a joint that moved is a bug in one of the two copies.
#[test]
fn each_bodys_rest_joints_are_its_rigs() {
    for animal in Animal::ALL {
        let body = body_of(animal);
        let rest = animal.spec().rest();
        for k in 0..20 {
            assert!(
                body.rest[k].distance(rest[k]) < 1e-3,
                "{} {}: mesh {:?}, rig {:?}",
                animal.name(),
                rig::BONES[k],
                body.rest[k],
                rest[k]
            );
        }
        assert!(body.positions.len() > 2000, "{}", animal.name());
        assert_eq!(body.positions.len(), body.weights.len());
        for w in &body.weights {
            let sum: f32 = w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-3);
        }
        assert!(body.indices.len().is_multiple_of(3));
    }
}

/// The forward kinematics agree with the rig: at rest every bone is at its
/// rest point.
#[test]
fn the_rest_pose_puts_every_bone_at_rest() {
    for animal in Animal::ALL {
        let spec = animal.spec();
        let p = rig::Pose {
            y: spec.h,
            ..Default::default()
        };
        let fk = rig::fk(spec, &p);
        let rest = spec.rest();
        for k in 0..20 {
            let at: Vec3 = fk[k].translation.into();
            assert!(
                at.distance(rest[k]) < 1e-5,
                "{} {}",
                animal.name(),
                rig::BONES[k]
            );
        }
    }
}

/// A strike's hit lands where its four stages say: fastest at contact, the
/// contact value on the boundary between the hit and the recoil.
#[test]
fn a_strike_is_windup_anticipation_hit_and_recoil() {
    let d = [0.3, 0.2, 0.1, 0.4];
    let (v, s) = strike(0.15, d, 1.0, 1.2, -0.5, 0.1);
    assert_eq!(s, super::actor::Stage::Windup);
    assert!(v > 0.0 && v < 1.0);
    assert_eq!(
        strike(0.4, d, 1.0, 1.2, -0.5, 0.1).1,
        super::actor::Stage::Anticipation
    );
    assert_eq!(
        strike(0.55, d, 1.0, 1.2, -0.5, 0.1).1,
        super::actor::Stage::Hit
    );
    let (at_contact, s) = strike(0.6 + 1e-4, d, 1.0, 1.2, -0.5, 0.1);
    assert_eq!(s, super::actor::Stage::Recoil);
    assert!((at_contact + 0.5).abs() < 0.01, "{at_contact}");
    // the hit is fastest at its end (t cubed), never slowing into contact
    let a = strike(0.59, d, 1.0, 1.2, -0.5, 0.1).0;
    let b = strike(0.5999, d, 1.0, 1.2, -0.5, 0.1).0;
    let c = strike(0.51, d, 1.0, 1.2, -0.5, 0.1).0;
    let e = strike(0.5199, d, 1.0, 1.2, -0.5, 0.1).0;
    assert!((a - b).abs() > (c - e).abs());
    assert_eq!(
        strike(2.0, d, 1.0, 1.2, -0.5, 0.1).1,
        super::actor::Stage::Rest
    );
}

/// The gnaw's bites land on one steady beat: the k-th at k beats plus the
/// windup, anticipation and hit, within a frame (B28: one steady beat).
#[test]
fn the_gnaws_hit_is_on_its_beat() {
    let m = CreatureMotion::default();
    let mut a = Actor::new(Animal::Beaver, Vec3::new(1.0, 0.0, 0.0), 0.0, 3);
    a.start(Kind::GnawLow { bites: 6, last: -1 });
    let dt = 1.0 / 120.0;
    let d = gnaw_d(&m);
    let beat: f32 = d.iter().sum();
    let mut hits = Vec::new();
    let mut t = 0.0;
    let mut seen = 0;
    while t < beat * 6.0 - dt {
        a.update(&Meadow, &m, dt);
        t += dt;
        if a.hits > seen {
            seen = a.hits;
            hits.push(t);
        }
    }
    assert_eq!(hits.len(), 6, "{hits:?}");
    for (k, at) in hits.iter().enumerate() {
        let want = k as f32 * beat + d[0] + d[1] + d[2];
        assert!(
            (at - want).abs() <= dt + 1e-4,
            "bite {k} at {at}, its beat {want}"
        );
    }
    for w in hits.windows(2) {
        assert!(
            (w[1] - w[0] - beat).abs() <= dt + 1e-4,
            "an uneven beat {hits:?}"
        );
    }
}

/// The tail slap's hit lands on its contact: windup, anticipation and hit.
#[test]
fn the_tail_slap_lands_on_its_contact() {
    let m = CreatureMotion::default();
    let mut a = Actor::new(Animal::Beaver, Vec3::new(-2.0, 0.0, 0.0), 0.0, 5);
    a.anchor = a.pos;
    a.start(Kind::Slap);
    let dt = 1.0 / 120.0;
    let s = &m.beaver.slap;
    let want = s.windup_s + s.anticipate_s + s.strike_s;
    let mut t = 0.0;
    while a.hits == 0 && t < 3.0 {
        a.update(&Pond, &m, dt);
        t += dt;
    }
    assert!(
        (t - want).abs() <= dt + 1e-4,
        "slapped at {t}, contact {want}"
    );
}

/// No foot moves while it is down: in the beaver's walk and the otter's
/// bound, a foot in its stance on the ground stays where it was put, frame
/// after frame.
#[test]
fn no_foot_slides_in_stance() {
    let m = CreatureMotion::default();
    for animal in [Animal::Beaver, Animal::RiverOtter] {
        let mut a = Actor::new(animal, Vec3::ZERO, 0.0, 9);
        a.start(Kind::Move {
            goal: super::actor::Goal::Point(Vec3::new(40.0, 0.0, 0.0)),
            speed: 1.0,
            tol: 0.1,
        });
        let (stance, offs) = if animal == Animal::Beaver {
            (m.beaver.walk.stance, [-0.25, -0.75, 0.0, -0.5])
        } else {
            (m.river_otter.bound.stance, [-0.5, -0.56, 0.0, -0.06])
        };
        let dt = 1.0 / 120.0;
        let mut last: [Option<Vec3>; 4] = [None; 4];
        let mut worst = 0.0f32;
        let mut checked = 0;
        for frame in 0..1200 {
            a.update(&Meadow, &m, dt);
            // after the first step's blend, and with the speed settled
            if frame < 240 {
                continue;
            }
            let fk = rig::fk(animal.spec(), &a.out);
            let root = bevy::math::Affine3A::from_rotation_translation(
                Quat::from_rotation_y(a.yaw),
                a.pos,
            );
            for i in 0..4 {
                let u = rig::frac(a.phi + offs[i]);
                let foot: Vec3 = root.transform_point3a(fk[rig::foot(i)].translation).into();
                // well inside the stance, not at its lift-off or touch-down,
                // and down on the ground (the bound's leap lifts every foot)
                if u > 0.03 && u < stance - 0.03 && foot.y.abs() < 5e-4 {
                    if let Some(was) = last[i] {
                        worst = worst.max(foot.distance(was));
                        checked += 1;
                    }
                    last[i] = Some(foot);
                } else {
                    last[i] = None;
                }
            }
        }
        assert!(checked > 500, "{} checked {checked}", animal.name());
        assert!(
            worst < 1.5e-3,
            "{}: a foot in stance moved {worst} m in a frame",
            animal.name()
        );
    }
}

/// The forelimb's limits (version 5) are never passed: each animal left to
/// its own seeded choices for 600 s by a pond, startled now and then, its
/// elbow and wrist as drawn every frame within its limits.
#[test]
fn the_forelimbs_never_pass_their_limits_over_a_seeded_run() {
    let m = CreatureMotion::default();
    for animal in Animal::ALL {
        let lim = Limits::of(match animal {
            Animal::Beaver => &m.beaver.forelimb,
            Animal::RiverOtter => &m.river_otter.forelimb,
            Animal::SeaOtter => &m.sea_otter.forelimb,
        });
        let mut a = Actor::new(animal, Vec3::new(-2.0, 0.0, 0.0), 0.0, 7);
        let dt = 1.0 / 30.0;
        let mut steps = std::collections::BTreeSet::new();
        let (mut elbow, mut flex, mut ext) = (f32::MIN, f32::MIN, f32::MIN);
        for frame in 0..(600 * 30) {
            let t = frame as f32 * dt;
            // its creature swims a slow loop in the pond
            a.anchor = Vec3::new(-2.5 + 1.5 * (t * 0.05).cos(), 0.0, 1.5 * (t * 0.05).sin());
            a.spooked = (t % 97.0) < 3.0;
            a.update(&Pond, &m, dt);
            if let Some(s) = a.step {
                steps.insert(s.name());
            }
            for i in 0..2 {
                let (lk, lf) = (a.out.lk[i], a.out.lf[i]);
                assert!(lk.is_finite() && lf.is_finite());
                assert!(
                    lk <= lim.e_max + 1e-5 && lk >= lim.e_min - 1e-5,
                    "{} elbow {} in {:?}",
                    animal.name(),
                    lk.to_degrees(),
                    a.step.map(|s| s.name())
                );
                assert!(
                    lf <= lim.w_hi + 1e-5 && lf >= lim.w_lo - 1e-5,
                    "{} wrist {} in {:?}",
                    animal.name(),
                    lf.to_degrees(),
                    a.step.map(|s| s.name())
                );
                elbow = elbow.max(lk);
                flex = flex.max(-(lf + std::f32::consts::FRAC_PI_2));
                ext = ext.max(lf + std::f32::consts::FRAC_PI_2);
            }
            assert!(a.pos.is_finite(), "{} left the world", animal.name());
        }
        // it did more than swim
        assert!(steps.len() >= 4, "{}: only {steps:?}", animal.name());
        println!(
            "{}: elbow {:.0}, wrist flexion {:.0}, extension {:.0} degrees; {steps:?}",
            animal.name(),
            elbow.to_degrees(),
            flex.to_degrees(),
            ext.to_degrees()
        );
    }
}

/// A beaver goes ashore and back: out of the pond onto the bank, and into
/// the water again by its creature.
#[test]
fn a_beaver_climbs_out_onto_the_bank_and_back() {
    let m = CreatureMotion::default();
    let mut on_bank = false;
    for seed in 0..20u32 {
        let mut a = Actor::new(Animal::Beaver, Vec3::new(-1.5, 0.0, 0.0), 0.0, seed);
        a.anchor = a.pos;
        for _ in 0..(120 * 30) {
            a.update(&Pond, &m, 1.0 / 30.0);
            if a.pos.x > 0.2 {
                on_bank = true;
            }
        }
        if on_bank {
            assert!(a.pos.is_finite());
            break;
        }
    }
    assert!(on_bank, "no beaver went ashore in 20 seeds");
}

/// Ground of the game's frame: a tangent frame turns a point to the planet
/// and back to itself, its height the radius's.
#[test]
fn a_tangent_frame_turns_points_both_ways() {
    let base = DVec3::new(0.3, 0.9, 0.2).normalize() * 4800.0;
    let f = Tangent::at(base);
    for p in [
        Vec3::ZERO,
        Vec3::new(3.0, 0.5, -2.0),
        Vec3::new(-11.0, -1.2, 7.5),
    ] {
        let back = f.from_planet(f.to_planet(p));
        assert!(back.distance(p) < 0.01, "{p} came back {back}");
    }
    assert!((f.to_planet(Vec3::new(0.0, 2.0, 0.0)).length() - 4802.0).abs() < 1e-6);
}

/// The skinned bodies' drawn count is capped, nearest first.
#[test]
fn at_most_eight_are_drawn() {
    assert_eq!(MAX_DRAWN, 8);
    assert!(DRAW_M <= crate::config::FaunaConfig::default().0.spawn.despawn_m);
}
