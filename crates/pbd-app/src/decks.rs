//! Pieces that move, and the walker riding them (`sail-the-cog` design 6).
//! A deck's `Frame` is set each tick from where the deck is: its solids and
//! surfaces answer in that frame, so they move rigidly with it and nothing
//! is cut again. A walker the deck holds is carried by the deck's own
//! [`Motion`] over the tick, `p ← now · then⁻¹ · p`, before it walks: its
//! place on the deck is kept, so a turn neither slides nor jitters it.
//!
//! Two kinds of deck move. A moored ship's ([`Decks`], step 2) is a town
//! piece in the walker's [`Structures`] swung about where it rests. A ship
//! under sail's ([`CraftDecks`], step 3) is held by the craft and stands
//! where its physics puts it.

use crate::towns::Towns;
use crate::vehicles::Vehicle;
use crate::walking::Structures;
use bevy::prelude::*;
use pbd_core::settlement::pieces::{BuildingSolids, Frame, Meshes};
use pbd_core::vehicle::Kind;

/// How a deck moved over the last tick: where its frame was, and is.
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub then: Frame,
    pub now: Frame,
}

impl Motion {
    /// Where a planet-local point on the deck went over the last tick.
    pub fn carry(&self, p: Vec3) -> Vec3 {
        self.now.world(self.then.local(p))
    }

    /// Where a planet-local direction on the deck turned over the last tick.
    pub fn carry_dir(&self, v: Vec3) -> Vec3 {
        let l = Vec3::new(v.dot(self.then.x), v.dot(self.then.y), v.dot(self.then.z));
        self.now.x * l.x + self.now.y * l.y + self.now.z * l.z
    }

    /// The deck's own velocity at a planet-local point on it, over the last
    /// tick of `dt` seconds.
    pub fn velocity_at(&self, p: Vec3, dt: f32) -> Vec3 {
        if dt > 0.0 {
            (self.carry(p) - p) / dt
        } else {
            Vec3::ZERO
        }
    }
}

/// A smooth step from 0 at `x` 0 to 1 at `x` 1, flat at both ends.
pub fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// How a deck moves about where it rests: a heave, a roll and a swing, each
/// a sine, and a steady turn. Amplitudes in metres and radians, periods in
/// seconds, the turn in radians a second.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    pub heave_m: f32,
    pub heave_s: f32,
    pub roll: f32,
    pub roll_s: f32,
    pub yaw: f32,
    pub yaw_s: f32,
    pub spin: f32,
}

impl Swing {
    /// A ship at its mooring: 0.15 m of heave on 6 s, 2° of roll on 7 s and
    /// 4° of swing on 23 s.
    pub const MOORED: Swing = Swing {
        heave_m: 0.15,
        heave_s: 6.0,
        roll: 2.0 * std::f32::consts::PI / 180.0,
        roll_s: 7.0,
        yaw: 4.0 * std::f32::consts::PI / 180.0,
        yaw_s: 23.0,
        spin: 0.0,
    };

    /// Still: a deck that stays where it rests.
    pub const STILL: Swing = Swing {
        heave_m: 0.0,
        heave_s: 1.0,
        roll: 0.0,
        roll_s: 1.0,
        yaw: 0.0,
        yaw_s: 1.0,
        spin: 0.0,
    };

    /// Every motion `k` times as large.
    pub fn scaled(self, k: f32) -> Self {
        Self {
            heave_m: self.heave_m * k,
            roll: self.roll * k,
            yaw: self.yaw * k,
            spin: self.spin * k,
            ..self
        }
    }

    /// Where a deck resting in `rest` is `t` seconds in: heaved along the
    /// rest frame's up, rolled about `bow` and swung about the up, both
    /// through the rest frame's origin, the waterline's middle.
    pub fn at(&self, rest: &Frame, bow: Vec3, t: f32) -> Frame {
        let wave = |a: f32, period: f32| a * (std::f32::consts::TAU * t / period).sin();
        let turn = Quat::from_axis_angle(rest.y, wave(self.yaw, self.yaw_s) + self.spin * t)
            * Quat::from_axis_angle(bow, wave(self.roll, self.roll_s));
        Frame {
            origin: rest.origin + rest.y * wave(self.heave_m, self.heave_s),
            x: turn * rest.x,
            y: turn * rest.y,
            z: turn * rest.z,
        }
    }
}

