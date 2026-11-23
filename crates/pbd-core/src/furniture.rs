//! Furniture (`furniture`): the pieces a carpenter's bench makes, where one
//! stands in a cell, who sits or lies on it, and what a container holds.
//!
//! - [`Kind`]: the ten pieces, each a good of the same code, its footprint,
//!   its height and its use (design 2).
//! - **Where it stands** (design 3): in a cell, its back to one of the cell's
//!   sides, or in its middle. Footprints are rectangles in a regular hexagon
//!   of the standard cell ([`footprint`]); the building rules and a town's
//!   layout ask [`fits`], [`overlaps`] and [`in_doorway`] of them alike.
//! - [`model`]: each piece as one shaped form, polygons in the town atlas's
//!   own textures.
//! - [`Occupancy`] (design 7): who holds each place of each seat or bed.
//! - [`store`]: what a container holds, its record and its moves, and a town
//!   container's loot (design 9).

pub mod model;
pub mod store;

#[cfg(test)]
mod tests;

use glam::{DVec3, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The standard cell's width across its flats, metres (CLAUDE.md, the
/// Tenebris gold standard): the hexagon furniture is laid out in.
pub const CELL_M: f32 = 2.833;
/// The hexagon's inner radius.
pub const INNER_M: f32 = CELL_M / 2.0;
/// How far a piece's back stands in from its cell's edge: half a wall and a
/// hand's gap, metres.
pub const BACK_M: f32 = 0.16;
/// The clear space inside a door: across, and in from the wall, metres.
pub const DOOR_CLEAR_M: (f32, f32) = (1.1, 0.9);
/// The side number that means a cell's middle.
pub const MIDDLE: u8 = 6;

/// A piece of furniture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Kind {
    Chair,
    Stool,
    Bench,
    Table,
    Bed,
    Drawers,
    Closet,
    Chest,
    BandedChest,
    Crate,
}

/// What a piece is for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Use {
    /// Sat on: the seat's height over the floor, metres, and its places.
    Seat { seat_m: f32, places: u8 },
    /// Lain on: the top of the mattress, metres.
    Bed { top_m: f32 },
    /// Holds things: its slots.
    Store { slots: usize },
    /// Neither: a table.
    Surface,
}

impl Kind {
    pub const ALL: [Kind; 10] = [
        Kind::Chair,
        Kind::Stool,
        Kind::Bench,
        Kind::Table,
        Kind::Bed,
        Kind::Drawers,
        Kind::Closet,
        Kind::Chest,
        Kind::BandedChest,
        Kind::Crate,
    ];

    /// Its code: the good's, the recipe's, and `item.<code>.name`.
    pub fn code(self) -> &'static str {
        match self {
            Kind::Chair => "chair",
            Kind::Stool => "stool",
            Kind::Bench => "bench",
            Kind::Table => "table",
            Kind::Bed => "bed",
            Kind::Drawers => "drawers",
            Kind::Closet => "closet",
            Kind::Chest => "chest",
            Kind::BandedChest => "banded-chest",
            Kind::Crate => "crate",
        }
    }

    pub fn by_code(code: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.code() == code)
    }

    /// Its good.
    pub fn good(self) -> crate::goods::Good {
        crate::goods::Good::by_code(self.code()).expect("every piece is a good")
    }

    /// The piece a good is, if it is one.
    pub fn of_good(good: crate::goods::Good) -> Option<Kind> {
        Kind::by_code(good.code())
    }

    /// Its name: the good's.
    pub fn name(self) -> crate::locale::Localized {
        crate::locale::tr(&format!("item.{}.name", self.code()))
    }

    /// Its footprint: across (along the wall it backs on) and deep, metres.
    pub fn size_m(self) -> Vec2 {
        match self {
            Kind::Chair => Vec2::new(0.48, 0.48),
            Kind::Stool => Vec2::new(0.40, 0.40),
            Kind::Bench => Vec2::new(1.30, 0.42),
            Kind::Table => Vec2::new(1.05, 1.05),
            Kind::Bed => Vec2::new(0.98, 2.0),
            Kind::Drawers => Vec2::new(0.90, 0.48),
            Kind::Closet => Vec2::new(1.04, 0.58),
            Kind::Chest => Vec2::new(0.88, 0.50),
            Kind::BandedChest => Vec2::new(0.98, 0.56),
            Kind::Crate => Vec2::new(0.62, 0.62),
        }
    }

    /// Its height, metres: its highest point over the floor.
    pub fn height_m(self) -> f32 {
        match self {
            Kind::Chair => 0.95,
            Kind::Stool => 0.50,
            Kind::Bench => 0.46,
            Kind::Table => 0.76,
            Kind::Bed => 0.95,
            Kind::Drawers => 0.92,
            Kind::Closet => 1.95,
            Kind::Chest => 0.55,
            Kind::BandedChest => 0.62,
            Kind::Crate => 0.62,
        }
    }

    pub fn use_(self) -> Use {
        match self {
            Kind::Chair => Use::Seat {
                seat_m: 0.46,
                places: 1,
            },
            Kind::Stool => Use::Seat {
                seat_m: 0.50,
                places: 1,
            },
            Kind::Bench => Use::Seat {
                seat_m: 0.46,
                places: 2,
            },
            Kind::Table => Use::Surface,
            Kind::Bed => Use::Bed { top_m: 0.48 },
            Kind::Drawers => Use::Store { slots: 12 },
            Kind::Closet => Use::Store { slots: 20 },
            Kind::Chest => Use::Store { slots: 15 },
            Kind::BandedChest => Use::Store { slots: 24 },
            Kind::Crate => Use::Store { slots: 8 },
        }
    }

    /// Its slots, if it is a container.
    pub fn slots(self) -> Option<usize> {
        match self.use_() {
            Use::Store { slots } => Some(slots),
            _ => None,
        }
    }

    /// How many bodies it holds at once: a seat's places, a bed's one.
    pub fn places(self) -> u8 {
        match self.use_() {
            Use::Seat { places, .. } => places,
            Use::Bed { .. } => 1,
            _ => 0,
        }
    }

    /// Whether the walker is stopped by it: everything but a seat, which is
    /// walked up to and sat on.
    pub fn solid(self) -> bool {
        !matches!(self.use_(), Use::Seat { .. })
    }

    /// Where a body sits or lies on it, in its own frame (x across, y up, z
    /// toward its front, origin under its middle): a place's hips, and the
    /// way the body faces. A bed's place is its pillow end's middle, the body
    /// lying toward +z.
    pub fn place_at(self, place: u8) -> Option<(glam::Vec3, glam::Vec3)> {
        let z = glam::Vec3::Z;
        match self.use_() {
            Use::Seat { seat_m, places } => {
                if place >= places {
                    return None;
                }
                let x = if places == 1 {
                    0.0
                } else {
                    (f32::from(place) + 0.5) / f32::from(places) * self.size_m().x
                        - self.size_m().x / 2.0
                };
                Some((glam::Vec3::new(x, seat_m, 0.0), z))
            }
            Use::Bed { top_m } => (place == 0).then(|| (glam::Vec3::new(0.0, top_m, -0.62), z)),
            _ => None,
        }
    }
}

