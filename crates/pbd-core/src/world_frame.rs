//! World frames and the local simulation origin (`world-frames`).
//!
//! Three levels, with frame identity explicit at every one:
//!
//! - **The system frame** ([`SYSTEM`]): the root, inertial, where orbits are
//!   evaluated. f64. Nothing on a surface is ever stored here: at 1 AU an f64
//!   holds 30 µm, at 30 AU 1 mm.
//! - **Body and vessel frames** ([`FrameTree`]): a planet's centre (inertial,
//!   or turning with the ground), a moon, a station, a ship. Each has an f64
//!   pose in its parent over time. A position relative to a body's centre is
//!   precise to about 1e-9 m at planet scale, so everything a player made or
//!   walked to is stored as a [`WorldPos`] in its body's frame.
//! - **The local frame** ([`SimOrigin`]): an f64 origin in one frame, near the
//!   player, and plain f32 `Vec3`s relative to it. Avian, the simulation and
//!   the renderer work here, where an f32 within a few tens of metres holds
//!   micrometres. It moves (a *rebase*) by whole grid cells at a physics tick
//!   boundary, and changes frame (a *handoff*) when the player leaves a
//!   body or boards a ship.
//!
//! The only conversions between a local `Vec3` and a [`WorldPos`] are the
//! methods of [`SimOrigin`]. There is deliberately no `From`/`Into` between
//! them: a planet-frame number cannot be passed where a local one is meant
//! without naming the conversion.

use glam::{DQuat, DVec3, Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::frame::{KinematicState, LocalFrame};
use crate::orbit::CircularOrbit;

/// A frame in the [`FrameTree`]. `FrameId(0)` is the system root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FrameId(pub u16);

/// The system frame: the root of every tree.
pub const SYSTEM: FrameId = FrameId(0);

/// A position in a named frame, in f64 metres. Long-lived positions (saves,
/// sites, anything kept across a rebase) are stored as this, never as an f32
/// relative to a planet's centre.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "Stored", into = "Stored")]
pub struct WorldPos {
    pub frame: FrameId,
    pub pos: DVec3,
}

/// How a [`WorldPos`] is written: its frame and three f64s.
#[derive(Clone, Copy, Serialize, Deserialize)]
struct Stored(FrameId, [f64; 3]);

impl From<Stored> for WorldPos {
    fn from(s: Stored) -> Self {
        Self::new(s.0, DVec3::from_array(s.1))
    }
}

impl From<WorldPos> for Stored {
    fn from(w: WorldPos) -> Self {
        Self(w.frame, w.pos.to_array())
    }
}

impl WorldPos {
    pub const fn new(frame: FrameId, pos: DVec3) -> Self {
        Self { frame, pos }
    }
}

/// A position and a velocity, both relative to `frame` (the velocity is the
/// rate of change of `pos` in that frame's own axes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameState {
    pub frame: FrameId,
    pub pos: DVec3,
    pub vel: DVec3,
}

/// What kind of thing a frame is the frame of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    System,
    /// A body's centre, axes not turning (orbit, flight near the body).
    BodyInertial,
    /// A body's centre, axes turning with its ground (a surface).
    BodyFixed,
    /// A ship, a station, a vehicle: its pose is set by the game each tick.
    Vessel,
}

/// How a frame moves in its parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameMotion {
    Root,
    /// On rails: a circular orbit about the parent's origin, axes parallel.
    Rails(CircularOrbit),
    /// Same origin as the parent, axes turning about `axis` (the parent's
    /// axes, unit) at `rate` rad/s, `phase` rad at t = 0.
    Spin {
        axis: DVec3,
        rate: f64,
        phase: f64,
    },
    /// A pose the game sets (a ship): the frame as it was at `at` seconds,
    /// carried forward at its velocities.
    Vessel {
        pose: LocalFrame,
        at: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameNode {
    pub parent: Option<FrameId>,
    pub kind: FrameKind,
    pub motion: FrameMotion,
}

/// The tree of frames: the system, its bodies, their surfaces and vessels.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameTree {
    nodes: Vec<FrameNode>,
}