/// A deck: the piece of the walker's [`Structures`] it is, where it rests,
/// how it moves, and where it was a tick ago and is now.
#[derive(Clone, Debug)]
pub struct Deck {
    /// Its town and its piece there (`Towns::index`), so the place is found
    /// again as towns come and go. None for a deck a test puts at a fixed
    /// place.
    pub site: Option<(u32, usize)>,
    pub structure: usize,
    pub rest: Frame,
    /// The bow, a unit tangent: what it rolls about.
    pub bow: Vec3,
    pub swing: Swing,
    pub then: Frame,
    pub now: Frame,
    /// Its drawing, which is moved with it.
    pub mesh: Option<Entity>,
}

impl Deck {
    pub fn new(
        site: Option<(u32, usize)>,
        structure: usize,
        rest: Frame,
        bow: Vec3,
        swing: Swing,
    ) -> Self {
        Self {
            site,
            structure,
            rest,
            bow,
            swing,
            then: rest,
            now: rest,
            mesh: None,
        }
    }

    /// How it moved over the last tick.
    pub fn motion(&self) -> Motion {
        Motion {
            then: self.then,
            now: self.now,
        }
    }

    /// Where a planet-local point on the deck went over the last tick.
    pub fn carry(&self, p: Vec3) -> Vec3 {
        self.motion().carry(p)
    }

    /// Its drawing's transform under the town's root: from where it rests
    /// to where it is.
    fn transform(&self) -> Transform {
        let m = |f: &Frame| Mat3::from_cols(f.x, f.y, f.z);
        let turn = Quat::from_mat3(&(m(&self.now) * m(&self.rest).transpose())).normalize();
        Transform {
            translation: self.now.origin - turn * self.rest.origin,
            rotation: turn,
            scale: Vec3::ONE,
        }
    }
}

/// Every deck standing, and the time they swing by.
#[derive(Resource, Default)]
pub struct Decks {
    pub list: Vec<Deck>,
    pub t: f32,
}

impl Decks {
    /// The deck a piece of the walker's [`Structures`] is, if it is one.
    pub fn at(&self, structure: usize) -> Option<&Deck> {
        self.list.iter().find(|d| d.structure == structure)
    }
}

/// How far the harbours' cogs swing at their moorings, against the mooring
/// swing ([`Swing::MOORED`]): `--cog-swing`.
#[derive(Resource, Clone, Copy)]
pub struct CogSwing(pub f32);

