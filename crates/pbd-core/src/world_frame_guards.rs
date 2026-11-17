//! The guard tests of `world-frames` (design 6): the simulation run far
//! from a body's centre, at the home planet's radius and at a thousand times
//! it, on a body far out in the system, steps as exactly as it does at the
//! origin. A planet-frame number that leaks into local math fails here, as a
//! number, before it is a feel.

use glam::{DQuat, DVec3, Vec3};

use crate::fauna::{FaunaSettings, FlockSettings, HOME_BODY, School, Water};
use crate::fishing::{Angler, Controls, Line, Phase};
use crate::orbit::CircularOrbit;
use crate::world_frame::{
    FrameKind, FrameMotion, FrameState, FrameTree, ORIGIN_CELL_M, SYSTEM, SimOrigin, WorldPos,
};

const AU: f64 = 1.495_978_707e11;

/// A flat sea: the same surface and bed everywhere, radii from the centre.
struct Flat {
    surface: f32,
    bed: f32,
}

impl Water for Flat {
    fn surface(&self, _: Vec3) -> f32 {
        self.surface
    }
    fn bed(&self, _: Vec3) -> f32 {
        self.bed
    }
}

/// A body at `offset` from the system's origin, and the origin near
/// `world` in its frame (`near`), or at its centre (the old planet-centred
/// simulation).
fn body(offset: f64) -> (FrameTree, crate::world_frame::FrameId) {
    let mut tree = FrameTree::new();
    let motion = if offset == 0.0 {
        FrameMotion::Rails(CircularOrbit::stationary())
    } else {
        FrameMotion::Rails(CircularOrbit::new(offset, 3.156e7, 0.7, DQuat::IDENTITY).unwrap())
    };
    let id = tree.add(SYSTEM, FrameKind::BodyInertial, motion);
    (tree, id)
}

fn origin(frame: crate::world_frame::FrameId, world: DVec3, near: bool) -> SimOrigin {
    let at = if near {
        (world / ORIGIN_CELL_M).round() * ORIGIN_CELL_M
    } else {
        DVec3::ZERO
    };
    SimOrigin::new(frame, at)
}

/// The worst move of a fish along the water, per 30 Hz step, against its
/// velocity times the step: the part of the move the flock asked for.
fn school_error(radius: f64, offset: f64, near: bool) -> f64 {
    let (tree, frame) = body(offset);
    let up = DVec3::new(0.3, 0.9, -0.31).normalize();
    let anchor_world = up * (radius - 3.0);
    let o = origin(frame, anchor_world, near);
    let anchor = o.to_local(WorldPos::new(frame, anchor_world));
    let water = Flat {
        surface: radius as f32,
        bed: (radius - 8.0) as f32,
    };
    let fauna = FaunaSettings::default();
    let roster = fauna.roster(HOME_BODY);
    let i = roster.iter().position(|s| s.id == "minnow").unwrap();
    let flock = FlockSettings::default();
    let mut school = School::spawn(&o, i as u16, &roster[i], anchor, 7, &water);
    let dt = 1.0 / 30.0;
    let mut worst = 0.0f64;
    for _ in 0..300 {
        let before: Vec<_> = school.positions.clone();
        school.step(&o, dt, &roster[i], &flock, &water, None);
        for (p, (was, v)) in school
            .positions
            .iter()
            .zip(before.iter().zip(&school.velocities))
        {
            let (now, then) = (o.to_world(*p).pos, o.to_world(*was).pos);
            let e = (now - then) - (*v * dt).as_dvec3();
            let up = now.normalize();
            worst = worst.max((e - up * e.dot(up)).length());
        }
    }
    // The body's place in the system never entered the step: the same
    // motion seen from the system is the same motion.
    let s = tree.convert(
        FrameState {
            frame,
            pos: o.to_world(school.positions[0]).pos,
            vel: school.velocities[0].as_dvec3(),
        },
        SYSTEM,
        0.0,
    );
    assert!(s.pos.is_finite());
    worst
}

#[test]
fn a_school_steps_exactly_in_local_space_wherever_its_body_is() {
    for radius in [4_800.0, 4_800_000.0] {
        for offset in [0.0, AU] {
            let local = school_error(radius, offset, true);
            eprintln!("GUARD school r {radius} offset {offset}: local {local:.3e} m/step");
            assert!(local < 1e-5, "local {local} at r {radius} offset {offset}");
        }
    }
    // The planet-centred simulation before `world-frames`, measured: at the
    // home planet's radius a fish's step is off by tenths of a millimetre.
    let planet = school_error(4_800.0, 0.0, false);
    eprintln!("GUARD school r 4800 planet-centred: {planet:.3e} m/step");
    assert!(planet > 1e-5, "planet-centred {planet}");
}

