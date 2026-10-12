//! A cell's key: its lattice address, packed, so no two cells share one.
//!
//! A fine cell is a vertex of the subdivided icosahedron, addressed as the
//! lattice point `(i, j)` of face `f` at level `L`, with `i + j <= 2^L`
//! (`planet_lattice.rs`). Its saved identity used to be [`old_hash`], a mix of
//! that address into 32 bits. About one finest cell in two hundred shares its
//! hash with another cell elsewhere on the planet, so an edit to one was also
//! an edit to the other (`openspec/changes/exact-cell-keys`).
//!
//! [`key`] packs the address instead, which cannot collide:
//!
//! ```text
//! bit 31    30..26    25..24       23..12   11..0
//!   0       face      level - 8    i        j
//! ```
//!
//! A point on a face's edge lies on two faces, and a corner of the
//! icosahedron on five, so it has more than one address. [`canonical`] picks
//! one: the lowest-numbered face the point lies on. The key is a pure
//! function of the point, whichever face's triangle met it first.
//!
//! The old hash stays the seed the shaders roll clutter and texture from, so
//! the ground looks exactly as it did. Only what an edit is saved and found
//! by changes.

use crate::topology::ICOSAHEDRON_FACES;

/// The coarsest level that carries a key: the first fine level.
pub const FIRST_LEVEL: u8 = 8;
/// The finest level: the one a player edits.
pub const LAST_LEVEL: u8 = 11;

/// A lattice point by address. The same point has one address per face it
/// lies on; [`canonical`] picks the one its key is made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address {
    pub face: u8,
    pub level: u8,
    pub i: u32,
    pub j: u32,
}

impl Address {
    /// The lattice's side at this level: `i + j` never exceeds it.
    pub fn side(self) -> u32 {
        1 << self.level
    }

    /// Whether the address names a point of its face.
    pub fn is_valid(self) -> bool {
        (self.face as usize) < ICOSAHEDRON_FACES.len()
            && self.level <= 30
            && self
                .i
                .checked_add(self.j)
                .is_some_and(|sum| sum <= self.side())
    }

    /// The point's weight on each of its face's three corners: how many
    /// lattice steps it is from the opposite side. A corner it does not
    /// lean on at all has weight nought, which is what places it on an edge.
    fn weights(self) -> [u32; 3] {
        [self.side() - self.i - self.j, self.i, self.j]
    }
}

/// `point_id`'s mix of an address, as the LOD records carried it before
/// exact keys. It stays the seed the shaders roll clutter and texture
/// variation from, and the migration reads it to find what an old save's
/// edit meant. It collides: never save by it.
pub fn old_hash(a: Address) -> u32 {
    let mut h = (a.face as u32).wrapping_mul(0x9e37_79b9)
        ^ a.i.wrapping_mul(0x85eb_ca6b)
        ^ a.j.wrapping_mul(0xc2b2_ae35)
        ^ (a.level as u32) << 27;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^ (h >> 12)
}

/// The address a point's key is made from: its own where it lies inside a
/// face, and otherwise the lowest-numbered face that holds every corner it
/// leans on. A point on an edge leans on that edge's two corners, and a
/// corner of the icosahedron on itself.
pub fn canonical(a: Address) -> Address {
    debug_assert!(a.is_valid(), "not a lattice point: {a:?}");
    let corners = ICOSAHEDRON_FACES[a.face as usize];
    let weights = a.weights();
    if weights.iter().all(|&w| w > 0) {
        return a;
    }
    let weight_of = |vertex: usize| {
        corners
            .iter()
            .position(|&c| c == vertex)
            .map_or(0, |k| weights[k])
    };
    let leans: Vec<usize> = (0..3)
        .filter(|&k| weights[k] > 0)
        .map(|k| corners[k])
        .collect();
    let face = ICOSAHEDRON_FACES
        .iter()
        .position(|f| leans.iter().all(|v| f.contains(v)))
        .expect("the point's own face holds its corners");
    let [_, b, c] = ICOSAHEDRON_FACES[face];
    Address {
        face: face as u8,
        level: a.level,
        i: weight_of(b),
        j: weight_of(c),
    }
}

