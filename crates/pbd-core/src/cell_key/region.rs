//! A cell's region and its place in it (`region-cell-keys` design 1 and 2).
//!
//! Within a face, a level-`L` lattice is a triangle `2^L` steps on a side.
//! `(i >> 6, j >> 6)` names a rhombus of 64 by 64 steps, and the rhombus is
//! two triangles of side 64: the upright one, where `(i & 63) + (j & 63) <
//! 64`, and the upside-down one beside it. A region is one of those
//! triangles, so it is about 180 m across on every body (a cell is 2.833 m on
//! every body). On the home world, level 11, it is the level-5 triangle that
//! `world-persistence` and `multiplayer` already plan for.
//!
//! ```text
//! region (u32): bit 31..27  26..23    22..12  11..1  0
//!               face        level-7   ri      rj     half
//! place  (u16): bit 15..12  11..6     5..0
//!               0           li        lj
//! ```
//!
//! Both are made from the point's canonical address, so a point on a face's
//! edge or an icosahedron corner has one region and one place. The pair is
//! one `u64`, `(region << 16) | place`, which sorts by region and then place.

use super::{Address, canonical};

/// Lattice steps along a region's side.
pub const SIDE: u32 = 64;
/// The bits of a place's `li` and `lj`.
const LOCAL: u32 = SIDE - 1;
/// The coarsest level a region address holds (`level - 7` in 4 bits).
pub const FIRST_LEVEL: u8 = 7;
/// The finest: 4 bits of level.
pub const LAST_LEVEL: u8 = FIRST_LEVEL + 15;
/// The highest `ri` or `rj`: 11 bits each, which with the face's closing
/// edge is level 17 (a 307 km radius at the gold standard's cell).
const MAX_ROW: u32 = (1 << 11) - 1;

/// The region the point `a` names lies in, or `None` for an address that is
/// not a lattice point or whose level or rows the address cannot hold.
pub fn region(a: Address) -> Option<u32> {
    let c = fits(a)?;
    let (ri, rj) = (c.i / SIDE, c.j / SIDE);
    let half = u32::from((c.i & LOCAL) + (c.j & LOCAL) >= SIDE);
    Some(
        u32::from(c.face) << 27
            | u32::from(c.level - FIRST_LEVEL) << 23
            | ri << 12
            | rj << 1
            | half,
    )
}

/// The point's place in its region: `(i & 63, j & 63)` of its canonical
/// address.
pub fn place(a: Address) -> Option<u16> {
    let c = fits(a)?;
    Some(((c.i & LOCAL) << 6 | (c.j & LOCAL)) as u16)
}

/// The region and place as one number that sorts by region, then place.
pub fn whole(a: Address) -> Option<u64> {
    Some(u64::from(region(a)?) << 16 | u64::from(place(a)?))
}

/// The canonical address a region and a place name, or `None` for a pair
/// no point has: a place outside its region's half, a point off its face,
/// or an address that is not its point's canonical one (that point belongs
/// to another face's region).
pub fn address(region: u32, place: u16) -> Option<Address> {
    if place >> 12 != 0 {
        return None;
    }
    let (li, lj) = (u32::from(place) >> 6, u32::from(place) & LOCAL);
    let half = region & 1;
    if u32::from(li + lj >= SIDE) != half {
        return None;
    }
    let a = Address {
        face: (region >> 27) as u8,
        level: ((region >> 23) & 0xf) as u8 + FIRST_LEVEL,
        i: ((region >> 12) & MAX_ROW) * SIDE + li,
        j: ((region >> 1) & MAX_ROW) * SIDE + lj,
    };
    (a.is_valid() && canonical(a) == a).then_some(a)
}

/// The region of a cell key ([`super::key`]): the region its edits are
/// digested and sent under.
pub fn of_key(key: u32) -> Option<u32> {
    super::unpack(key).and_then(region)
}

/// The level a region address is at.
pub fn level(region: u32) -> u8 {
    ((region >> 23) & 0xf) as u8 + FIRST_LEVEL
}

/// The canonical address of a valid point whose level and rows fit.
fn fits(a: Address) -> Option<Address> {
    if !(FIRST_LEVEL..=LAST_LEVEL).contains(&a.level) || !a.is_valid() {
        return None;
    }
    let c = canonical(a);
    (c.i / SIDE <= MAX_ROW && c.j / SIDE <= MAX_ROW).then_some(c)
}

#[cfg(test)]
mod tests {
    use super::super::{addresses, each_point, key, point_count};
    use super::*;
    use std::collections::HashMap;

