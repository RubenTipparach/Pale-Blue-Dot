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
/// (`planet_gen::Biome`): the foliage pass's `density` up to generator
/// version 9 ([`TreeTable::V4`]).
pub const TREE_DENSITY: [(u32, u32); 5] = [(4, 115), (5, 34), (2, 13), (1, 13), (7, 2)];

/// Where a world's trees stand (`flora-v2` design 12.1): trees per 256 cells
/// in each biome, and which material codes of the record's word a tree may
/// stand on there, as bits. Part of the generator (`TerrainConfig::trees`):
/// a tree is a thing a player works and a `tree` record names its cell, so a
/// world keeps the table it was made with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeTable {
    /// Per biome, `planet_gen::Biome` order: trees per 256 cells, 0..=255.
    pub density: [u32; 8],
    /// Per biome: bit `m` set where a tree stands on material code `m`.
    pub ground: [u32; 8],
}

/// The material codes a tree stands on in every biome: the three grasses.
const GRASS_CODES: u32 = 1 << 2 | 1 << 3 | 1 << 7;
/// The tundra's pine on its snow (code 6).
const SNOW_CODE: u32 = 1 << 6;
/// A beach's dry sand (code 1).
const SAND_CODE: u32 = 1 << 1;

impl TreeTable {
    /// Versions 4 to 9: the Tenebris scatter rule at its own rates
    /// ([`TREE_DENSITY`]): a grass of any kind, or the tundra pine on snow.
    pub const V4: Self = Self {
        density: [0, 13, 13, 0, 115, 34, 0, 2],
        ground: [
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES | SNOW_CODE,
        ],
    };

    /// Version 10 (design 12.2): M13's mean densities (its `treeP` times
    /// 256), and the beach's sand stands the acacia.
    pub const V10: Self = Self {
        density: [0, 15, 82, 0, 141, 67, 0, 77],
        ground: [
            GRASS_CODES,
            GRASS_CODES | SAND_CODE,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES,
            GRASS_CODES | SNOW_CODE,
        ],
    };

    /// Whether a finest cell stands a tree under this table: the foliage
    /// pass's rule, which reads the same numbers from the uniform
    /// ([`Self::packed`]).
    pub fn grows(&self, tree_id: u32, word: u32) -> bool {
        if (word >> 24) & 1 == 1 {
            return false;
        }
        let material = word & 0xff;
        let biome = ((word >> 8) & 0xff) as usize;
        let (Some(&density), Some(&ground)) = (self.density.get(biome), self.ground.get(biome))
        else {
            return false;
        };
        material < 32 && (ground >> material) & 1 == 1 && shader_hash(tree_id) & 0xff < density
    }

    /// One biome's entry as the shader reads it, in the top half of the
    /// uniform's `plant_means[biome].w` (the low half is the fruit bits):
    /// bits 16..24 the density, bit 24 the sand, bit 25 the snow. Only the
    /// codes these tables use are packed: grass in every biome, so it is not.
    pub fn packed(&self, biome: usize) -> u32 {
        let density = self.density.get(biome).copied().unwrap_or(0).min(255);
        let ground = self.ground.get(biome).copied().unwrap_or(0);
        density << 16
            | u32::from(ground & SAND_CODE != 0) << 24
            | u32::from(ground & SNOW_CODE != 0) << 25
    }
}

