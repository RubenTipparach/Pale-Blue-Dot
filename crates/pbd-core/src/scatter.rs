//! The ground clutter a hand can take (`gather-by-hand`): where the draw puts
//! a cell's pebble, bush, dead shrub and fern, so F can pick one up.
//!
//! The draw rolls these in `planet_surface.wgsl` from the cell's clutter id
//! and never stores them. This is the same rule on the CPU, from the shader's
//! shared block (`clutter:shared`): the same hash, the same salts, the same
//! `f32` steps. `pbd-app`'s GPU test runs that block on an adapter against
//! [`pieces`], so the two cannot drift apart.
//!
//! What a player took is a stored record ([`CLUTTER_KIND`], [`Taken`]); the
//! pieces themselves are derived and never saved.

use crate::flora::BIOME_FLORA;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// The shader's `hash`: a PCG-style mix, in wrapping `u32` arithmetic.
pub fn hash(x: u32) -> u32 {
    let h = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let h = ((h >> ((h >> 28) + 4)) ^ h).wrapping_mul(277_803_737);
    (h >> 22) ^ h
}

/// The shader's `roll`: one decision about one cell, in `0..1`.
pub fn roll(id: u32, salt: u32) -> f32 {
    (hash(id ^ salt.wrapping_mul(2_654_435_761)) & 0x00ff_ffff) as f32 / 16_777_216.0
}

pub const SALT_BUSH: u32 = 0x53;
pub const SALT_ROCK: u32 = 0x54;
pub const SALT_SHRUB: u32 = 0x55;
pub const SALT_FERN: u32 = 0x57;

/// The shader's `grassy`: a top that grows grass (its material codes).
pub fn grassy(material: u32) -> bool {
    matches!(material, 2 | 3 | 7)
}

/// The shader's `bare`: earth, sand, snow or stone with nothing growing.
pub fn bare(material: u32) -> bool {
    matches!(material, 1 | 4 | 5 | 6)
}

/// A piece a hand can take, in the shader's order within a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Pebble = 0,
    Bush = 1,
    Shrub = 2,
    Fern = 3,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Pebble, Kind::Bush, Kind::Shrub, Kind::Fern];

    pub fn salt(self) -> u32 {
        match self {
            Kind::Pebble => SALT_ROCK,
            Kind::Bush => SALT_BUSH,
            Kind::Shrub => SALT_SHRUB,
            Kind::Fern => SALT_FERN,
        }
    }

    /// The piece's bit in a taken mask.
    pub fn bit(self) -> u8 {
        1 << self as u8
    }

    /// What a hand gets for it: the good's code and how many.
    pub fn gives(self) -> (&'static str, u16) {
        match self {
            Kind::Pebble => ("stone", 1),
            Kind::Bush => ("fibre", 2),
            Kind::Shrub | Kind::Fern => ("fibre", 1),
        }
    }

    /// What the hover prompt calls taking it.
    pub fn verb(self) -> &'static str {
        match self {
            Kind::Pebble => "pick up stone",
            Kind::Bush | Kind::Shrub | Kind::Fern => "pull fibre",
        }
    }
}

/// The chances and sizes the draw reads (`scatter.ron`, as the planet's
/// parameters pack them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Odds {
    pub rock_chance: f32,
    pub rock_size_m: f32,
    pub bush_chance: f32,
    pub bush_size_m: f32,
    pub shrub_chance: f32,
    pub shrub_size_m: f32,
    /// Before the biome's weight.
    pub fern_chance: f32,
}

/// One cell as the draw's record holds it.
#[derive(Clone, Copy, Debug)]
pub struct Cell<'a> {
    /// The clutter id (`metadata.w`).
    pub id: u32,
    /// The surface material code (`metadata.y`'s low byte).
    pub material: u32,
    /// The biome (`metadata.y`'s second byte, at most 7).
    pub biome: usize,
    pub axis: Vec3,
    /// The corner rays, `degree` of them.
    pub corners: &'a [Vec3],
    /// The cap's radius: the planet's plus the cell's height, m.
    pub radius: f32,
}

/// Whether `kind` grows on a cell (the shader's `clutter_grows`).
pub fn grows(kind: Kind, id: u32, material: u32, chance: f32) -> bool {
    let (green, dry) = (grassy(material), bare(material));
    let r = roll(id, kind.salt());
    match kind {
        Kind::Pebble => {
            let c = if dry { chance } else { chance * 0.3 };
            (green || dry) && r < c
        }
        Kind::Bush | Kind::Fern => green && r < chance,
        Kind::Shrub => (material == 4 || material == 6) && r < chance,
    }
}

