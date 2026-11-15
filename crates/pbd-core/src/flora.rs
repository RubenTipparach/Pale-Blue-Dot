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

// ---- Flora v2 (`flora-v2`): the per-biome looks and the plants' calendar ----

/// Flower colours a biome's flower heads are drawn in, as bits of
/// [`BiomeFlora::flowers`]; the shader's palette is in this order.
pub const FLOWER_RED: u32 = 1;
pub const FLOWER_YELLOW: u32 = 2;
pub const FLOWER_WHITE: u32 = 4;
pub const FLOWER_BLUE: u32 = 8;

/// What a biome's ground cover looks like (`flora-v2` design 3): the mockup's
/// biome table (`docs/mockups/foliage.html`, `BIOMES`) read onto the game's
/// eight biomes. Derived decoration only: nothing here decides a cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiomeFlora {
    /// Times `fern_chance`: how much more (or less) often a grassy cell grows
    /// a fern here.
    pub fern: f32,
    /// Times `mushroom_chance`.
    pub mushroom: f32,
    /// Which flower colours grow here, [`FLOWER_RED`] and the rest as bits.
    pub flowers: u32,
    /// Whether the trees drop their leaves in winter and turn in autumn.
    pub deciduous: bool,
    /// Red caps with white flecks, rather than brown.
    pub red_caps: bool,
}

/// [`BiomeFlora`] per biome, in `planet_gen::Biome` order: ocean, beach,
/// fields, desert, jungle, swamp, mountains, tundra.
pub const BIOME_FLORA: [BiomeFlora; 8] = [
    // Ocean: nothing grassy grows under the sea; white for a sandbar's flower.
    BiomeFlora {
        fern: 0.0,
        mushroom: 0.0,
        flowers: FLOWER_WHITE,
        deciduous: false,
        red_caps: false,
    },
    // Beach: dune grass with a few ferns, the savanna's yellow and white.
    BiomeFlora {
        fern: 0.3,
        mushroom: 0.0,
        flowers: FLOWER_YELLOW | FLOWER_WHITE,
        deciduous: true,
        red_caps: false,
    },
    // Fields: the temperate forest's four flowers, ferns and fly agarics.
    BiomeFlora {
        fern: 1.0,
        mushroom: 1.0,
        flowers: FLOWER_RED | FLOWER_YELLOW | FLOWER_WHITE | FLOWER_BLUE,
        deciduous: true,
        red_caps: true,
    },
    // Desert: no fern and no mushroom; a red bloom, rarely.
    BiomeFlora {
        fern: 0.0,
        mushroom: 0.0,
        flowers: FLOWER_RED,
        deciduous: false,
        red_caps: false,
    },
    // Jungle: ferns everywhere under the canopy, brown caps.
    BiomeFlora {
        fern: 3.5,
        mushroom: 1.2,
        flowers: FLOWER_RED | FLOWER_YELLOW,
        deciduous: false,
        red_caps: false,
    },
    // Swamp: the most mushrooms, groves that drop their leaves.
    BiomeFlora {
        fern: 1.0,
        mushroom: 2.0,
        flowers: FLOWER_WHITE | FLOWER_YELLOW,
        deciduous: true,
        red_caps: false,
    },
    // Mountains: a little of each, alpine white and blue.
    BiomeFlora {
        fern: 0.3,
        mushroom: 0.5,
        flowers: FLOWER_WHITE | FLOWER_BLUE,
        deciduous: false,
        red_caps: false,
    },
    // Tundra: the taiga's brown caps, white and blue flowers, evergreen pines.
    BiomeFlora {
        fern: 0.3,
        mushroom: 1.0,
        flowers: FLOWER_WHITE | FLOWER_BLUE,
        deciduous: false,
        red_caps: false,
    },
];

/// The pixel tuft's three tones per biome, root to tip, as `0xRRGGBB`, in
/// `planet_gen::Biome` order (`flora-v2` design 1). The mockup's ramps
/// (`foliage.html`, `TUFT_RAMP`); fields is the Play demo's own greens.
pub const TUFT_RAMPS: [[u32; 3]; 8] = [
    [0x0e4a3e, 0x2a7a64, 0x6ab8a0], // ocean: the kelp ramp
    [0x6b5a1e, 0xa8913a, 0xd8c070], // beach: the savanna ramp
    [0x05582d, 0x559642, 0x9bb675], // fields: temperate
    [0x5c5a2a, 0x8a8448, 0xb9b07a], // desert
    [0x0a4a24, 0x2f7d32, 0x6fb043], // jungle
    [0x2f3d16, 0x5a6a26, 0x8c9446], // swamp
    [0x1d3f2c, 0x3d6a48, 0x7c9a6a], // mountains: the taiga ramp
    [0x1d3f2c, 0x3d6a48, 0x7c9a6a], // tundra: taiga
];

/// Where the seasons start and reach full strength, as the sine of the
/// latitude (`flora-v2` design 5): none in the tropics, full from about 27
/// degrees. `planet_surface.wgsl`'s `SEASON_TROPIC` is the same pair.
pub const SEASON_TROPIC: [f32; 2] = [0.12, 0.45];
/// Autumn colour: its peak in a place's own year (0 its midsummer) and the
/// half-width of its bump. `SEASON_TURN` in the shader.
pub const SEASON_TURN: [f32; 2] = [0.27, 0.15];
/// Bare branches, peaking at midwinter. `SEASON_BARE`.
pub const SEASON_BARE: [f32; 2] = [0.5, 0.13];
/// Dry grass, a little before midwinter and wider. `SEASON_STRAW`.
pub const SEASON_STRAW: [f32; 2] = [0.47, 0.22];