impl Default for CogSwing {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Move each deck to where its swing has it now, its piece and its drawing
/// with it. A deck whose town has gone is dropped.
pub fn move_decks(
    time: Res<Time>,
    mut decks: ResMut<Decks>,
    structures: Option<ResMut<Structures>>,
    towns: Option<Res<Towns>>,
    mut transforms: Query<&mut Transform>,
) {
    let dt = time.delta_secs();
    decks.t += dt;
    let t = decks.t;
    let mut structures = structures;
    decks.list.retain_mut(|d| {
        if let Some((site, n)) = d.site {
            match towns.as_ref().and_then(|towns| towns.index(site, n)) {
                Some(i) => d.structure = i,
                None => return false,
            }
        }
        d.then = d.now;
        d.now = d.swing.at(&d.rest, d.bow, t);
        if let Some(piece) = structures.as_mut().and_then(|s| s.0.get_mut(d.structure)) {
            piece.frame = d.now;
        }
        if let Some(mut transform) = d.mesh.and_then(|e| transforms.get_mut(e).ok()) {
            *transform = d.transform();
        }
        true
    });
}

/// A ship's deck under sail (`sail-the-cog` step 3): the craft's ship cut
/// once in its own frame ([`pbd_core::settlement::pieces::cog::sailing`]),
/// standing where the craft is.
#[derive(Clone, Debug)]
pub struct CraftDeck {
    pub craft: Entity,
    pub solids: BuildingSolids,
    pub motion: Motion,
}

/// Every craft's deck, by its craft: what the walker meets aboard a ship,
/// beside the towns' pieces.
#[derive(Resource, Default)]
pub struct CraftDecks {
    pub list: Vec<CraftDeck>,
    /// The ship as cut once, in the craft's frame.
    cut: Option<BuildingSolids>,
}

impl CraftDecks {
    /// The deck a craft holds, if it holds one.
    pub fn at(&self, craft: Entity) -> Option<&CraftDeck> {
        self.list.iter().find(|d| d.craft == craft)
    }
}

/// Where a craft stands, as a frame in the walker's: its reference frame's
/// origin and axes, the render frame's centre added (`PlanetRenderFrame`),
/// as the vehicles reach the walker.
pub fn craft_frame(craft: &pbd_core::vehicle::Craft, centre: bevy::math::DVec3) -> Frame {
    let o = craft.body.orientation;
    Frame {
        origin: (centre + craft.reference_position()).as_vec3(),
        x: (o * bevy::math::DVec3::X).as_vec3(),
        y: (o * bevy::math::DVec3::Y).as_vec3(),
        z: (o * bevy::math::DVec3::Z).as_vec3(),
    }
}

/// Stand each cog's deck where its craft is after the craft's step, the
/// last tick's frame kept as where it was; cut a deck for a cog that has
/// none, and drop one whose craft is gone.
pub(crate) fn move_craft_decks(
    mut decks: ResMut<CraftDecks>,
    vehicles: Query<(Entity, &Vehicle)>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    assets: Option<Res<crate::towns::TownAssets>>,
) {
    let centre = frame.map_or(bevy::math::DVec3::ZERO, |f| f.center);
    let decks = &mut *decks;
    decks.list.retain(|d| {
        vehicles
            .get(d.craft)
            .is_ok_and(|(_, v)| v.craft.kind == Kind::Cog)
    });
    for (entity, vehicle) in &vehicles {
        if vehicle.craft.kind != Kind::Cog {
            continue;
        }
        let now = craft_frame(&vehicle.craft, centre);
        if let Some(deck) = decks.list.iter_mut().find(|d| d.craft == entity) {
            deck.motion.then = deck.motion.now;
            deck.motion.now = now;
            deck.solids.frame = now;
            continue;
        }
        let cut = decks.cut.get_or_insert_with(|| {
            // Its gangway on the side the harbour moors it, as it is drawn.
            let gang_side = assets
                .as_ref()
                .and_then(|a| a.harbour.cog.as_ref())
                .map_or(-1, |c| c.gang_side);
            pbd_core::settlement::pieces::cog::sailing(&mut Meshes::new(), &|_| 2.0, gang_side)
        });
        let mut solids = cut.clone();
        solids.frame = now;
        decks.list.push(CraftDeck {
            craft: entity,
            solids,
            motion: Motion { then: now, now },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rest() -> Frame {
        let y = Vec3::new(0.2, 1.0, -0.1).normalize();
        let x = y.any_orthonormal_vector();
        Frame {
            origin: y * 1000.0,
            x,
            y,
            z: x.cross(y),
        }
    }

    #[test]
    fn a_still_swing_rests_and_the_mooring_one_stays_small() {
        let rest = rest();
        let at = Swing::STILL.at(&rest, rest.x, 3.7);
        assert!(at.origin.distance(rest.origin) < 1e-4);
        assert!(at.x.distance(rest.x) < 1e-6);
        for k in 0..230 {
            let t = k as f32 * 0.1;
            let f = Swing::MOORED.at(&rest, rest.x, t);
            assert!(f.origin.distance(rest.origin) <= 0.15 + 1e-3);
            assert!(f.y.angle_between(rest.y) <= 0.04);
        }
    }

    #[test]
    fn a_point_carried_keeps_its_place_on_the_deck() {
        let rest = rest();
        let mut deck = Deck::new(None, 0, rest, rest.x, Swing::MOORED.scaled(10.0));
        let local = Vec3::new(3.0, 1.9, -1.2);
        let mut p = rest.world(local);
        for k in 1..=500 {
            deck.then = deck.now;
            deck.now = deck.swing.at(&deck.rest, deck.bow, k as f32 / 60.0);
            p = deck.carry(p);
        }
        assert!(deck.now.local(p).distance(local) < 1e-3);
    }

    #[test]
    fn a_decks_drawing_goes_where_its_frame_does() {
        let rest = rest();
        let mut deck = Deck::new(None, 0, rest, rest.x, Swing::MOORED.scaled(5.0));
        deck.now = deck.swing.at(&deck.rest, deck.bow, 4.2);
        let local = Vec3::new(-6.0, 3.5, 2.0);
        let drawn = deck.transform().transform_point(rest.world(local));
        assert!(drawn.distance(deck.now.world(local)) < 1e-3);
    }
}