/// Every address of the point `a` names: one, or two on an edge, or five at
/// a corner of the icosahedron. In face order, so the list is the same
/// whichever address it was asked from.
pub fn addresses(a: Address) -> Vec<Address> {
    let corners = ICOSAHEDRON_FACES[a.face as usize];
    let weights = a.weights();
    if weights.iter().all(|&w| w > 0) {
        return vec![a];
    }
    let leaning: Vec<(usize, u32)> = (0..3)
        .filter(|&k| weights[k] > 0)
        .map(|k| (corners[k], weights[k]))
        .collect();
    ICOSAHEDRON_FACES
        .iter()
        .enumerate()
        .filter(|(_, f)| leaning.iter().all(|(v, _)| f.contains(v)))
        .map(|(face, f)| {
            let weight_of = |vertex: usize| {
                leaning
                    .iter()
                    .find(|(v, _)| *v == vertex)
                    .map_or(0, |&(_, w)| w)
            };
            Address {
                face: face as u8,
                level: a.level,
                i: weight_of(f[1]),
                j: weight_of(f[2]),
            }
        })
        .collect()
}

/// The key of the point `a` names: its canonical address, packed. `None` for
/// a level outside the fine levels, which carry no key.
pub fn key(a: Address) -> Option<u32> {
    if !(FIRST_LEVEL..=LAST_LEVEL).contains(&a.level) || !a.is_valid() {
        return None;
    }
    let c = canonical(a);
    Some((c.face as u32) << 26 | ((c.level - FIRST_LEVEL) as u32) << 24 | c.i << 12 | c.j)
}

/// The canonical address a key was packed from, or `None` for a number no
/// cell has: bit 31 set, a face past twenty, a point off its face, or an
/// address that is not its point's canonical one.
pub fn unpack(key: u32) -> Option<Address> {
    if key >> 31 != 0 {
        return None;
    }
    let a = Address {
        face: (key >> 26) as u8,
        level: ((key >> 24) & 0b11) as u8 + FIRST_LEVEL,
        i: (key >> 12) & 0xfff,
        j: key & 0xfff,
    };
    (a.is_valid() && canonical(a) == a).then_some(a)
}

/// Every lattice point of `level` once, by its canonical address, in face
/// order and then `(i, j)` order: the enumeration the collision test, the
/// migration's reverse table and the pair finder share.
pub fn each_point(level: u8, mut visit: impl FnMut(Address)) {
    let n = 1u32 << level;
    for face in 0..ICOSAHEDRON_FACES.len() as u8 {
        for i in 0..=n {
            for j in 0..=n - i {
                let a = Address { face, level, i, j };
                // Interior points are always canonical; only an edge or a
                // corner needs the check that this face is the one it keeps.
                if (i > 0 && j > 0 && i + j < n) || canonical(a) == a {
                    visit(a);
                }
            }
        }
    }
}

