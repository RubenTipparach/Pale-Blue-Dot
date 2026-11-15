//! The plate field (`ocean-environments` design B10): where the sea floor
//! parts in a ridge and where it closes in a trench.
//!
//! A pure function of the seed. `PLATES` plates are seeded on the sphere
//! (a Fibonacci lattice, each point jittered by a hash), and each turns about
//! a pole of its own at a rate of its own. A direction belongs to the plate
//! whose seed is nearest after a low-frequency warp, so the boundaries wander
//! instead of running as great-circle arcs. At a boundary, the two plates'
//! motions along the line between their seeds say what it is: parting is a
//! ridge, closing is a trench.
//!
//! Nothing here is saved; the generator reads it (the trench's carve), and
//! so do the spawner and the map, through [`super::places`].

use crate::fauna::hash2;
use crate::planet_gen::fractal;
use glam::Vec3;

/// How many plates a body has.
pub const PLATES: usize = 14;

/// The warp's strength (unit-sphere offset) and its frequency.
const WARP: f32 = 0.18;
const WARP_SCALE: f32 = 2.5;
const SALT: u64 = 0x91a7_e5ed;

/// One plate: where its seed is, and its turn (an angular velocity: its
/// direction the pole, its length the rate, arbitrary units of 0.5 to 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plate {
    pub seed: Vec3,
    pub spin: Vec3,
}

/// The plates of a seed.
pub fn plates(seed: u64) -> [Plate; PLATES] {
    let unit = |k: u64, n: u64| (hash2(seed ^ SALT, k * 16 + n) >> 40) as f32 / (1u64 << 24) as f32;
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    std::array::from_fn(|i| {
        let k = i as u64;
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / PLATES as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let t = golden * i as f32;
        let lattice = Vec3::new(r * t.cos(), y, r * t.sin());
        let jitter = Vec3::new(unit(k, 0), unit(k, 1), unit(k, 2)) * 2.0 - Vec3::ONE;
        let seed_dir = (lattice + jitter * 0.35).normalize_or(lattice);
        let pole =
            (Vec3::new(unit(k, 3), unit(k, 4), unit(k, 5)) * 2.0 - Vec3::ONE).normalize_or(Vec3::Y);
        let rate = 0.5 + 0.5 * unit(k, 6);
        Plate {
            seed: seed_dir,
            spin: pole * rate,
        }
    })
}

/// What the plate field says at a direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boundary {
    /// The plate the direction is on.
    pub plate: usize,
    /// Its nearest neighbour across the boundary.
    pub other: usize,
    /// How far the boundary is, radians of arc (multiply by the radius for
    /// metres).
    pub arc: f32,
    /// How fast the two plates close across it: above nought a trench,
    /// below a ridge.
    pub closing: f32,
}

/// The plate field at a direction.
pub fn at(seed: u64, direction: Vec3) -> Boundary {
    at_with(&plates(seed), seed, direction)
}

/// [`at`] with the plates already in hand.
pub fn at_with(plates: &[Plate; PLATES], seed: u64, direction: Vec3) -> Boundary {
    let d = direction.normalize_or(Vec3::Y);
    let warp = Vec3::new(
        fractal(seed ^ SALT, d, WARP_SCALE, 2, 0.5, 2.0),
        fractal(seed ^ SALT ^ 1, d, WARP_SCALE, 2, 0.5, 2.0),
        fractal(seed ^ SALT ^ 2, d, WARP_SCALE, 2, 0.5, 2.0),
    );
    let p = (d + warp * WARP).normalize_or(d);
    let (mut a, mut b) = (0usize, 1usize);
    let (mut da, mut db) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for (i, plate) in plates.iter().enumerate() {
        let dot = plate.seed.dot(p);
        if dot > da {
            (b, db) = (a, da);
            (a, da) = (i, dot);
        } else if dot > db {
            (b, db) = (i, dot);
        }
    }
    let (sa, sb) = (plates[a].seed, plates[b].seed);
    // The bisecting plane of the two seeds; its normal points from b to a.
    let n = (sa - sb).normalize_or(Vec3::X);
    let arc = p.dot(n).clamp(-1.0, 1.0).asin().max(0.0);
    // Each plate's motion at the boundary, and how fast b comes on toward a.
    let on = (p - n * p.dot(n)).normalize_or(p);
    let (va, vb) = (plates[a].spin.cross(on), plates[b].spin.cross(on));
    let closing = (vb - va).dot(n);
    Boundary {
        plate: a,
        other: b,
        arc,
        closing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(n: usize) -> impl Iterator<Item = Vec3> {
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..n).map(move |i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let t = golden * i as f32;
            Vec3::new(r * t.cos(), y, r * t.sin())
        })
    }

    /// Every plate holds ground, the field is a function of the seed, and
    /// both kinds of boundary exist.
    #[test]
    fn every_plate_holds_ground_and_both_kinds_of_boundary_exist() {
        let seed = crate::planet_gen::TerrainConfig::TENEBRIS.seed;
        let mut held = [0usize; PLATES];
        let (mut ridges, mut trenches) = (0, 0);
        for d in sphere(20_000) {
            let b = at(seed, d);
            assert_eq!(b, at(seed, d), "a function of the seed");
            assert!(b.arc.is_finite() && b.arc >= 0.0 && b.closing.is_finite());
            held[b.plate] += 1;
            if b.arc < 0.02 {
                if b.closing > 0.2 {
                    trenches += 1;
                } else if b.closing < -0.2 {
                    ridges += 1;
                }
            }
        }
        for (i, n) in held.iter().enumerate() {
            assert!(*n > 20_000 / PLATES / 4, "plate {i} holds {n} of 20,000");
        }
        assert!(
            ridges > 100 && trenches > 100,
            "{ridges} ridge and {trenches} trench samples"
        );
        assert_ne!(plates(seed), plates(seed ^ 1), "another seed, other plates");
    }
}