impl Default for FrameTree {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameTree {
    /// A tree holding only the system frame.
    pub fn new() -> Self {
        Self {
            nodes: vec![FrameNode {
                parent: None,
                kind: FrameKind::System,
                motion: FrameMotion::Root,
            }],
        }
    }

    pub fn add(&mut self, parent: FrameId, kind: FrameKind, motion: FrameMotion) -> FrameId {
        assert!(
            (parent.0 as usize) < self.nodes.len(),
            "unknown parent frame"
        );
        assert!(
            !matches!(motion, FrameMotion::Root),
            "only the system is a root"
        );
        let id = FrameId(u16::try_from(self.nodes.len()).expect("frame tree full"));
        self.nodes.push(FrameNode {
            parent: Some(parent),
            kind,
            motion,
        });
        id
    }

    pub fn node(&self, id: FrameId) -> &FrameNode {
        &self.nodes[id.0 as usize]
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    /// Set a vessel's pose in its parent (the game's integration of a ship).
    pub fn set_vessel(&mut self, id: FrameId, pose: LocalFrame, at: f64) {
        let node = &mut self.nodes[id.0 as usize];
        assert_eq!(node.kind, FrameKind::Vessel, "only a vessel's pose is set");
        node.motion = FrameMotion::Vessel { pose, at };
    }

    /// Move a vessel to another parent, keeping where it is and how it moves
    /// (`at` seconds): docking, leaving a body's sphere of influence.
    pub fn reparent_vessel(&mut self, id: FrameId, parent: FrameId, t: f64) {
        let here = self.in_frame(id, parent, t);
        let node = &mut self.nodes[id.0 as usize];
        assert_eq!(node.kind, FrameKind::Vessel, "only a vessel changes parent");
        node.parent = Some(parent);
        node.motion = FrameMotion::Vessel { pose: here, at: t };
    }

    /// The frame `id` as seen from its parent at `t`: where its origin is,
    /// how fast it moves, how its axes are turned and turning, all in the
    /// parent's axes.
    pub fn in_parent(&self, id: FrameId, t: f64) -> LocalFrame {
        match self.node(id).motion {
            FrameMotion::Root => LocalFrame::default(),
            FrameMotion::Rails(orbit) => {
                let s = orbit.sample(t);
                LocalFrame::moving(s.position, s.velocity)
            }
            FrameMotion::Spin { axis, rate, phase } => LocalFrame {
                origin: DVec3::ZERO,
                velocity: DVec3::ZERO,
                orientation: DQuat::from_axis_angle(axis, phase + rate * t),
                angular_velocity: axis * rate,
            },
            FrameMotion::Vessel { pose, at } => pose.advanced(t - at),
        }
    }

    fn depth(&self, mut id: FrameId) -> usize {
        let mut d = 0;
        while let Some(p) = self.node(id).parent {
            id = p;
            d += 1;
        }
        d
    }

    /// The lowest frame both `a` and `b` hang from.
    pub fn common_ancestor(&self, mut a: FrameId, mut b: FrameId) -> FrameId {
        let (mut da, mut db) = (self.depth(a), self.depth(b));
        while da > db {
            a = self.node(a).parent.unwrap();
            da -= 1;
        }
        while db > da {
            b = self.node(b).parent.unwrap();
            db -= 1;
        }
        while a != b {
            a = self.node(a).parent.unwrap();
            b = self.node(b).parent.unwrap();
        }
        a
    }

    /// Compose `inner` (a frame in `outer`'s child) out through `outer`.
    fn compose(outer: LocalFrame, inner: LocalFrame) -> LocalFrame {
        let origin = outer.to_global(KinematicState {
            position: inner.origin,
            velocity: inner.velocity,
        });
        LocalFrame {
            origin: origin.position,
            velocity: origin.velocity,
            orientation: (outer.orientation * inner.orientation).normalize(),
            angular_velocity: outer.angular_velocity + outer.orientation * inner.angular_velocity,
        }
    }

    /// The frame `id` as seen from its ancestor `ancestor` at `t`.
    fn in_ancestor(&self, id: FrameId, ancestor: FrameId, t: f64) -> LocalFrame {
        let mut pose = LocalFrame::default();
        let mut at = id;
        while at != ancestor {
            pose = Self::compose(self.in_parent(at, t), pose);
            at = self
                .node(at)
                .parent
                .expect("ancestor is not above this frame");
        }
        pose
    }

    /// The frame `id` as seen from `from` at `t` (any two frames). Composed
    /// only up to their common ancestor, so two frames near each other far
    /// out in the system convert without passing through system-sized
    /// numbers.
    pub fn in_frame(&self, id: FrameId, from: FrameId, t: f64) -> LocalFrame {
        let common = self.common_ancestor(id, from);
        let id_in_common = self.in_ancestor(id, common, t);
        let from_in_common = self.in_ancestor(from, common, t);
        // id in `from` = from⁻¹ ∘ id.
        let o = from_in_common.to_local(KinematicState {
            position: id_in_common.origin,
            velocity: id_in_common.velocity,
        });
        let back = from_in_common.orientation.inverse();
        LocalFrame {
            origin: o.position,
            velocity: o.velocity,
            orientation: (back * id_in_common.orientation).normalize(),
            angular_velocity: back
                * (id_in_common.angular_velocity - from_in_common.angular_velocity),
        }
    }

    /// A position and velocity in one frame, as seen from another, with the
    /// turning terms (`ω × r`) of every frame between them.
    pub fn convert(&self, state: FrameState, to: FrameId, t: f64) -> FrameState {
        if state.frame == to {
            return state;
        }
        let common = self.common_ancestor(state.frame, to);
        let up = self
            .in_ancestor(state.frame, common, t)
            .to_global(KinematicState {
                position: state.pos,
                velocity: state.vel,
            });
        let down = self.in_ancestor(to, common, t).to_local(up);
        FrameState {
            frame: to,
            pos: down.position,
            vel: down.velocity,
        }
    }

    pub fn convert_pos(&self, p: WorldPos, to: FrameId, t: f64) -> WorldPos {
        let s = self.convert(
            FrameState {
                frame: p.frame,
                pos: p.pos,
                vel: DVec3::ZERO,
            },
            to,
            t,
        );
        WorldPos::new(to, s.pos)
    }
}

/// The grid a rebase moves the local origin by on foot (the owner,
/// 2026-11-17: "16 m is kinda small, 128 m probably good"). A power of two,
/// so every rebase moves every local coordinate by an f32-exact amount and
/// repeated rebases never drift. With the slack below a local coordinate
/// stays within 96 m of the origin on each axis, where an f32 holds 7.6 µm.
pub const ORIGIN_CELL_M: f64 = 128.0;

/// The local origin is rebased when the anchor (the player) is more than
/// this many cells from it, so a walker on a cell edge does not rebase back
/// and forth.
pub const REBASE_SLACK_CELLS: f64 = 0.75;

/// No local position of anything simulated or drawn near the player is ever
/// this far from the local origin: past it a planet-frame number has leaked
/// into local math (`ORIGIN_LEAK`).
pub const LOCAL_BOUND_M: f32 = 2_000.0;

/// The grid cell for an anchor moving at `speed` m/s with physics steps of
/// `dt` s: [`ORIGIN_CELL_M`] on foot, doubled until a step covers under a quarter cell,
/// so at ship speeds the origin rebases every few ticks, not every tick, and
/// what it costs stays one cheap shift per rebase.
pub fn cell_for_speed(speed: f64, dt: f64) -> f64 {
    let mut cell = ORIGIN_CELL_M;
    let step = (speed.abs() * dt.max(0.0)).min(1e12);
    while step > cell * 0.25 && cell < 1.0e9 {
        cell *= 2.0;
    }
    cell
}

/// The local simulation origin: an f64 point in `frame`, and the f32 local
/// space around it that Avian, the simulation and the renderer use. The
/// local axes are `frame`'s axes; the local space does not move in `frame`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimOrigin {
    pub frame: FrameId,
    pub origin: DVec3,
}

/// A rebase: every local position moves by `-shift` (exact in f32); no
/// velocity changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rebase {
    pub shift: Vec3,
    pub delta: DVec3,
}

