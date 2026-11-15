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
/// `planet_gen::Biome` order (`flora-v2` design 1, 9.1). Read off the ground
/// the tufts stand on, as the approved mockup does (`foliage.html`,
/// `groundRamp`): the biome sheet's ground tile (0,0) in
/// `assets/tilesets/atlas.png`, its texels sorted by luminance and cut in
/// thirds, each third's commonest colour. `planet_terrain`'s atlas test
/// recomputes this table off the committed file and fails on any difference.
pub const TUFT_RAMPS: [[u32; 3]; 8] = [
    [0xb8cba8, 0xcbd6b0, 0xcfdab3], // ocean: seabed sand
    [0xe0c996, 0xe2cb98, 0xe3cc98], // beach: dry shell sand
    [0x617741, 0x5f7841, 0xb8c25f], // fields: pasture grass
    [0xeac385, 0xebc586, 0xecc587], // desert: pale sand
    [0x164a35, 0x194a35, 0x2f673a], // jungle: jungle grass
    [0x2b3831, 0x475230, 0x5e6634], // swamp: swamp moss
    [0x374051, 0x374352, 0x808a94], // mountains: scree
    [0x536155, 0x637261, 0xe9eeec], // tundra: snowy moss
];

/// The mean colour of the three tiles the plants wear, per biome sheet, in
/// `planet_gen::Biome` order (`flora-v2` design 9.2): leaves (0,2), bark
/// (2,1) and stone (3,0). Each is the tile's mean LINEAR colour (the atlas is
/// sRGB, so the mean is taken after decoding), written as `0xRRGGBB` through
/// the shader's own `pow(1/2.2)`, so `rgb8` reads it back as that mean. A
/// plant is drawn part way from its texel toward this, the mockup's "one art
/// style". The same atlas test holds the table to the file.
pub const PLANT_MEANS: [[u32; 3]; 8] = [
    [0x87a78d, 0x48776a, 0x7ca499], // ocean
    [0xb3a378, 0x826e5d, 0xbc9f73], // beach
    [0x5f703c, 0x6c553e, 0xa6a5a4], // fields
    [0xb17b4e, 0x5d6c43, 0xb46546], // desert
    [0x33643d, 0x58492e, 0x2f3e3e], // jungle
    [0x565f3c, 0x5c5643, 0x5e654f], // swamp
    [0x7d8381, 0x807879, 0x596472], // mountains
    [0x687767, 0x73624e, 0x707d7b], // tundra
];

/// How much of a texel's own contrast a plant keeps (`flora-v2` design 9.2,
/// the mockup's `detail`): leaves and bushes, and the solids (trunks, twigs,
/// pebbles). `planet_surface.wgsl` carries the same pair.
pub const LEAF_DETAIL: f32 = 0.45;
pub const SOLID_DETAIL: f32 = 0.7;