/// How far into each of the plants' seasonal looks a place is, each 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Season {
    /// Deciduous leaves turned gold and red.
    pub turn: f32,
    /// Deciduous leaves fallen.
    pub bare: f32,
    /// Grass dried to straw.
    pub straw: f32,
}

fn smooth(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// One at `at[0]` on the circle of the year, falling smoothly to nought
/// `at[1]` either side of it.
fn bump(x: f32, at: [f32; 2]) -> f32 {
    let d = (x - at[0]).abs();
    1.0 - smooth(0.0, at[1], d.min(1.0 - d))
}

/// The plants' season at a place (`flora-v2` design 5): one calendar for the
/// planet (`daylight::Clock::year_fraction`, which starts at the +Y
/// hemisphere's midsummer), half a year on across the equator, and fading out
/// toward the equator. `latitude_sin` is the place's direction's +Y.
pub fn season_at(year_fraction: f64, latitude_sin: f32) -> Season {
    let own = if latitude_sin < 0.0 {
        year_fraction + 0.5
    } else {
        year_fraction
    };
    let local = own.rem_euclid(1.0) as f32;
    let strength = smooth(SEASON_TROPIC[0], SEASON_TROPIC[1], latitude_sin.abs());
    Season {
        turn: strength * bump(local, SEASON_TURN),
        bare: strength * bump(local, SEASON_BARE),
        straw: strength * bump(local, SEASON_STRAW),
    }
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

    /// Every biome has a tuft ramp that runs dark at the root to light at
    /// the tip, and its own flora row.
    #[test]
    fn every_biome_has_a_ramp_dark_to_light() {
        let luma = |c: u32| {
            let (r, g, b) = ((c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff);
            0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
        };
        assert_eq!(TUFT_RAMPS.len(), 8);
        for ramp in TUFT_RAMPS {
            assert!(ramp.iter().all(|c| *c <= 0xff_ffff));
            assert!(
                luma(ramp[0]) < luma(ramp[1]) && luma(ramp[1]) < luma(ramp[2]),
                "{ramp:x?}"
            );
        }
        // The fields' ramp is the Play demo's own greens.
        assert_eq!(TUFT_RAMPS[2], [0x05582d, 0x559642, 0x9bb675]);
    }

    /// The table's weights are weights, every biome grows some flower colour,
    /// and the desert grows no fern and no mushroom (the mockup's table).
    #[test]
    fn the_biome_table_is_the_mockups() {
        for row in BIOME_FLORA {
            assert!(row.fern >= 0.0 && row.mushroom >= 0.0);
            assert!(row.flowers != 0 && row.flowers <= 15);
        }
        let desert = BIOME_FLORA[3];
        assert_eq!((desert.fern, desert.mushroom), (0.0, 0.0));
        // The jungle has the most ferns, the swamp the most mushrooms.
        assert!(BIOME_FLORA.iter().all(|r| r.fern <= BIOME_FLORA[4].fern));
        assert!(
            BIOME_FLORA
                .iter()
                .all(|r| r.mushroom <= BIOME_FLORA[5].mushroom)
        );
        assert!(BIOME_FLORA[2].red_caps && BIOME_FLORA[2].deciduous);
        assert!(!BIOME_FLORA[4].deciduous && !BIOME_FLORA[7].deciduous);
    }

    /// Autumn a quarter-year after midsummer in the +Y hemisphere is spring
    /// across the equator; midwinter is bare in one and green in the other.
    #[test]
    fn the_hemispheres_have_opposite_seasons() {
        let north = season_at(0.27, 0.7);
        let south = season_at(0.27, -0.7);
        assert!(north.turn > 0.99, "{north:?}");
        assert!(south.turn < 0.01 && south.bare < 0.01, "{south:?}");
        let winter = season_at(0.5, 0.7);
        assert!(winter.bare > 0.99 && winter.straw > 0.9, "{winter:?}");
        let south_winter = season_at(0.0, -0.7);
        assert!(south_winter.bare > 0.99, "{south_winter:?}");
        // Midsummer: green everywhere.
        let summer = season_at(0.0, 0.7);
        assert_eq!((summer.turn, summer.bare, summer.straw), (0.0, 0.0, 0.0));
    }

    /// No seasons at the equator, all of them from about 27 degrees, and the
    /// year wraps without a seam.
    #[test]
    fn the_tropics_have_no_seasons_and_the_year_wraps() {
        for yf in [0.0, 0.27, 0.5, 0.75] {
            let s = season_at(yf, 0.05);
            assert_eq!((s.turn, s.bare, s.straw), (0.0, 0.0, 0.0));
        }
        let a = season_at(0.999_999, 0.8);
        let b = season_at(1.000_001, 0.8);
        assert!((a.straw - b.straw).abs() < 1e-4 && (a.turn - b.turn).abs() < 1e-4);
        let deep = season_at(0.5, 0.5);
        let half = season_at(0.5, 0.285);
        assert!(
            deep.bare > 0.99 && half.bare > 0.3 && half.bare < 0.7,
            "{half:?}"
        );
    }
}