impl SimOrigin {
    pub const fn new(frame: FrameId, origin: DVec3) -> Self {
        Self { frame, origin }
    }

    /// The point `local` in the origin's frame. f32 to f64 is exact.
    pub fn to_world(&self, local: Vec3) -> WorldPos {
        WorldPos::new(self.frame, self.origin + local.as_dvec3())
    }

    /// `world` in local space, subtracted in f64 and cast after. `world`
    /// must be in the origin's frame; convert it through the tree first.
    pub fn to_local(&self, world: WorldPos) -> Vec3 {
        debug_assert_eq!(world.frame, self.frame, "a position from another frame");
        (world.pos - self.origin).as_vec3()
    }

    /// Up at `local`: away from the frame's centre, in f64 before the cast.
    pub fn up_at(&self, local: Vec3) -> Vec3 {
        (self.origin + local.as_dvec3())
            .try_normalize()
            .unwrap_or(DVec3::Y)
            .as_vec3()
    }

    /// The distance of `local` from the frame's centre, in f64.
    pub fn radius_at(&self, local: Vec3) -> f64 {
        (self.origin + local.as_dvec3()).length()
    }

    /// `local` moved along its own up to `radius` from the frame's centre,
    /// in f64: how a body is stood on the ground or kept at a height without
    /// rounding at the planet's scale.
    pub fn with_radius(&self, local: Vec3, radius: f64) -> Vec3 {
        let w = self.origin + local.as_dvec3();
        let dir = w.try_normalize().unwrap_or(DVec3::Y);
        (dir * radius - self.origin).as_vec3()
    }