/// The fruit that hang from the trees (`flora-v2` design 9.6), in the order
/// their bits in [`BIOME_FRUIT`] and the shader's `fruit_ramp` count them.
pub const FRUIT_KINDS: [&str; 9] = [
    "apple", "pear", "plum", "cherry", "mango", "papaya", "marula", "baobab", "pinecone",
];
/// Each fruit's three tones, dark to light, `0xRRGGBB`: the mockup's palette
/// (`docs/mockups/foliage/fruit_art.py`, `FRUIT`). The shader's `fruit_ramp`
/// is the same table, which a test holds.
pub const FRUIT_RAMPS: [[u32; 3]; 9] = [
    [0xa61e1f, 0xd12d41, 0xfa3a64], // apple
    [0xc56d25, 0xfaaf5c, 0xeab798], // pear
    [0x4f1d83, 0x7d2dbd, 0x9e3ba2], // plum
    [0x841c1d, 0xd12d41, 0xfa3a64], // cherry
    [0xd4462b, 0xfb902f, 0xfaaf5c], // mango
    [0xc56d25, 0xfe6420, 0xfb902f], // papaya
    [0xc56d25, 0xfaaf5c, 0xeab798], // marula
    [0xa09f91, 0xcfaa81, 0xeab798], // baobab
    [0x522b2f, 0x7d4636, 0xab8465], // pine cone
];
/// Which fruit a biome's trees bear, as bits of [`FRUIT_KINDS`], in
/// `planet_gen::Biome` order: the mockup's `FRUIT` table on the game's biomes
/// (recommendation taken, ask only with screenshots). The beach is the
/// savanna's stand-in, as for its tuft ramp; the swamp's fruit are forage
/// plants in the mockup, and phase 1's wild plants are the game's.
pub const BIOME_FRUIT: [u32; 8] = [
    0,               // ocean
    1 << 6 | 1 << 7, // beach: marula, baobab
    0b1111,          // fields: apple, pear, plum, cherry
    0,               // desert: no trees
    1 << 4 | 1 << 5, // jungle: mango, papaya
    0,               // swamp
    0,               // mountains: no trees
    1 << 8,          // tundra: pine cones
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
/// Fallen leaves on the ground (`flora-v2` design 9.4): they land from the
/// start of the turn to its peak (the first pair), lie through the bare
/// months, and are gone before spring (the second pair). `SEASON_LITTER`.
pub const SEASON_LITTER: [f32; 4] = [0.15, 0.27, 0.55, 0.68];
/// How much of the autumn fall still lies at midwinter: the mockup's winter
/// is 0.6 of its autumn. `LITTER_WINTER` in the shader.
pub const LITTER_WINTER: f32 = 0.6;

/// How far into each of the plants' seasonal looks a place is, each 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Season {
    /// Deciduous leaves turned gold and red.
    pub turn: f32,
    /// Deciduous leaves fallen.
    pub bare: f32,
    /// Grass dried to straw.
    pub straw: f32,
    /// Fallen leaves on the ground under deciduous trees.
    pub litter: f32,
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
        litter: strength * litter(local),
    }
}

/// How much of the year's fall lies on the ground at `local` (0 a place's
/// midsummer), 0..1: landing through autumn, thinned to [`LITTER_WINTER`] by
/// midwinter, gone before spring.
fn litter(local: f32) -> f32 {
    let [a, b, c, d] = SEASON_LITTER;
    let fall = smooth(a, b, local) * (1.0 - smooth(c, d, local));
    fall * (1.0 - (1.0 - LITTER_WINTER) * bump(local, SEASON_BARE))
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
        // The ramp is sorted by the mockup's own Rec. 709 luminance; the
        // table's colours are pinned to the atlas in `planet_terrain`.
        let luma = |c: u32| {
            let (r, g, b) = ((c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff);
            0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
        };
        assert_eq!(TUFT_RAMPS.len(), 8);
        for ramp in TUFT_RAMPS {
            assert!(ramp.iter().all(|c| *c <= 0xff_ffff));
            assert!(
                luma(ramp[0]) <= luma(ramp[1]) && luma(ramp[1]) <= luma(ramp[2]),
                "{ramp:x?}"
            );
        }
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
        assert_eq!(summer.litter, 0.0);
    }

    /// The leaves land through autumn, lie thinner at midwinter, and are
    /// gone by spring; none in the tropics.
    #[test]
    fn the_leaves_fall_in_autumn_and_are_gone_by_spring() {
        let at = |yf: f64| season_at(yf, 0.7).litter;
        assert!(at(0.10) < 0.01, "summer");
        assert!(at(0.27) > 0.99, "autumn's peak");
        assert!(at(0.20) > 0.1 && at(0.20) < at(0.27), "landing");
        assert!((at(0.5) - LITTER_WINTER).abs() < 0.02, "midwinter");
        assert!(at(0.72) < 0.01 && at(0.85) < 0.01, "spring");
        assert_eq!(season_at(0.27, 0.05).litter, 0.0);
        // The southern hemisphere's autumn is half a year on.
        assert!(season_at(0.77, -0.7).litter > 0.99);
    }

    /// Every fruit has a ramp and every bit a biome names is a fruit; the
    /// trees that bear none are the ones the mockup gives none.
    #[test]
    fn the_fruit_table_is_whole() {
        assert_eq!(FRUIT_KINDS.len(), FRUIT_RAMPS.len());
        for bits in BIOME_FRUIT {
            assert!(bits < 1 << FRUIT_KINDS.len());
        }
        assert_eq!(BIOME_FRUIT[2].count_ones(), 4);
        assert_eq!(BIOME_FRUIT[7], 1 << 8);
        assert_eq!(BIOME_FRUIT[5], 0);
        assert!(FRUIT_RAMPS.iter().flatten().all(|c| *c <= 0xff_ffff));
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