/// How many points the lattice has at `level`: `10 * 4^level + 2`.
pub fn point_count(level: u8) -> u64 {
    10 * (1u64 << (2 * level as u64)) + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many values repeat an earlier one: the length less the distinct.
    fn repeats(mut values: Vec<u32>) -> usize {
        values.sort_unstable();
        values.windows(2).filter(|w| w[0] == w[1]).count()
    }

    /// The measurement that found the bug (`exact-cell-keys`, the proposal):
    /// over every interior finest cell, 202,571 repeat another's hash. It is
    /// pinned so the hash this module keeps as the seed is provably the one
    /// the saves were written with.
    #[test]
    fn the_old_hash_collides_as_measured() {
        let level = LAST_LEVEL;
        let n = 1u32 << level;
        let mut hashes = Vec::with_capacity(20 * ((n - 1) * (n - 2) / 2) as usize);
        for face in 0..20u8 {
            for i in 1..n {
                for j in 1..n - i {
                    hashes.push(old_hash(Address { face, level, i, j }));
                }
            }
        }
        assert_eq!(hashes.len(), 41_881_620);
        assert_eq!(repeats(hashes), 202_571);
    }

    /// Every finest point, from every one of its addresses, gets one key, and
    /// no other point has it: the distinct keys are exactly the points.
    #[test]
    fn every_finest_key_is_its_own() {
        let level = LAST_LEVEL;
        let n = 1u32 << level;
        let mut keys = Vec::with_capacity(point_count(level) as usize + 64_000);
        for face in 0..20u8 {
            for i in 0..=n {
                for j in 0..=n - i {
                    keys.push(key(Address { face, level, i, j }).expect("a fine level"));
                }
            }
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len() as u64, point_count(level));
    }

    #[test]
    fn every_point_is_enumerated_once_and_unpacks_to_itself() {
        for level in FIRST_LEVEL..=LAST_LEVEL {
            let mut count = 0u64;
            let mut last = None;
            each_point(level, |a| {
                count += 1;
                let k = key(a).expect("a fine level");
                assert_eq!(unpack(k), Some(a), "{a:?}");
                assert!(last < Some(k), "keys rise in enumeration order");
                last = Some(k);
            });
            assert_eq!(count, point_count(level), "level {level}");
        }
    }

    /// Both faces of every edge of the icosahedron agree on every point along
    /// it, so a cell on a seam gets one key from anchors on either side.
    #[test]
    fn every_edge_point_gets_one_key_from_either_side() {
        let level = LAST_LEVEL;
        let n = 1u32 << level;
        let mut edges = 0;
        for (fa, a) in ICOSAHEDRON_FACES.iter().enumerate() {
            for (fb, b) in ICOSAHEDRON_FACES.iter().enumerate().skip(fa + 1) {
                let shared: Vec<usize> = a.iter().copied().filter(|v| b.contains(v)).collect();
                if shared.len() != 2 {
                    continue;
                }
                edges += 1;
                let (u, v) = (shared[0], shared[1]);
                for t in 0..=n {
                    // The point t steps from u toward v, on each face.
                    let on = |face: usize, corners: &[usize; 3]| {
                        let weight = |c: usize| {
                            if c == u {
                                n - t
                            } else if c == v {
                                t
                            } else {
                                0
                            }
                        };
                        Address {
                            face: face as u8,
                            level,
                            i: weight(corners[1]),
                            j: weight(corners[2]),
                        }
                    };
                    let (p, q) = (on(fa, a), on(fb, b));
                    assert_eq!(key(p), key(q), "edge {u}-{v} step {t}");
                    assert_eq!(canonical(p), canonical(q));
                    assert!(addresses(p).contains(&q) && addresses(q).contains(&p));
                }
            }
        }
        assert_eq!(edges, 30);
    }

    #[test]
    fn a_corner_of_the_icosahedron_has_five_addresses_and_one_key() {
        let level = 9;
        let corner = Address {
            face: 0,
            level,
            i: 0,
            j: 0,
        };
        let all = addresses(corner);
        assert_eq!(all.len(), 5);
        let keys: Vec<_> = all.iter().map(|&a| key(a)).collect();
        assert!(keys.windows(2).all(|w| w[0] == w[1]));
    }

    #[test]
    fn numbers_no_cell_has_do_not_unpack() {
        assert_eq!(unpack(1 << 31), None);
        assert_eq!(unpack(20 << 26), None, "no face twenty");
        // i + j past the side at level 8.
        assert_eq!(unpack(200 << 12 | 100), None);
        // Face 1's (0, 0) is vertex 0, which face 0 keeps.
        assert_eq!(unpack(1 << 26), None);
        assert!(unpack(0).is_some(), "face 0's corner is canonical");
    }

    #[test]
    fn only_the_fine_levels_carry_a_key() {
        let a = |level| Address {
            face: 3,
            level,
            i: 1,
            j: 1,
        };
        assert_eq!(key(a(7)), None);
        assert!(key(a(8)).is_some());
        assert!(key(a(11)).is_some());
        assert_eq!(key(a(12)), None);
    }
}