    /// The local point at `radius` along the direction `up` (a unit vector
    /// in the frame's axes).
    pub fn at(&self, up: DVec3, radius: f64) -> Vec3 {
        (up.normalize() * radius - self.origin).as_vec3()
    }

    /// `local` as an f32 point in the frame, for a query that is happy with
    /// half a millimetre at planet scale (a terrain column, a town wall).
    /// Never stored, never integrated.
    pub fn query_point(&self, local: Vec3) -> Vec3 {
        self.to_world(local).pos.as_vec3()
    }

    /// The local point for an f32 frame point a query handed back (a deck's
    /// place): the half-millimetre it holds is the query's.
    pub fn from_query_point(&self, frame_point: Vec3) -> Vec3 {
        (frame_point.as_dvec3() - self.origin).as_vec3()
    }

    /// The origin snapped to the grid for an anchor at `anchor` (local).
    /// `None` while the anchor is within [`REBASE_SLACK_CELLS`] of it.
    pub fn rebase_for(&self, anchor: Vec3, cell: f64) -> Option<Rebase> {
        let world = self.origin + anchor.as_dvec3();
        if !world.is_finite() {
            return None;
        }
        let off = anchor.as_dvec3();
        if off.abs().max_element() <= cell * REBASE_SLACK_CELLS {
            return None;
        }
        let snapped = (world / cell).round() * cell;
        let delta = snapped - self.origin;
        if delta == DVec3::ZERO {
            return None;
        }
        Some(Rebase {
            shift: delta.as_vec3(),
            delta,
        })
    }

    /// The origin after `rebase`.
    pub fn rebased(&self, rebase: Rebase) -> Self {
        Self::new(self.frame, self.origin + rebase.delta)
    }

    /// The handoff of the local space from this origin to `to` (another
    /// frame, or the same frame at another place) at time `t`.
    pub fn handoff(&self, tree: &FrameTree, to: SimOrigin, t: f64) -> Handoff {
        // The old local space as seen from the new: the old frame in the new
        // frame, shifted by both origins.
        let old_in_new = tree.in_frame(self.frame, to.frame, t);
        let origin = old_in_new.to_global(KinematicState {
            position: self.origin,
            velocity: DVec3::ZERO,
        });
        Handoff {
            from: *self,
            to,
            rel: LocalFrame {
                origin: origin.position - to.origin,
                velocity: origin.velocity,
                orientation: old_in_new.orientation,
                angular_velocity: old_in_new.angular_velocity,
            },
        }
    }
}

/// Moving the local space into another frame: every local position,
/// velocity and orientation goes through it at the same tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handoff {
    pub from: SimOrigin,
    pub to: SimOrigin,
    /// The old local space as seen from the new one.
    pub rel: LocalFrame,
}