/// Where `kind` stands on a cell (the shader's `clutter_seat`): a hashed
/// point on the cap, toward a hashed corner.
pub fn seat(kind: Kind, cell: &Cell) -> Vec3 {
    let salt = kind.salt();
    let degree = cell.corners.len().max(1) as u32;
    let k = (roll(cell.id, salt + 1) * degree as f32) as u32;
    let f0 = 0.15 + 0.60 * roll(cell.id, salt + 2);
    let corner = cell.corners[k.min(degree - 1) as usize];
    (cell.axis * (1.0 - f0) + corner * f0) * cell.radius
}

/// A piece on a cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub kind: Kind,
    /// Where it stands, body-local m.
    pub seat: Vec3,
    /// The middle of what is drawn, which an aim is measured to.
    pub centre: Vec3,
    /// How near the eye ray passes its centre to take it, m.
    pub reach: f32,
}

impl Odds {
    /// The chance `kind` is rolled against on a cell of `biome`.
    pub fn chance(&self, kind: Kind, biome: usize) -> f32 {
        match kind {
            Kind::Pebble => self.rock_chance,
            Kind::Bush => self.bush_chance,
            Kind::Shrub => self.shrub_chance,
            Kind::Fern => (self.fern_chance * BIOME_FLORA[biome.min(7)].fern).min(1.0),
        }
    }
}

/// The pieces a hand can take on a cell, in [`Kind`] order.
pub fn pieces(cell: &Cell, odds: &Odds) -> Vec<Piece> {
    let up = cell.axis.normalize_or(Vec3::Y);
    Kind::ALL
        .into_iter()
        .filter(|k| grows(*k, cell.id, cell.material, odds.chance(*k, cell.biome)))
        .map(|kind| {
            let seat = seat(kind, cell);
            // Each about as tall and wide as the draw makes it, never
            // smaller than a hand's easy aim.
            let (lift, reach) = match kind {
                Kind::Pebble => (odds.rock_size_m * 0.4, odds.rock_size_m * 1.4),
                Kind::Bush => (odds.bush_size_m * 0.5, odds.bush_size_m * 1.8),
                Kind::Shrub => (odds.shrub_size_m * 0.5, odds.shrub_size_m * 1.2),
                Kind::Fern => (0.3, 0.55),
            };
            Piece {
                kind,
                seat,
                centre: seat + up * lift,
                reach: reach.max(0.25),
            }
        })
        .collect()
}

/// The record kind of what a hand took from a cell, by the cell's key, and
/// its schema.
pub const CLUTTER_KIND: &str = "clutter";
pub const CLUTTER_SCHEMA: u32 = 1;

/// Game days a taken piece takes to grow back.
pub const REGROW_DAYS: f64 = 3.0;

/// A `clutter` record's body: when each piece was taken, in game days, by
/// [`Kind`] order; `None` for one never taken.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Taken {
    pub at: [Option<f64>; 4],
}

impl Taken {
    /// Whether `kind` is gone on game day `day`: taken, and not yet grown
    /// back.
    pub fn gone(&self, kind: Kind, day: f64) -> bool {
        self.at[kind as usize].is_some_and(|t| day >= t && day < t + REGROW_DAYS)
    }

    /// How far a piece is drawn shrunk into the ground on game day `day`,
    /// 0 whole to 1 gone: it sinks over `fade_days` once taken and rises over
    /// as long once grown back, never in one frame.
    pub fn sunk(&self, kind: Kind, day: f64, fade_days: f64) -> f32 {
        let Some(t) = self.at[kind as usize] else {
            return 0.0;
        };
        let fade = fade_days.max(1e-9);
        let since = day - t;
        if since < 0.0 {
            return 0.0;
        }
        let out = (since / fade).min(1.0);
        let back = ((since - REGROW_DAYS) / fade).clamp(0.0, 1.0);
        (out - back).clamp(0.0, 1.0) as f32
    }