/// The unit normal of side `side` of the layout hexagon, pointing out of the
/// cell: side `s` at `s` sixths of a turn from +x toward +z.
pub fn side_normal(side: u8) -> Vec2 {
    let a = f32::from(side % 6) * std::f32::consts::FRAC_PI_3;
    Vec2::new(a.cos(), a.sin())
}

/// Where a piece stands in its cell's layout hexagon (centre at the origin):
/// its middle, and its own +z (its front) in the plan.
pub fn standing(kind: Kind, side: u8) -> (Vec2, Vec2) {
    if side >= MIDDLE {
        return (Vec2::ZERO, -side_normal(0));
    }
    let n = side_normal(side);
    let d = kind.size_m().y;
    (n * (INNER_M - BACK_M - d / 2.0), -n)
}

/// A rectangle's corners: `size` (across, deep) at `at`, its front `front`.
pub fn rect(at: Vec2, front: Vec2, size: Vec2) -> [Vec2; 4] {
    let z = front.normalize_or(Vec2::Y);
    let x = Vec2::new(z.y, -z.x);
    let (hx, hz) = (size.x / 2.0, size.y / 2.0);
    [
        at - x * hx - z * hz,
        at + x * hx - z * hz,
        at + x * hx + z * hz,
        at - x * hx + z * hz,
    ]
}

/// A piece's footprint against `side` (or in the middle) of the layout
/// hexagon.
pub fn footprint(kind: Kind, side: u8) -> [Vec2; 4] {
    let (at, front) = standing(kind, side);
    rect(at, front, kind.size_m())
}

/// The clear space inside a door in `side`'s wall.
pub fn door_clear(side: u8) -> [Vec2; 4] {
    let n = side_normal(side);
    let (w, d) = DOOR_CLEAR_M;
    rect(n * (INNER_M - BACK_M - d / 2.0), -n, Vec2::new(w, d))
}

/// Whether a footprint is inside the cell, clear of its walls: every corner
/// at least [`BACK_M`] in from every side (design 4, "Into a wall").
pub fn fits(points: &[Vec2; 4]) -> bool {
    points
        .iter()
        .all(|p| (0..6).all(|s| p.dot(side_normal(s)) <= INNER_M - BACK_M + 1e-4))
}

/// Whether two convex quads overlap (by the separating axis), touching
/// edges not counted.
pub fn overlaps(a: &[Vec2; 4], b: &[Vec2; 4]) -> bool {
    let axes = |q: &[Vec2; 4]| {
        (0..4)
            .map(|i| {
                let e = q[(i + 1) % 4] - q[i];
                Vec2::new(-e.y, e.x).normalize_or_zero()
            })
            .collect::<Vec<_>>()
    };
    for axis in axes(a).into_iter().chain(axes(b)) {
        let span = |q: &[Vec2; 4]| {
            q.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                let t = p.dot(axis);
                (lo.min(t), hi.max(t))
            })
        };
        let (a0, a1) = span(a);
        let (b0, b1) = span(b);
        if a1 <= b0 + 1e-3 || b1 <= a0 + 1e-3 {
            return false;
        }
    }
    true
}