impl Handoff {
    /// A local position and velocity in the old space, in the new.
    pub fn state(&self, pos: Vec3, vel: Vec3) -> (Vec3, Vec3) {
        let s = self.rel.to_global(KinematicState {
            position: pos.as_dvec3(),
            velocity: vel.as_dvec3(),
        });
        (s.position.as_vec3(), s.velocity.as_vec3())
    }

    pub fn position(&self, pos: Vec3) -> Vec3 {
        self.state(pos, Vec3::ZERO).0
    }

    /// An orientation (a body's rotation) in the old space, in the new.
    pub fn rotation(&self, q: Quat) -> Quat {
        (self.rel.orientation * q.as_dquat()).normalize().as_quat()
    }

    /// An angular velocity in the old space's axes, in the new.
    pub fn angular_velocity(&self, w: Vec3) -> Vec3 {
        (self.rel.orientation * w.as_dvec3() + self.rel.angular_velocity).as_vec3()
    }
}

/// The guard (`world-frames` 6): whether `local` is a local value at all.
/// In a debug build a value past [`LOCAL_BOUND_M`] prints
/// `ORIGIN_LEAK <site> <value>` once per site, which is a planet-frame
/// number used as a local one. Free in a release build.
#[inline]
pub fn check_local(site: &'static str, local: Vec3) -> bool {
    let ok = !local.is_finite() || local.abs().max_element() <= LOCAL_BOUND_M;
    #[cfg(debug_assertions)]
    if !ok {
        leak(site, local);
    }
    #[cfg(not(debug_assertions))]
    let _ = site;
    ok
}