/// Whether a finest cell stands a tree (`tool-crafting` decision 3) under
/// the world's table: the foliage pass's rule (`planet_visibility.wgsl`,
/// `has_nearby_foliage`) on the CPU, so a tree the player sees is a tree the
/// player can work.
///
/// `tree_id` is the cell record's tree lane (`spare.x`); `word` its material
/// word (`metadata.y`): the material index in the low byte, the biome in the
/// next, and bit 24 set in a town's cleared footprint. The distance and
/// partition tests the shader adds decide only what is drawn this frame.
pub fn grows_tree(table: &TreeTable, tree_id: u32, word: u32) -> bool {
    table.grows(tree_id, word)
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

/// The fruit's salt in the tree draw (`planet_surface.wgsl`, `SALT_FRUIT`).
pub const SALT_FRUIT: u32 = 0x58;

/// Each fruit kind's good, by [`FRUIT_KINDS`] index (`flora-v2` design 12.5).
pub const FRUIT_GOODS: [&str; 9] = [
    "apple",
    "pear",
    "plum",
    "cherry",
    "mango",
    "papaya",
    "marula",
    "baobab",
    "pine-cone",
];

/// A `fruit` record (design 12.5): what a player took from a tree, by the
/// tree's cell key. Its fruit grows back [`FRUIT_REGROW_DAYS`] after.
pub const FRUIT_RECORD: &str = "fruit";
pub const FRUIT_SCHEMA: u32 = 1;
pub const FRUIT_REGROW_DAYS: f64 = 8.0;

/// A `fruit` record's body.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FruitPicked {
    /// The world day it was last picked.
    pub day: f64,
    /// How many it gave.
    pub took: u16,
}

impl FruitPicked {
    /// Whether the tree's fruit is still gone at `day`.
    pub fn bare_at(&self, day: f64) -> bool {
        day - self.day < FRUIT_REGROW_DAYS
    }
}

/// What a tree has hanging now (design 12.5): its fruit kind (a
/// [`FRUIT_KINDS`] index) and how many of its four hanging fruit the season
/// keeps, or `None` for a tree that bears none. The tree draw's rule
/// (`planet_surface.wgsl`, the fruit part of the near tree) on the CPU, so
/// what a player picks is what hung there.
///
/// `axis` is the tree's unit direction, `radius` the body's (the draw's
/// `params.settings.x`), `fruit_chance` the scatter's.
pub fn hanging_fruit(
    tree_id: u32,
    biome: u32,
    axis: [f32; 3],
    radius: f32,
    year_fraction: f64,
    fruit_chance: f32,
) -> Option<(usize, u32)> {
    use crate::tree_kit::roll;
    let bits = BIOME_FRUIT.get(biome.min(7) as usize).copied().unwrap_or(0) & 0xffff;
    if bits == 0 || roll(tree_id, SALT_FRUIT) >= fruit_chance {
        return None;
    }
    // A patch is a forty-metre stretch of the body: neighbours bear alike.
    let grove = axis.map(|a| (a * radius / 40.0).floor() as i32 as u32);
    let p = shader_hash(
        grove[0].wrapping_mul(73_856_093)
            ^ grove[1].wrapping_mul(19_349_663)
            ^ grove[2].wrapping_mul(83_492_791),
    );
    let count = bits.count_ones();
    let mut pick = (p.wrapping_add(u32::from(roll(tree_id, SALT_FRUIT + 1) < 0.25))) % count;
    let mut which = 0;
    for q in 0..9 {
        if (bits >> q) & 1 == 1 {
            if pick == 0 {
                which = q;
                break;
            }
            pick -= 1;
        }
    }
    let cone = which == 8;
    let season = season_at(year_fraction, axis[1]);
    let keep = if cone {
        1.0
    } else {
        (0.85 + (1.01 - 0.85) * season.turn) * (1.0 - 0.92 * season.bare)
    };
    let hanging = (0..4u32)
        .filter(|item| roll(tree_id, SALT_FRUIT + 2 + item) < keep)
        .count() as u32;
    Some((which as usize, hanging))
}

/// Where snow lies on the trees' branches in winter (`flora-v2` design
/// 12.3): from no cover at 35 degrees of latitude (its sine) to full at 50.
pub const SNOW_COLD: [f32; 2] = [0.574, 0.766];