/// Whether a piece against `side` stands in the clear space of a door in
/// any of `doors` (the sides of its cell with a door, on its storey).
pub fn in_doorway(kind: Kind, side: u8, doors: &[u8]) -> bool {
    let f = footprint(kind, side);
    doors.iter().any(|&d| overlaps(&f, &door_clear(d)))
}

/// Who holds a place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Who {
    Player,
    /// A townsperson: their site and their place in its people.
    Person {
        site: u32,
        index: u32,
    },
}

/// One place of one piece: the piece's id and the place's number.
pub type Place = (u64, u8);

/// Who holds each place of each seat and bed (design 7). A body holds one
/// place at a time; a place holds one body.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Occupancy {
    held: BTreeMap<Place, Who>,
}

impl Occupancy {
    /// Who holds a place.
    pub fn holder(&self, place: Place) -> Option<Who> {
        self.held.get(&place).copied()
    }

    /// The place a body holds.
    pub fn held_by(&self, who: Who) -> Option<Place> {
        self.held.iter().find(|(_, w)| **w == who).map(|(p, _)| *p)
    }

    /// Take a place for `who`: refused with its holder when another holds
    /// it. A body's last place is let go.
    pub fn claim(&mut self, place: Place, who: Who) -> Result<(), Who> {
        match self.holder(place) {
            Some(other) if other != who => return Err(other),
            Some(_) => return Ok(()),
            None => {}
        }
        self.release(who);
        self.held.insert(place, who);
        Ok(())
    }

    /// Let go whatever `who` holds.
    pub fn release(&mut self, who: Who) {
        self.held.retain(|_, w| *w != who);
    }

    /// The first free place of a piece with `places` places.
    pub fn free_place(&self, piece: u64, places: u8) -> Option<u8> {
        (0..places).find(|&k| !self.held.contains_key(&(piece, k)))
    }

    /// Every held place.
    pub fn iter(&self) -> impl Iterator<Item = (Place, Who)> + '_ {
        self.held.iter().map(|(p, w)| (*p, *w))
    }
}

/// A piece somewhere on the planet, as the townsfolk's schedule asks of it:
/// its id, its kind, and its middle on the floor (planet frame, f64).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub id: u64,
    pub kind: Kind,
    pub at: DVec3,
}

/// What a townsperson wants now (design 7): a bed, a seat, or neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Want {
    Bed,
    Seat,
    Nothing,
}

/// The townsfolk's schedule as a rule: by night every person but those who
/// keep watch wants a bed; by day those at a sitting post, and one in three
/// of the idle by their seed, want a seat (design 7).
pub fn want(night: bool, watch: bool, sitting_post: bool, idle: bool, seed: u32) -> Want {
    if watch {
        return Want::Nothing;
    }
    if night {
        return Want::Bed;
    }
    if sitting_post || (idle && seed % 3 == 0) {
        Want::Seat
    } else {
        Want::Nothing
    }
}

/// Whether a role keeps watch by night (a guard, a lookout, a watcher).
pub fn keeps_watch(role: &str) -> bool {
    role.contains("guard") || role.contains("lookout") || role.contains("watcher")
}

/// Seat or lay each person who wants it on the free place nearest their post
/// within `reach_m`, in their order: the place each takes. Places already
/// held (the player's) are left alone, and every body this rule placed
/// before is let go first, so the rule is the same from any start.
pub fn assign(
    people: &[(Who, DVec3, Want)],
    spots: &[Spot],
    occupancy: &mut Occupancy,
    reach_m: f64,
) -> Vec<(Who, Spot, u8)> {
    for (who, _, _) in people {
        occupancy.release(*who);
    }
    let mut out = Vec::new();
    for &(who, post, want) in people {
        let fits = |k: Kind| match want {
            Want::Bed => matches!(k.use_(), Use::Bed { .. }),
            Want::Seat => matches!(k.use_(), Use::Seat { .. }),
            Want::Nothing => false,
        };
        let best = spots
            .iter()
            .filter(|s| fits(s.kind) && (s.at - post).length() <= reach_m)
            .filter_map(|s| occupancy.free_place(s.id, s.kind.places()).map(|k| (s, k)))
            .min_by(|a, b| {
                (a.0.at - post)
                    .length()
                    .total_cmp(&(b.0.at - post).length())
                    .then(a.0.id.cmp(&b.0.id))
            });
        if let Some((spot, place)) = best
            && occupancy.claim((spot.id, place), who).is_ok()
        {
            out.push((who, *spot, place));
        }
    }
    out
}

/// A town piece's id (seats, beds and containers alike): its site, its
/// building's number and its own number in the building.
pub fn town_piece_id(site: u32, building: u32, n: u32) -> u64 {
    crate::river_cache::fnv(format!("furniture:town:{site}:{building}:{n}").as_bytes())
}