    /// Every point of a level, each of its addresses: one region and place
    /// whichever address it is asked from, back to its canonical address,
    /// no two points sharing a pair, and no region holding more than a
    /// triangle of side 64 holds.
    fn every_point_round_trips(level: u8) {
        let mut seen: HashMap<u64, Address> = HashMap::new();
        let mut per_region: HashMap<u32, u32> = HashMap::new();
        let mut count = 0u64;
        each_point(level, |a| {
            count += 1;
            let (r, p) = (region(a).unwrap(), place(a).unwrap());
            for other in addresses(a) {
                assert_eq!(region(other), Some(r), "{other:?} of {a:?}");
                assert_eq!(place(other), Some(p), "{other:?} of {a:?}");
            }
            assert_eq!(address(r, p), Some(a), "region {r:#x} place {p:#x}");
            assert_eq!(level_of(r), level);
            let pair = u64::from(r) << 16 | u64::from(p);
            assert_eq!(whole(a), Some(pair));
            assert!(seen.insert(pair, a).is_none(), "{a:?} shares {pair:#x}");
            *per_region.entry(r).or_default() += 1;
        });
        assert_eq!(count, point_count(level));
        // An upright triangle of side 64 holds 64 * 65 / 2 points of its own
        // plus its closing edge; neither half holds more than 65 * 66 / 2.
        let most = per_region.values().copied().max().unwrap();
        assert!(most <= 65 * 66 / 2, "a region holds {most}");
    }

    fn level_of(r: u32) -> u8 {
        super::level(r)
    }

    /// `region-cell-keys` 2.1 at the gold standard's level 7, and level 8.
    #[test]
    fn every_point_of_l7_and_l8_has_one_region_and_place_and_back() {
        every_point_round_trips(7);
        every_point_round_trips(8);
    }

    /// The home world's level, 41 million points: every edge and corner
    /// point of every face, and a stride through the interiors.
    #[test]
    fn l11_and_l12_edges_corners_and_a_sample_round_trip() {
        for level in [11u8, 12] {
            let n = 1u32 << level;
            for face in 0..20u8 {
                let mut check = |i: u32, j: u32| {
                    let a = Address { face, level, i, j };
                    let c = canonical(a);
                    let (r, p) = (region(a).unwrap(), place(a).unwrap());
                    assert_eq!(address(r, p), Some(c), "{a:?}");
                    for other in addresses(a) {
                        assert_eq!((region(other), place(other)), (Some(r), Some(p)));
                    }
                };
                for t in 0..=n {
                    check(t, 0);
                    check(0, t);
                    check(t, n - t);
                }
                for i in (1..n).step_by(37) {
                    for j in (1..n - i).step_by(41) {
                        check(i, j);
                    }
                }
            }
        }
    }

    /// At level 11 a region is the level-5 triangle `world-persistence`
    /// planned: 20 faces of 32 x 32 rhombi of two halves, plus the closing
    /// rows, and the interior ones each hold the same points.
    #[test]
    fn at_l11_a_region_is_a_level_5_triangle() {
        let mut regions: HashMap<u32, u32> = HashMap::new();
        for face in 0..20u8 {
            for i in (0..2048u32).step_by(7) {
                for j in (0..2048 - i).step_by(7) {
                    let a = Address {
                        face,
                        level: 11,
                        i,
                        j,
                    };
                    *regions.entry(region(a).unwrap()).or_default() += 1;
                }
            }
        }
        let full: Vec<u32> = regions
            .keys()
            .copied()
            .filter(|r| {
                let (ri, rj) = ((r >> 12) & MAX_ROW, (r >> 1) & MAX_ROW);
                ri + rj < 31
            })
            .collect();
        // 20 faces x (31 * 32 / 2 rhombi wholly inside) x 2 halves.
        assert_eq!(full.len(), 20 * 496 * 2);
    }

    /// A pair no point has is refused: a place past 12 bits, a place in the
    /// wrong half, and an edge point asked for under the face that does not
    /// keep it.
    #[test]
    fn a_pair_no_point_has_is_refused() {
        let a = Address {
            face: 3,
            level: 11,
            i: 100,
            j: 5,
        };
        let (r, p) = (region(a).unwrap(), place(a).unwrap());
        assert_eq!(address(r, p | 1 << 12), None);
        assert_eq!(address(r ^ 1, p), None);
        // Face 19's edge point (0, t) is kept by a lower face.
        let edge = Address {
            face: 19,
            level: 11,
            i: 0,
            j: 70,
        };
        let kept = canonical(edge);
        assert_ne!(kept.face, 19);
        let as_19 = u32::from(19u8) << 27 | 4 << 23 | 1 << 1;
        assert_eq!(address(as_19, 6), None);
        assert!(region(edge).is_some());
    }

    /// A cell key's region is its address's region.
    #[test]
    fn a_keys_region_is_its_addresss() {
        let a = Address {
            face: 7,
            level: 11,
            i: 1500,
            j: 300,
        };
        assert_eq!(of_key(key(a).unwrap()), region(a));
        assert_eq!(of_key(u32::MAX), None);
    }
}
