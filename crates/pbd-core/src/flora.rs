//! Which cells grow a glowing flower (`lamps-and-lanterns` decision 8).
//!
//! Decided once, on the CPU, from the cell's exact key ([`crate::cell_key`]),
//! so the record that draws the flower and the bake that lights the ground
//! around it read one answer. The shader never rolls for it: a cell whose
//! record says it glows grows the flower, and its own roll only places and
//! turns it.

/// How bright a glowing flower is on the field's 0..15 scale: enough to light
/// its own cell and, faintly, the ones beside it, which with a step across
/// costing three levels ([`crate::light::ACROSS`]) is about a cell and a
/// half of meadow.
pub const GLOW_FLOWER_LEVEL: u8 = 6;

/// A salt of its own, so which cells glow owes nothing to anything else
/// hashed off the same key.
const SALT: u32 = 0x6c6f_7766;

/// Whether the cell with this exact key grows a glowing flower, for a share
/// `chance` (0..1) of cells. Deterministic: the same key answers the same on
/// every machine and in every build of the set.
pub fn glows(key: u32, chance: f32) -> bool {
    // Murmur3's 32-bit finaliser, which spreads neighbouring keys - and a
    // cell's neighbours have neighbouring keys - across the whole range.
    let mut h = key ^ SALT;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    ((h >> 8) as f32 / (1u32 << 24) as f32) < chance
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The share that glows is the chance asked for, over any run of keys,
    /// including a run of neighbouring ones.
    #[test]
    fn the_share_that_glows_is_the_chance() {
        for chance in [0.04_f32, 0.12, 0.5] {
            let keys = 200_000u32;
            let lit = (1_000_000..1_000_000 + keys)
                .filter(|&key| glows(key, chance))
                .count();
            let share = lit as f32 / keys as f32;
            assert!(
                (share - chance).abs() < chance * 0.05,
                "{share} for a chance of {chance}"
            );
        }
    }

    /// Nothing glows at a chance of nought, everything at one, and a key
    /// answers the same every time it is asked.
    #[test]
    fn the_ends_and_the_same_answer_twice() {
        assert!((0..10_000).all(|key| !glows(key, 0.0)));
        assert!((0..10_000).all(|key| glows(key, 1.0)));
        for key in [0, 7, 1_328_120_728, u32::MAX - 1] {
            assert_eq!(glows(key, 0.04), glows(key, 0.04));
        }
    }
}
