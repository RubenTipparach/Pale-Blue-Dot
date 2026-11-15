//! Which cells grow a glowing flower (`lamps-and-lanterns` decision 8).
//!
//! Decided once, on the CPU, from the cell's exact key ([`crate::cell_key`]),
//! so the record that draws the flower and the bake that lights the ground
//! around it read one answer. The shader never rolls for it: a cell whose
//! record says it glows grows the flower, and its own roll only places and
//! turns it.

/// How bright a glowing flower is on the field's 0..15 scale: enough to light
/// its own cell faintly, and with a step across costing three levels
/// ([`crate::light::ACROSS`]) no further. It was 6 (about a cell and a half
/// of meadow) until the owner asked for a quarter of the brightness
/// (`night-lights-from-space`): the field's flood curve gives 0.41 at level 6
/// and 0.13 at level 3, the nearest whole level to a quarter.
pub const GLOW_FLOWER_LEVEL: u8 = 3;

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

/// The foliage pass's hash (`planet_visibility.wgsl`, `hash`): PCG's output
/// step, which the tree roll reads the low byte of.
pub fn shader_hash(x: u32) -> u32 {
    let mut h = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    h = ((h >> ((h >> 28) + 4)) ^ h).wrapping_mul(277_803_737);
    (h >> 22) ^ h
}

/// Trees per 256 cells in each biome, by the biome's index
/// (`planet_gen::Biome`): the foliage pass's `density`.
pub const TREE_DENSITY: [(u32, u32); 5] = [(4, 115), (5, 34), (2, 13), (1, 13), (7, 2)];

/// Whether a finest cell stands a tree (`tool-crafting` decision 3): the
/// foliage pass's rule (`planet_visibility.wgsl`, `has_nearby_foliage`)
/// on the CPU, so a tree the player sees is a tree the player can work.
///
/// `tree_id` is the cell record's tree lane (`spare.x`); `word` its material
/// word (`metadata.y`): the material index in the low byte, the biome in the
/// next, and bit 24 set in a town's cleared footprint. The distance and
/// partition tests the shader adds decide only what is drawn this frame.
pub fn grows_tree(tree_id: u32, word: u32) -> bool {
    if (word >> 24) & 1 == 1 {
        return false;
    }
    let roll = shader_hash(tree_id) & 0xff;
    let material = word & 0xff;
    let biome = (word >> 8) & 0xff;
    let grass = matches!(material, 2 | 3 | 7);
    let pine = biome == 7 && material == 6;
    let density = TREE_DENSITY
        .iter()
        .find(|(b, _)| *b == biome)
        .map_or(0, |(_, d)| *d);
    (grass || pine) && roll < density
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

    /// A smaller share keeps a subset of the cells a larger one chose: the
    /// owner's quarter (`night-lights-from-space`) thins the glowers, it
    /// does not move them.
    #[test]
    fn a_smaller_share_keeps_a_subset_of_the_glowers() {
        let keys = 1_000_000u32..1_200_000;
        let old: Vec<u32> = keys.clone().filter(|&k| glows(k, 0.04)).collect();
        let new: Vec<u32> = keys.filter(|&k| glows(k, 0.01)).collect();
        assert!(new.iter().all(|&k| glows(k, 0.04)));
        let share = new.len() as f32 / old.len() as f32;
        assert!(
            (share - 0.25).abs() < 0.02,
            "{share} of the old glowers stay"
        );
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

    /// The CPU's tree rule is the shader's: its hash and its densities, read
    /// off the shipped file (`tool-crafting` decision 3).
    #[test]
    fn the_tree_rule_is_the_foliage_pass_s() {
        let shader = include_str!("../../../assets/shaders/planet_visibility.wgsl");
        for line in [
            "var h = x*747796405u+2891336453u;",
            "h = ((h >> ((h >> 28u)+4u))^h)*277803737u;",
            "return (h>>22u)^h;",
            "let roll = hash(cell.spare.x) & 0xffu;",
            "let grass = material==2u || material==3u || material==7u;",
            "let pine = biome==7u && material==6u;",
            "if biome==4u { density = 115u; }",
            "if biome==5u { density = 34u; }",
            "if biome==2u || biome==1u { density = 13u; }",
            "if biome==7u { density = 2u; }",
            "let tree = (grass||pine) && roll < density;",
            "if ((cell.metadata.y >> 24u) & 1u) == 1u { return false; }",
        ] {
            assert!(
                shader.contains(line),
                "the foliage pass no longer says `{line}`"
            );
        }
        // About 5% of a field's grass stands a tree, none of its sand, and a
        // town's footprint none.
        let field = |material: u32| material | 2 << 8;
        let trees = (0..100_000u32)
            .filter(|&id| grows_tree(id, field(2)))
            .count();
        assert!((4_500..5_700).contains(&trees), "{trees}");
        assert!(!(0..10_000u32).any(|id| grows_tree(id, field(1))));
        assert!(!(0..10_000u32).any(|id| grows_tree(id, field(2) | 1 << 24)));
    }
}