/// How much snow lies on a tree's branches at a place, 0..1: the season's
/// `bare` (midwinter) times the cold of the latitude. Decoration on a drawn
/// model, never a cell: the ground's snow stays the generator's. The tree
/// draw's `winter_cover` is the same lines.
pub fn winter_cover(year_fraction: f64, latitude_sin: f32) -> f32 {
    season_at(year_fraction, latitude_sin).bare
        * smooth(SNOW_COLD[0], SNOW_COLD[1], latitude_sin.abs())
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
        // The shader's lines, transcribed below as `shader_rule`: each must
        // still be in both passes word for word, so the mirror is the
        // shader's own rule (`fishing-and-equipment` task 4, `flora-v2` 12.1).
        let lines = [
            "let roll = hash(cell.spare.x) & 0xffu;",
            "let material = cell.metadata.y & 0xffu;",
            "let biome = (cell.metadata.y >> 8u) & 0xffu;",
            "let entry = params.plant_means[min(biome, 7u)].w;",
            "let density = (entry >> 16u) & 0xffu;",
            "let grass = material==2u || material==3u || material==7u;",
            "let sand = ((entry >> 24u) & 1u) == 1u && material==1u;",
            "let pine = ((entry >> 25u) & 1u) == 1u && material==6u;",
            "let tree = (grass||sand||pine) && roll < density;",
            "if ((cell.metadata.y >> 24u) & 1u) == 1u { return false; }",
        ];
        for (name, shader) in [
            (
                "planet_visibility",
                include_str!("../../../assets/shaders/planet_visibility.wgsl"),
            ),
            (
                "planet_surface",
                include_str!("../../../assets/shaders/planet_surface.wgsl"),
            ),
        ] {
            for line in lines {
                assert!(shader.contains(line), "{name} no longer says `{line}`");
            }
            for literal in ["density = 115u", "density = 34u", "density = 13u"] {
                assert!(!shader.contains(literal), "{name} keeps `{literal}`");
            }
        }
        let visibility = include_str!("../../../assets/shaders/planet_visibility.wgsl");
        for line in [
            "var h = x*747796405u+2891336453u;",
            "h = ((h >> ((h >> 28u)+4u))^h)*277803737u;",
            "return (h>>22u)^h;",
        ] {
            assert!(
                visibility.contains(line),
                "the hash no longer says `{line}`"
            );
        }
        // The mirror, line for line.
        fn shader_rule(means_w: &[u32; 8], id: u32, word: u32) -> bool {
            if (word >> 24) & 1 == 1 {
                return false;
            }
            let roll = shader_hash(id) & 0xff;
            let material = word & 0xff;
            let biome = (word >> 8) & 0xff;
            let entry = means_w[biome.min(7) as usize];
            let density = (entry >> 16) & 0xff;
            let grass = material == 2 || material == 3 || material == 7;
            let sand = (entry >> 24) & 1 == 1 && material == 1;
            let pine = (entry >> 25) & 1 == 1 && material == 6;
            (grass || sand || pine) && roll < density
        }
        // The uniform's lanes as the app packs them (the fruit in the low
        // half), against the CPU rule, for every biome and material code and
        // 10,000 cell ids, on both tables.
        for table in [TreeTable::V4, TreeTable::V10] {
            let w: [u32; 8] = std::array::from_fn(|b| BIOME_FRUIT[b] | table.packed(b));
            for biome in 0..8u32 {
                for material in 0..16u32 {
                    for cleared in [0, 1 << 24] {
                        let word = material | biome << 8 | cleared;
                        for id in 0..10_000u32 {
                            assert_eq!(
                                shader_rule(&w, id, word),
                                grows_tree(&table, id, word),
                                "{table:?} id {id} word {word:#x}"
                            );
                        }
                    }
                }
            }
        }
        // Version 9's rule is today's: about 5% of a field's grass stands a
        // tree, none of its sand, and a town's footprint none.
        let field = |material: u32| material | 2 << 8;
        let v4 = TreeTable::V4;
        let trees = (0..100_000u32)
            .filter(|&id| grows_tree(&v4, id, field(2)))
            .count();
        assert!((4_500..5_700).contains(&trees), "{trees}");
        assert!(!(0..10_000u32).any(|id| grows_tree(&v4, id, field(1))));
        assert!(!(0..10_000u32).any(|id| grows_tree(&v4, id, field(2) | 1 << 24)));
    }

    /// Where trees stand under a table, folded: every biome, material code
    /// and cleared bit over 10,000 cell ids. A cell's id and word are pinned
    /// by the ground digests and the cell keys, so this pins where a world's
    /// trees stand.
    fn tree_digest(table: &TreeTable) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for biome in 0..8u32 {
            for material in 0..16u32 {
                for cleared in [0, 1 << 24] {
                    let word = material | biome << 8 | cleared;
                    for id in 0..10_000u32 {
                        let grows = u64::from(grows_tree(table, id, word));
                        hash = (hash ^ grows).wrapping_mul(0x0000_0100_0000_01b3);
                    }
                }
            }
        }
        hash
    }

    /// Versions 4 to 9 stand their trees where they always stood (taken
    /// before version 10 landed, `flora-v2` 12.1), and version 10's are
    /// pinned as it lands. Integer arithmetic only: the same on every target.
    #[test]
    fn every_version_keeps_where_its_trees_stand() {
        assert_eq!(tree_digest(&TreeTable::V4), V4_TREE_DIGEST);
        assert_eq!(tree_digest(&TreeTable::V10), V10_TREE_DIGEST);
        // The same rule as the table this build shipped before, for versions
        // 4 to 9, read off `TREE_DENSITY`.
        for (biome, density) in TREE_DENSITY {
            assert_eq!(TreeTable::V4.density[biome as usize], density);
        }
        // And cell for cell the rule as this build had it before the table,
        // transcribed: the grasses, or the tundra pine on snow.
        let before = |id: u32, word: u32| {
            if (word >> 24) & 1 == 1 {
                return false;
            }
            let roll = shader_hash(id) & 0xff;
            let material = word & 0xff;
            let biome = (word >> 8) & 0xff;
            let grass = matches!(material, 2 | 3 | 7);
            let pine = biome == 7 && material == 6;
            let density = TREE_DENSITY
                .iter()
                .find(|(b, _)| *b == biome)
                .map_or(0, |(_, d)| *d);
            (grass || pine) && roll < density
        };
        for word in (0..8u32).flat_map(|b| (0..16u32).map(move |m| m | b << 8)) {
            for id in 0..10_000u32 {
                assert_eq!(grows_tree(&TreeTable::V4, id, word), before(id, word));
            }
        }
    }

    const V4_TREE_DIGEST: u64 = 16_944_701_438_888_284_610;
    const V10_TREE_DIGEST: u64 = 10_242_190_396_241_142_731;

    /// Version 10's forests are M13's: per 256 cells of grass, about its
    /// `treeP` times 256 by biome; the beach's dry sand stands acacias.
    #[test]
    fn version_10_grows_m13_s_forests() {
        let v10 = TreeTable::V10;
        let share = |biome: u32, material: u32| {
            (0..100_000u32)
                .filter(|&id| grows_tree(&v10, id, material | biome << 8))
                .count() as f32
                / 100_000.0
                * 256.0
        };
        for (biome, material, want) in [
            (2, 2, 82.0),
            (4, 3, 141.0),
            (5, 7, 67.0),
            (7, 2, 77.0),
            (7, 6, 77.0),
            (1, 1, 15.0),
        ] {
            let got = share(biome, material);
            assert!(
                (got - want).abs() < 2.0,
                "biome {biome} material {material}: {got} of 256"
            );
        }
        assert_eq!(
            share(3, 1),
            0.0,
            "a desert's sand stands cacti, never trees"
        );
        assert_eq!(share(2, 1), 0.0, "only the beach's sand stands a tree");
    }

    /// The fruit a player picks is the fruit the tree draw hangs: the lines
    /// `hanging_fruit` mirrors are the shader's, a fields tree bears one of
    /// its four kinds, neighbours mostly alike, and winter thins the crop.
    #[test]
    fn the_fruit_to_pick_is_the_fruit_the_draw_hangs() {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        for line in [
            "const SALT_FRUIT: u32 = 0x58u;",
            "let bits = params.plant_means[b].w & 0xffffu;",
            "if bits != 0u && anchors > 0u && roll(id,SALT_FRUIT) < params.flora_fall.w {",
            "let grove = vec3<i32>(floor(axis*params.settings.x/40.0));",
            "let p = hash(u32(grove.x)*73856093u ^ u32(grove.y)*19349663u ^ u32(grove.z)*83492791u);",
            "var pick = (p + select(0u, 1u, roll(id,SALT_FRUIT+1u) < 0.25)) % count;",
            "let keep = select(mix(0.85, 1.01, season.x)*(1.0-0.92*season.y), 1.0, cone);",
            "if roll(id,SALT_FRUIT+2u+item) < keep && !(f < 48u && tree_picked(id)) {",
        ] {
            assert!(
                shader.contains(line),
                "the tree draw no longer says `{line}`"
            );
        }
        let axis = {
            let v = [0.6f32, 0.6, 0.53];
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v.map(|x| x / l)
        };
        let (mut bearing, mut kinds) = (0, std::collections::BTreeSet::new());
        for id in 0..2_000u32 {
            if let Some((kind, n)) = hanging_fruit(id, 2, axis, 4_800.0, 0.27, 0.5) {
                bearing += 1;
                assert!(kind < 4, "a fields tree bears apple, pear, plum or cherry");
                assert!(n <= 4);
                kinds.insert(kind);
            }
        }
        assert!(
            (800..1_200).contains(&bearing),
            "about half bear: {bearing}"
        );
        assert!(kinds.len() <= 2, "one patch, one kind or two: {kinds:?}");
        assert_eq!(
            hanging_fruit(7, 5, axis, 4_800.0, 0.27, 1.0),
            None,
            "the swamp bears none"
        );
        let summer: u32 = (0..2_000u32)
            .filter_map(|id| hanging_fruit(id, 2, axis, 4_800.0, 0.0, 1.0))
            .map(|(_, n)| n)
            .sum();
        let winter: u32 = (0..2_000u32)
            .filter_map(|id| hanging_fruit(id, 2, axis, 4_800.0, 0.5, 1.0))
            .map(|(_, n)| n)
            .sum();
        assert!(
            winter * 4 < summer,
            "winter {winter} against summer {summer}"
        );
        let picked = FruitPicked { day: 10.0, took: 3 };
        assert!(picked.bare_at(17.9) && !picked.bare_at(18.0));
    }

    /// Snow lies on the branches only in a cold winter: none in summer, none
    /// at 30 degrees, full at midwinter past 50; and the tree draw's
    /// `winter_cover` reads the same numbers (design 12.3).
    #[test]
    fn snow_lies_on_branches_only_in_a_cold_winter() {
        let north_50 = 0.80;
        let midwinter = 0.5;
        assert!(winter_cover(midwinter, north_50) > 0.9);
        assert!(winter_cover(0.0, north_50) < 0.01, "midsummer");
        assert_eq!(
            winter_cover(midwinter, 0.5),
            0.0,
            "30 degrees is never snowy"
        );
        assert!(
            winter_cover(0.0, -north_50) > 0.9,
            "the south's winter is the north's summer"
        );
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        assert!(shader.contains(&format!(
            "const SNOW_COLD = vec2<f32>({:?}, {:?});",
            SNOW_COLD[0], SNOW_COLD[1]
        )));
        assert!(shader.contains(
            "return season_at(radial).y*smoothstep(SNOW_COLD.x, SNOW_COLD.y, abs(radial.y));"
        ));
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