/// The float's bob on a flat sea, per step: its height over the centre
/// changes by exactly the change in its dip, and it does not drift along
/// the water.
fn float_error(radius: f64, near: bool) -> (f64, f64) {
    let (_, frame) = body(0.0);
    let up = DVec3::new(-0.2, 0.95, 0.24).normalize();
    let feet_world = up * (radius + 0.4);
    let o = origin(frame, feet_world, near);
    let feet = o.to_local(WorldPos::new(frame, feet_world));
    let up32 = o.up_at(feet);
    let east = Vec3::Y.cross(up32).normalize();
    let angler = Angler {
        tip: feet + up32 * 1.9,
        look: east,
        feet,
    };
    let water = Flat {
        surface: radius as f32,
        bed: (radius - 8.0) as f32,
    };
    let fauna = FaunaSettings::default();
    let roster = fauna.roster(HOME_BODY).to_vec();
    let mut line = Line::new(3);
    line.phase = Phase::Floating;
    line.dip = 0.06;
    line.float = o.with_radius(feet + east * 9.0, radius - 0.06);
    let mut schools: Vec<School> = Vec::new();
    let dt = 1.0 / 60.0;
    let (mut worst_bob, mut worst_drift) = (0.0f64, 0.0f64);
    for _ in 0..120 {
        let (was, was_dip) = (o.to_world(line.float).pos, line.dip);
        line.step(
            &o,
            dt,
            Controls::default(),
            &angler,
            &mut schools,
            &roster,
            &fauna.fishing,
            &fauna.flock,
            &water,
            25.0,
            1.0,
        );
        let now = o.to_world(line.float).pos;
        let bob = (now.length() - was.length()) + f64::from(line.dip - was_dip);
        worst_bob = worst_bob.max(bob.abs());
        let along = (now - was) - now.normalize() * (now - was).dot(now.normalize());
        worst_drift = worst_drift.max(along.length());
    }
    (worst_bob, worst_drift)
}

#[test]
fn the_float_bobs_exactly_in_local_space() {
    for radius in [4_800.0, 4_800_000.0] {
        let (bob, drift) = float_error(radius, true);
        eprintln!("GUARD float r {radius}: bob {bob:.3e} drift {drift:.3e} m/step");
        assert!(bob < 1e-5 && drift < 1e-5, "{bob} {drift} at {radius}");
    }
    let (bob, drift) = float_error(4_800.0, false);
    eprintln!("GUARD float r 4800 planet-centred: bob {bob:.3e} drift {drift:.3e} m/step");
    assert!(bob.max(drift) > 1e-5, "planet-centred {bob} {drift}");
}

/// A rebase of the school and the line together: every planet-frame point
/// is where it was, to a micrometre.
#[test]
fn a_rebase_moves_the_school_and_the_line_with_the_origin() {
    let (_, frame) = body(0.0);
    let up = DVec3::new(0.3, 0.9, -0.31).normalize();
    let anchor_world = up * 4_797.0;
    let o = origin(frame, anchor_world, true);
    let water = Flat {
        surface: 4_800.0,
        bed: 4_792.0,
    };
    let fauna = FaunaSettings::default();
    let roster = fauna.roster(HOME_BODY);
    let mut school = School::spawn(
        &o,
        0,
        &roster[0],
        o.to_local(WorldPos::new(frame, anchor_world)),
        11,
        &water,
    );
    let mut line = Line::new(1);
    line.float = o.with_radius(school.anchor + Vec3::X * 4.0, 4_800.0);
    let before: Vec<_> = school
        .positions
        .iter()
        .map(|p| o.to_world(*p).pos)
        .collect();
    let float = o.to_world(line.float).pos;
    let r = o
        .rebase_for(Vec3::new(100.0, 0.0, 0.0), ORIGIN_CELL_M)
        .unwrap();
    school.rebase(r.shift);
    line.rebase(r.shift);
    let n = o.rebased(r);
    for (p, w) in school.positions.iter().zip(&before) {
        assert!((n.to_world(*p).pos - *w).length() < 1e-5);
    }
    assert!((n.to_world(line.float).pos - float).length() < 1e-5);
}