#[cfg(debug_assertions)]
#[cold]
fn leak(site: &'static str, local: Vec3) {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static SEEN: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    if seen.get_or_insert_with(HashSet::new).insert(site) {
        eprintln!("ORIGIN_LEAK {site} {local}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AU: f64 = 1.495_978_707e11;

    fn spin(rate: f64) -> FrameMotion {
        FrameMotion::Spin {
            axis: DVec3::new(0.1, 1.0, 0.2).normalize(),
            rate,
            phase: 0.3,
        }
    }

    /// A system with a planet at 1 AU, its turning surface, a moon, and a
    /// second planet: the frames a trip passes through.
    fn system() -> (FrameTree, FrameId, FrameId, FrameId, FrameId) {
        let mut tree = FrameTree::new();
        let planet = tree.add(
            SYSTEM,
            FrameKind::BodyInertial,
            FrameMotion::Rails(
                CircularOrbit::new(AU, 3.156e7, 0.4, DQuat::from_rotation_x(0.05)).unwrap(),
            ),
        );
        let surface = tree.add(planet, FrameKind::BodyFixed, spin(7.29e-5));
        let other = tree.add(
            SYSTEM,
            FrameKind::BodyInertial,
            FrameMotion::Rails(
                CircularOrbit::new(1.52 * AU, 5.94e7, 2.1, DQuat::IDENTITY).unwrap(),
            ),
        );
        let other_surface = tree.add(other, FrameKind::BodyFixed, spin(7.09e-5));
        (tree, planet, surface, other, other_surface)
    }

    #[test]
    fn a_rebase_keeps_every_world_position_and_moves_by_whole_cells() {
        let o = SimOrigin::new(FrameId(1), DVec3::new(1234.0, 4512.0, -2208.0));
        let bodies = [
            Vec3::new(13.25, -0.125, 3.0),
            Vec3::new(-7.5, 1.6, 12.0),
            Vec3::new(0.001, 0.0, -0.002),
        ];
        let before: Vec<_> = bodies.iter().map(|&b| o.to_world(b)).collect();
        let anchor = Vec3::new(100.0, 0.0, 0.0);
        let r = o.rebase_for(anchor, ORIGIN_CELL_M).expect("off the cell");
        assert_eq!(r.shift.as_dvec3(), r.delta, "exact in f32");
        let n = o.rebased(r);
        for (b, w) in bodies.iter().zip(&before) {
            let moved = *b - r.shift;
            // The shift is exact; what rounds is the result, to the f32 step
            // at its own size: 8 µm at 128 m.
            assert!((n.to_world(moved).pos - w.pos).length() < 1e-5);
        }
        assert!(
            n.rebase_for(Vec3::new(100.0, 0.0, 0.0) - r.shift, ORIGIN_CELL_M)
                .is_none(),
            "the slack: no rebase back"
        );
    }

    /// A slow walk 4 800 m from the centre, and 4 800 km: a step of 1 m/s
    /// at 60 Hz integrated in local f32 moves by `v dt` to a micrometre,
    /// where the same step in planet-centred f32 is off by up to a few
    /// tenths of a millimetre.
    #[test]
    fn a_slow_step_far_from_the_centre_is_exact_in_local_space() {
        let dt = 1.0f32 / 60.0;
        for radius in [4_800.0, 4_800_000.0] {
            let up = DVec3::new(0.3, 0.9, -0.31).normalize();
            let start = up * radius;
            let east = DVec3::Y.cross(up).normalize();
            let v = east.as_vec3() * 1.0;
            let mut o = SimOrigin::new(FrameId(1), (start / 16.0).round() * 16.0);
            let mut local = o.to_local(WorldPos::new(FrameId(1), start));
            let mut planet = start.as_vec3();
            let mut worst_local = 0.0f64;
            let mut worst_planet = 0.0f64;
            for _ in 0..600 {
                let was = o.to_world(local).pos;
                let was_planet = planet.as_dvec3();
                local += v * dt;
                planet += v * dt;
                let moved = o.to_world(local).pos - was;
                let ideal = (v * dt).as_dvec3();
                worst_local = worst_local.max((moved - ideal).length());
                worst_planet = worst_planet.max((planet.as_dvec3() - was_planet - ideal).length());
                if let Some(r) = o.rebase_for(local, ORIGIN_CELL_M) {
                    let w = o.to_world(local).pos;
                    local -= r.shift;
                    o = o.rebased(r);
                    assert!((o.to_world(local).pos - w).length() < 1e-9);
                }
            }
            assert!(worst_local < 1e-5, "local {worst_local} at {radius}");
            assert!(worst_planet > 1e-5, "planet {worst_planet} at {radius}");
        }
    }

    #[test]
    fn conversions_round_trip_between_any_two_frames() {
        let (tree, planet, surface, other, other_surface) = system();
        let t = 1.234e6;
        let s = FrameState {
            frame: surface,
            pos: DVec3::new(1200.0, 4700.0, -300.0),
            vel: DVec3::new(1.0, 0.0, -0.5),
        };
        for to in [SYSTEM, planet, other, other_surface] {
            let there = tree.convert(s, to, t);
            let back = tree.convert(there, surface, t);
            let tol = if to == planet { 1e-9 } else { 1e-3 };
            assert!((back.pos - s.pos).length() < tol, "{to:?}");
            assert!((back.vel - s.vel).length() < 1e-6, "{to:?}");
        }
        // A point fixed on a turning surface moves at ω × r in the body's
        // inertial frame.
        let still = FrameState {
            frame: surface,
            pos: DVec3::new(0.0, 0.0, 4800.0),
            vel: DVec3::ZERO,
        };
        let inertial = tree.convert(still, planet, t);
        let w = tree.in_parent(surface, t).angular_velocity;
        assert!((inertial.vel - w.cross(inertial.pos)).length() < 1e-9);
    }

    /// A velocity differenced from positions agrees with the converted one,
    /// through a turning surface on a planet at 1 AU.
    #[test]
    fn a_converted_velocity_is_the_rate_of_the_converted_position() {
        let (tree, planet, surface, ..) = system();
        let t = 5.0e5;
        let s = FrameState {
            frame: surface,
            pos: DVec3::new(10.0, 4800.0, 3.0),
            vel: DVec3::new(2.0, 0.0, 1.0),
        };
        let dt = 1e-3;
        let a = tree.convert(s, planet, t);
        let b = tree.convert(
            FrameState {
                pos: s.pos + s.vel * dt,
                ..s
            },
            planet,
            t + dt,
        );
        assert!((((b.pos - a.pos) / dt) - a.vel).length() < 1e-4);
    }

    /// A ship cruising at 10 km/s through the system with a walker inside it
    /// walking at 1 m/s: in the ship's frame the walk is as exact as on the
    /// ground, whatever the ship's speed and wherever it is, and the origin
    /// may rebase every tick.
    #[test]
    fn a_walker_in_a_ship_at_ten_km_per_second_steps_exactly() {
        let (mut tree, planet, ..) = system();
        let ship = tree.add(
            planet,
            FrameKind::Vessel,
            FrameMotion::Vessel {
                pose: LocalFrame::moving(DVec3::new(3.0e7, 1.0e6, -2.0e7), DVec3::X * 10_000.0),
                at: 0.0,
            },
        );
        let dt = 1.0 / 60.0;
        let mut o = SimOrigin::new(ship, DVec3::new(32.0, 0.0, 0.0));
        let mut local = Vec3::new(-3.0, 1.0, 2.0);
        let v = Vec3::new(0.0, 0.0, 1.0);
        let mut worst = 0.0f64;
        for tick in 0..240 {
            let was = o.to_world(local).pos;
            local += v * dt as f32;
            worst = worst.max((o.to_world(local).pos - was - (v * dt as f32).as_dvec3()).length());
            // Rebase every tick, by a cell, to show it costs nothing.
            let r = Rebase {
                shift: Vec3::new(if tick % 2 == 0 { 16.0 } else { -16.0 }, 0.0, 0.0),
                delta: DVec3::new(if tick % 2 == 0 { 16.0 } else { -16.0 }, 0.0, 0.0),
            };
            let w = o.to_world(local).pos;
            local -= r.shift;
            o = o.rebased(r);
            assert!((o.to_world(local).pos - w).length() < 1e-9);
        }
        assert!(worst < 1e-5, "{worst}");
        // The same walker seen from the planet moves at the ship's speed
        // plus its own.
        let s = tree.convert(
            FrameState {
                frame: ship,
                pos: o.to_world(local).pos,
                vel: v.as_dvec3(),
            },
            planet,
            4.0,
        );
        assert!((s.vel - DVec3::new(10_000.0, 0.0, 1.0)).length() < 1e-9);
    }

    /// The trip: a surface, the planet's inertial frame (orbit), the system,
    /// the second planet and its surface, and back. At each handoff the
    /// walker's position and velocity in the system are unchanged.
    #[test]
    fn a_handoff_through_the_system_keeps_position_and_velocity() {
        let (tree, planet, surface, other, _) = system();
        let t = 2.5e6;
        let start = SimOrigin::new(surface, DVec3::new(0.0, 4800.0, 0.0));
        let mut o = start;
        let mut pos = Vec3::new(1.5, 2.0, -0.75);
        let mut vel = Vec3::new(3.0, 0.5, -1.0);
        let in_system = |o: &SimOrigin, p: Vec3, v: Vec3| {
            tree.convert(
                FrameState {
                    frame: o.frame,
                    pos: o.to_world(p).pos,
                    vel: v.as_dvec3(),
                },
                SYSTEM,
                t,
            )
        };
        let first = in_system(&o, pos, vel);
        // A turning frame is handed to only near its body: far from it the
        // ground's `ω × r` is millions of m/s (`world-frames` design 3).
        for to in [planet, SYSTEM, other, SYSTEM, planet, surface] {
            // The new origin sits on the walker, in the new frame, snapped.
            let here = tree.convert_pos(o.to_world(pos), to, t).pos;
            let new = SimOrigin::new(to, (here / 16.0).round() * 16.0);
            let h = o.handoff(&tree, new, t);
            let (p, v) = h.state(pos, vel);
            let before = in_system(&o, pos, vel);
            let after = in_system(&new, p, v);
            // Through the system the positions hold f64's 30 µm at 1 AU.
            assert!((after.pos - before.pos).length() < 1e-3, "{to:?}");
            // A local velocity is f32: relative to a frame it moves through
            // at tens of km/s (the system, another planet) it holds a few
            // mm/s, which is why a trip is flown in the ship's own frame.
            let tol = 1e-6 * before.vel.length().max(1.0);
            assert!((after.vel - before.vel).length() < tol, "{to:?}");
            assert!(p.length() < 32.0, "the local point stays near the origin");
            o = new;
            pos = p;
            vel = v;
        }
        let last = in_system(&o, pos, vel);
        assert!((last.pos - first.pos).length() < 1e-3);
        assert!((last.vel - first.vel).length() < 1e-6 * first.vel.length());
        assert_eq!(o.frame, start.frame);
    }

    /// Arriving at the second planet: from its inertial frame to its turning
    /// surface, 200 m over the ground, nothing jumps.
    #[test]
    fn an_arrival_hands_off_to_the_surface_without_a_jump() {
        let (tree, _, _, other, other_surface) = system();
        let t = 7.0e5;
        let o = SimOrigin::new(other, DVec3::new(0.0, 3_400_000.0, 0.0));
        let (pos, vel) = (Vec3::new(2.0, 200.0, -1.0), Vec3::new(-40.0, -3.0, 5.0));
        let here = tree.convert_pos(o.to_world(pos), other_surface, t).pos;
        let new = SimOrigin::new(other_surface, (here / 16.0).round() * 16.0);
        let (p, v) = o.handoff(&tree, new, t).state(pos, vel);
        let before = tree.convert(
            FrameState {
                frame: other,
                pos: o.to_world(pos).pos,
                vel: vel.as_dvec3(),
            },
            SYSTEM,
            t,
        );
        let after = tree.convert(
            FrameState {
                frame: other_surface,
                pos: new.to_world(p).pos,
                vel: v.as_dvec3(),
            },
            SYSTEM,
            t,
        );
        assert!((after.pos - before.pos).length() < 1e-3);
        assert!((after.vel - before.vel).length() < 1e-4 * before.vel.length());
        // In the turning frame the ground's own speed (240 m/s at the
        // equator here) has come off.
        let w = tree.in_parent(other_surface, t).angular_velocity;
        let ground = w.cross(o.to_world(pos).pos);
        assert!(
            ((new_vel_in_other(&tree, other_surface, other, new, p, v, t)) - vel.as_dvec3())
                .length()
                < 1e-3
        );
        assert!(ground.length() > 1.0);
    }

    fn new_vel_in_other(
        tree: &FrameTree,
        from: FrameId,
        to: FrameId,
        o: SimOrigin,
        p: Vec3,
        v: Vec3,
        t: f64,
    ) -> DVec3 {
        tree.convert(
            FrameState {
                frame: from,
                pos: o.to_world(p).pos,
                vel: v.as_dvec3(),
            },
            to,
            t,
        )
        .vel
    }

    #[test]
    fn a_vessel_reparented_keeps_its_place_and_motion() {
        let (mut tree, planet, _, other, _) = system();
        let ship = tree.add(
            planet,
            FrameKind::Vessel,
            FrameMotion::Vessel {
                pose: LocalFrame::moving(DVec3::new(1.0e8, 0.0, 0.0), DVec3::Y * 9000.0),
                at: 0.0,
            },
        );
        let t = 100.0;
        let before = tree.in_frame(ship, SYSTEM, t);
        tree.reparent_vessel(ship, other, t);
        let after = tree.in_frame(ship, SYSTEM, t);
        assert!((after.origin - before.origin).length() < 1e-3);
        assert!((after.velocity - before.velocity).length() < 1e-6);
    }

    #[test]
    fn the_cell_grows_with_speed() {
        let dt = 1.0 / 60.0;
        assert_eq!(cell_for_speed(1.0, dt), ORIGIN_CELL_M);
        assert_eq!(cell_for_speed(600.0, dt), ORIGIN_CELL_M);
        assert_eq!(cell_for_speed(3_000.0, dt), 2.0 * ORIGIN_CELL_M);
        assert!(cell_for_speed(10_000.0, dt) >= 4.0 * 10_000.0 * dt);
    }

    #[test]
    fn a_leak_is_caught() {
        assert!(check_local("test-near", Vec3::new(12.0, -3.0, 1.0)));
        assert!(!check_local("test-far", Vec3::new(0.0, 4800.0, 0.0)));
    }

    #[test]
    fn up_and_radius_are_exact_far_from_the_centre() {
        let o = SimOrigin::new(FrameId(1), DVec3::new(0.0, 4_800_000.0, 16.0));
        let p = Vec3::new(0.25, 1.0, -0.5);
        let r = o.radius_at(p);
        let stood = o.with_radius(p, r + 0.001);
        assert!((o.radius_at(stood) - (r + 0.001)).abs() < 1e-6);
        assert!((o.up_at(p) - Vec3::Y).length() < 1e-5);
    }
}