    /// Take `kind` on day `day`; `false` when it is gone already.
    pub fn take(&mut self, kind: Kind, day: f64) -> bool {
        if self.gone(kind, day) {
            return false;
        }
        self.at[kind as usize] = Some(day);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn odds() -> Odds {
        Odds {
            rock_chance: 0.1,
            rock_size_m: 0.16,
            bush_chance: 0.05,
            bush_size_m: 0.34,
            shrub_chance: 0.14,
            shrub_size_m: 0.38,
            fern_chance: 0.06,
        }
    }

    /// The hash is the shader's: known values worked through by hand from
    /// its three steps.
    #[test]
    fn the_hash_is_the_shaders() {
        let by_hand = |x: u32| {
            let h = x.wrapping_mul(747796405).wrapping_add(2891336453);
            let h = ((h >> ((h >> 28) + 4)) ^ h).wrapping_mul(277803737);
            (h >> 22) ^ h
        };
        for x in [0u32, 1, 7, 0x51, 123_456_789, u32::MAX] {
            assert_eq!(hash(x), by_hand(x));
        }
        for id in 0..1000 {
            let r = roll(id, SALT_ROCK);
            assert!((0.0..1.0).contains(&r));
        }
    }

    /// The pieces grow at their chances, on their grounds: pebbles on bare
    /// ground at the full chance and grass at a third of it, bushes and
    /// ferns on grass only, shrubs on sand and snow only.
    #[test]
    fn each_piece_grows_at_its_chance_on_its_ground() {
        let o = odds();
        let share = |kind: Kind, material: u32| {
            (0..40_000u32)
                .filter(|&id| grows(kind, id, material, o.chance(kind, 0)))
                .count() as f32
                / 40_000.0
        };
        assert!((share(Kind::Pebble, 1) - 0.1).abs() < 0.01);
        assert!((share(Kind::Pebble, 2) - 0.03).abs() < 0.006);
        assert!((share(Kind::Bush, 2) - 0.05).abs() < 0.008);
        assert_eq!(share(Kind::Bush, 1), 0.0);
        assert!((share(Kind::Shrub, 4) - 0.14).abs() < 0.01);
        assert_eq!(share(Kind::Shrub, 2), 0.0);
        assert_eq!(share(Kind::Fern, 5), 0.0);
    }

    /// A seat lies on the cap, between its centre and the corner it leans
    /// to, and the same cell always answers the same.
    #[test]
    fn a_seat_lies_on_the_cap_toward_a_corner() {
        let axis = Vec3::Y;
        let corners: Vec<Vec3> = (0..6)
            .map(|k| {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                (axis + Vec3::new(a.cos(), 0.0, a.sin()) * 0.0003).normalize()
            })
            .collect();
        for id in 0..500u32 {
            let cell = Cell {
                id,
                material: 2,
                biome: 0,
                axis,
                corners: &corners,
                radius: 4_800.0,
            };
            for kind in Kind::ALL {
                let s = seat(kind, &cell);
                assert_eq!(s, seat(kind, &cell));
                let off = (s - axis * s.dot(axis)).length();
                assert!(off <= 0.0003 * 4_800.0 * 0.76, "{kind:?} {off}");
                assert!(off >= 0.0003 * 4_800.0 * 0.14, "{kind:?} {off}");
            }
        }
    }

    /// A taken piece is gone for three days, then grows back; it sinks and
    /// rises over the fade rather than in one frame; it cannot be taken
    /// twice while gone; and the record reads back.
    #[test]
    fn a_taken_piece_sinks_stays_gone_and_grows_back() {
        let mut t = Taken::default();
        assert!(!t.gone(Kind::Bush, 1.0));
        assert!(t.take(Kind::Bush, 1.0));
        assert!(!t.take(Kind::Bush, 1.5), "already gone");
        assert!(t.gone(Kind::Bush, 1.0));
        assert!(t.gone(Kind::Bush, 3.9));
        assert!(!t.gone(Kind::Bush, 4.0), "grown back");
        assert!(!t.gone(Kind::Pebble, 2.0), "only what was taken");
        let fade = 0.01;
        assert_eq!(t.sunk(Kind::Bush, 1.0, fade), 0.0);
        assert!((t.sunk(Kind::Bush, 1.005, fade) - 0.5).abs() < 1e-3);
        assert_eq!(t.sunk(Kind::Bush, 2.0, fade), 1.0);
        assert!((t.sunk(Kind::Bush, 4.005, fade) - 0.5).abs() < 1e-3);
        assert_eq!(t.sunk(Kind::Bush, 4.5, fade), 0.0);
        assert!(t.take(Kind::Bush, 4.5), "taken again once grown");
        let body = ron::to_string(&t).unwrap();
        assert_eq!(ron::from_str::<Taken>(&body).unwrap(), t);
    }
}
