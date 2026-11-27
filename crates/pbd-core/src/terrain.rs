//! Versioned deterministic CPU terrain reference for one planar hex patch.
//! This is a baseline sampler, not the eventual spherical multibiome generator.
//! Integer-coordinate hashes make chunk order irrelevant. Never change this
//! algorithm for an existing save without bumping GENERATOR_VERSION.

use crate::hex::{Hex, Voxel};

/// Version 2 is the spherical generator in [`crate::planet_gen`]; version 3
/// splits its fields into planet-scale and land-scale, which moves every
/// height on the body; version 5 (`bigger-biomes`) widens the moisture field
/// fourfold and moves its thresholds, which changes which biome the dry land
/// is and nothing about its shape; version 6 (`taller-mountains`) raises
/// mountain ranges to about 290 m in regions of their own and caps them with
/// snow above 200 m; version 7 (`ocean-environments` B10) adds the sea's
/// places and carves its trenches; version 8 (design 13) runs rivers to
/// the sea; version 9 (design B15) puts back the reefs version 8 lost;
/// version 10 (`flora-v2` design 12) grows M13's forests on version 9's
/// ground; version 11 (`river-downhill`) keeps every river's water from
/// rising downstream; version 12 (`swamps`) puts swamps on the low wet land
/// by the sea and the rivers, on version 11's heights; version 14
/// (`ocean-environments` B24) lays the sea's cells; version 15
/// (`mountain-ground`) grasses and snows the mountains' flats, scours their
/// crests and stands their trees to a tree line (13 was never shipped);
/// version 16 (`ocean-environments` B30) grows the reefs to about a tenth of
/// the sea. A new world is made
/// with this version;
/// a saved one keeps the version it was made with
/// ([`crate::planet_gen::TerrainConfig::for_version`]).
pub const GENERATOR_VERSION: u32 = 16;

/// The topology a world's cells are cut from: the Goldberg levels and the
/// finest cell keys (`pbd_core::cell_key`). A save records it beside the
/// generator (`world-persistence` decision 3); it moves only when the cells
/// themselves do.
pub const TOPOLOGY_VERSION: u32 = 1;

/// The planar sampler's hash salt: the generator version it was written
/// under. It is its own constant so that a spherical generator version which
/// changes nothing here (`bigger-biomes` 5 changes only the moisture) does
/// not move every voxel of the planar reference (`bigger-biomes` decision 6).
const PLANAR_SALT: u64 = 4;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Soil = 2,
    Grass = 3,
    Water = 4,
    Ore = 5,
    /// The top blocks the spherical generator hands out beyond the planar
    /// sampler's own: a beach or a desert, a yellow-green pasture, the deep
    /// green of a jungle, a snowfield, a rocky outcrop, bare dirt.
    Sand = 6,
    DryGrass = 7,
    JungleGrass = 8,
    Snow = 9,
    Rock = 10,
    Dirt = 11,
    /// A lamp on the ground. A MATERIAL rather than an entity, so it flows
    /// through every piece of machinery a block already has: the hotbar holds
    /// it, the aim ray targets it, the edit path places and removes it, the
    /// save records it and the tier relights around it. Nothing new had to
    /// learn about torches.
    ///
    /// It is neither solid nor opaque - a player walks through one and light
    /// goes past it - which is what stops a torch shadowing itself and what
    /// stops a corridor of them being a wall.
    Torch = 12,
    /// The rest of a city's lights (`lamps-and-lanterns`), each a material for
    /// the torch's reasons and, like it, neither solid nor opaque.
    ///
    /// A lantern on a post, standing on the floor: a street lamp. Lit from
    /// dusk to dawn.
    LanternPost = 13,
    /// A lantern on a bracket, on the side of its cell that has a wall. Lit
    /// from dusk to dawn.
    LanternWall = 14,
    /// A lantern hanging from the cell above. Always lit.
    LanternHanging = 15,
    /// A fire in an iron bowl: the brightest light there is. Always lit.
    Brazier = 16,
    /// A candle: enough to light a room, and no more. Always lit.
    Candle = 17,
    /// Tilled soil (`farming-and-smelting` decision 2): a grass, dirt, soil or
    /// sand top the shovel has turned, which a seed is planted on. Solid,
    /// drawn with the fields sheet's own tilled-soil tile.
    Tilled = 18,
    /// The forge (`farming-and-smelting` decision 8): the first station
    /// block. Solid, drawn in slate bricks, with a chimney stood on it.
    Forge = 19,
    /// The workbench (`tool-crafting` decision 5): the station tools are made
    /// at. Solid, drawn in the local trees' wood with a plank top stood on it.
    Workbench = 20,
    // 20 is `tool-crafting`'s; 22 and 23 are held for the kitchen.
    /// A campfire (`cooking` design S2): a ring of stones round a fire, the
    /// kitchen's first station and a light. Like every light it is neither
    /// solid nor opaque, and the column pass draws it as a lamp.
    Campfire = 21,
    // 22 and 23 are held for the kitchen.
    /// The sea's cells (`ocean-environments` design B24), made by generator
    /// version 14 and dug like any ground. Each is drawn as an existing
    /// render code from a sheet of its own, as ore is, so the column's
    /// four-bit material word does not widen. Reef rock: a reef's limestone,
    /// the ocean sheet's (3,0). Solid.
    ReefRock = 24,
    /// Sea ice: a floe's cell, standing on polar water; the frozen wastes'
    /// layered glacial ice. Solid.
    SeaIce = 25,
    /// Basalt: the ground round a vent; the basalt wastes' own. Solid.
    Basalt = 26,
    /// A mussel bed: the shells packed on a vent's basalt; the ocean sheet's
    /// shell bed (3,1). Solid.
    MusselBed = 27,
    /// Sticks: a beaver's dam and lodge (`ocean-environments` design B25),
    /// laid by the `beaver` world process; the local trees' bark, as the
    /// workbench is drawn. Solid, so flowing water treats a dam as a wall.
    Sticks = 28,
    /// Desert sand (`sand-tiles` design 5 (c)): what generated sand dug in a
    /// desert cell yields. Generation never writes it, so no generator
    /// version moves; a desert's ground is still [`Material::Sand`], drawn
    /// from the desert sheet. Placed, it is drawn from the desert sheet
    /// wherever it lands, as render code 1 (sand). It acts as sand in every
    /// other way ([`Material::is_sand`]).
    DesertSand = 29,
    /// The grinder (`townsfolk` design 41): the station that crushes stone,
    /// a hopper over two stone rollers in a timber frame. Solid, drawn in the
    /// local trees' wood as the workbench is, with its frame stood on it.
    Grinder = 30,
    /// The mixer (design 41): the station that mixes crushed stone and sand
    /// into gravel, a drum on a timber frame. Solid, drawn as the grinder.
    Mixer = 31,
    /// The carpenter's bench (`furniture` design 1): the station every piece
    /// of furniture is made at. Solid, drawn in the workbench's wood with a
    /// saw and a square on its top. Code 40: 30 to 39 are the paving's
    /// stations (`townsfolk` design 41).
    Carpenter = 40,
    /// The drafting table (`building-planner` design 3): where a plan is
    /// drawn. Solid, drawn in the workbench's wood with a sloped board on
    /// its top. It queues no jobs: F on it opens the planner.
    DraftingTable = 41,
    /// A chest (`storage-chests` design 1): a wooden box of twenty slots,
    /// made at the workbench and set down like a station. Solid, drawn in
    /// the local trees' wood as the workbench is, with an iron hasp stood on
    /// it. What it holds is a record of its own (`pbd_core::chest`). Code 42:
    /// 30 to 39 are the paving's stations, 40 the carpenter's bench, 41 the drafting table.
    Chest = 42,
    /// The tanning rack (`survival-mode` design 17): the station a hide is
    /// tanned into leather at, a hide laced in a timber frame. Solid, drawn
    /// in the local trees' wood with the frame and the hide stood on it.
    /// Code 48: 48 to 51 are Survival's stations, clear of the blocks'
    /// run other branches append to.
    TanningRack = 48,
    /// The still (`survival-mode` design 15 and 18): a stone pot with a lid
    /// and a cane spout that turns sea water into fresh beside a lit
    /// campfire. Solid, drawn as its stone pot and spout stood on its cell.
    Still = 49,
    /// Flowing water (`water-flow`): Luanti's flowing liquid on the hex grid,
    /// at a level from 7 (beside its source) down to 0, a thin film. A
    /// `Fall` is the same with Luanti's "flowing down" set: the cell below is
    /// air or flowing, so it offers nothing sideways. `Water` is the source.
    /// Neither solid nor opaque, swum like the source. Codes 64 to 79, far
    /// from the blocks' run, so a branch adding blocks never meets them.
    Flow0 = 64,
    Flow1 = 65,
    Flow2 = 66,
    Flow3 = 67,
    Flow4 = 68,
    Flow5 = 69,
    Flow6 = 70,
    Flow7 = 71,
    Fall0 = 72,
    Fall1 = 73,
    Fall2 = 74,
    Fall3 = 75,
    Fall4 = 76,
    Fall5 = 77,
    Fall6 = 78,
    Fall7 = 79,
}

/// Every flowing material as one pattern, for an exhaustive match.
#[macro_export]
macro_rules! flowing_water {
    () => {
        $crate::terrain::Material::Flow0
            | $crate::terrain::Material::Flow1
            | $crate::terrain::Material::Flow2
            | $crate::terrain::Material::Flow3
            | $crate::terrain::Material::Flow4
            | $crate::terrain::Material::Flow5
            | $crate::terrain::Material::Flow6
            | $crate::terrain::Material::Flow7
            | $crate::terrain::Material::Fall0
            | $crate::terrain::Material::Fall1
            | $crate::terrain::Material::Fall2
            | $crate::terrain::Material::Fall3
            | $crate::terrain::Material::Fall4
            | $crate::terrain::Material::Fall5
            | $crate::terrain::Material::Fall6
            | $crate::terrain::Material::Fall7
    };
}

/// The flowing materials by level, still then falling: `FLOWING[level]` and
/// `FLOWING[8 + level]`.
pub const FLOWING: [Material; 16] = [
    Material::Flow0,
    Material::Flow1,
    Material::Flow2,
    Material::Flow3,
    Material::Flow4,
    Material::Flow5,
    Material::Flow6,
    Material::Flow7,
    Material::Fall0,
    Material::Fall1,
    Material::Fall2,
    Material::Fall3,
    Material::Fall4,
    Material::Fall5,
    Material::Fall6,
    Material::Fall7,
];

impl Material {
    /// How much light this gives out, 0 for most things.
    ///
    /// On the MATERIAL because that is what a cell holds: a lamp that carried
    /// its brightness somewhere else would be a second place to look, and the
    /// tier's emitter list is derived from the columns rather than kept beside
    /// them.
    pub fn emission(self) -> u8 {
        match self {
            // Tenebris's `voxel_torch_emission`, which it ships at 14 and never
            // reads. One below full, so a torch is plainly a lamp and plainly
            // not the sun.
            Material::Torch => 14,
            // The design's levels (`lamps-and-lanterns` decision 3), on the
            // field's 0-15 scale where a level is how many cells the light
            // survives: a street lantern reaches the next one, a brazier
            // fills a square, a candle lights a room of about two cells.
            Material::LanternPost | Material::LanternWall => 13,
            Material::LanternHanging => 12,
            Material::Brazier => 15,
            Material::Candle => 8,
            // A cook fire: a street lantern's reach, below the brazier.
            Material::Campfire => 13,
            _ => 0,
        }
    }

    /// Whether this is a light: something that gives out light and is walked
    /// through rather than stood on.
    pub fn is_lamp(self) -> bool {
        self.emission() > 0
    }

    /// Flowing water at `level` (0 to 7), falling or not.
    pub fn flowing(level: u8, falling: bool) -> Material {
        FLOWING[usize::from(level.min(7)) + if falling { 8 } else { 0 }]
    }

    /// A flowing cell's level and whether it is falling, or `None` for
    /// anything else, the source included.
    pub fn flow(self) -> Option<(u8, bool)> {
        let code = self as u16;
        (64..80)
            .contains(&code)
            .then(|| ((code - 64) as u8 % 8, code >= 72))
    }

    /// Whether this is water, a source or flowing: what is swum, what the
    /// renderer's water code means and what a cell under it is wet in.
    pub fn is_water(self) -> bool {
        self == Material::Water || self.flow().is_some()
    }

    /// What this top is DRAWN as with water standing on it (`flora-v2`
    /// 12.10): a grass of any kind or snow is a bed of dirt, and nothing
    /// grows there; everything else is itself. A draw-time rule only: the
    /// column keeps its material, and no generator version or save moves.
    pub fn drowned(self) -> Material {
        match self {
            Material::Grass | Material::DryGrass | Material::JungleGrass | Material::Snow => {
                Material::Dirt
            }
            other => other,
        }
    }

    /// Whether this is sand of either kind, beach or desert (`sand-tiles`
    /// design 5): the two dig, sound, bounce, till and turn up bait alike,
    /// and differ only in the sheet they are drawn from.
    pub fn is_sand(self) -> bool {
        matches!(self, Material::Sand | Material::DesertSand)
    }

    /// Whether this light burns only from dusk to dawn. A property of the
    /// kind of light, not of where it stands (`lamps-and-lanterns` decision
    /// 4): a street lantern the player places behaves like a city's.
    pub fn dusk_lit(self) -> bool {
        matches!(self, Material::LanternPost | Material::LanternWall)
    }

    /// Every material, in code order. Codes need not be contiguous: two
    /// branches each take their own numbers, and a gap is a code another
    /// build holds.
    pub const ALL: [Material; 35] = [
        Material::Air,
        Material::Stone,
        Material::Soil,
        Material::Grass,
        Material::Water,
        Material::Ore,
        Material::Sand,
        Material::DryGrass,
        Material::JungleGrass,
        Material::Snow,
        Material::Rock,
        Material::Dirt,
        Material::Torch,
        Material::LanternPost,
        Material::LanternWall,
        Material::LanternHanging,
        Material::Brazier,
        Material::Candle,
        Material::Tilled,
        Material::Forge,
        Material::Workbench,
        Material::Campfire,
        Material::ReefRock,
        Material::SeaIce,
        Material::Basalt,
        Material::MusselBed,
        Material::Sticks,
        Material::DesertSand,
        Material::Grinder,
        Material::Mixer,
        Material::Carpenter,
        Material::DraftingTable,
        Material::Chest,
        Material::TanningRack,
        Material::Still,
    ];

    /// Every light, in material order. The column pass's lamp kind is the
    /// index here, in three bits: one more fits.
    pub const LAMPS: [Material; 7] = [
        Material::Torch,
        Material::LanternPost,
        Material::LanternWall,
        Material::LanternHanging,
        Material::Brazier,
        Material::Candle,
        Material::Campfire,
    ];
}

#[derive(Clone, Copy, Debug)]
pub struct TerrainGenerator {
    pub seed: u64,
    pub sea_level: i32,
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl TerrainGenerator {
    fn hash(self, q: i32, r: i32, layer: i32, channel: u64) -> u64 {
        let mut value = mix(self.seed ^ channel ^ PLANAR_SALT);
        for coordinate in [q, r, layer] {
            value = mix(value ^ (coordinate as i64 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        }
        value
    }

    fn lattice(self, q: i32, r: i32, scale: i32) -> f64 {
        let a = q.div_euclid(scale);
        let b = r.div_euclid(scale);
        let u = q.rem_euclid(scale) as f64 / scale as f64;
        let v = r.rem_euclid(scale) as f64 / scale as f64;
        let u = u * u * (3.0 - 2.0 * u);
        let v = v * v * (3.0 - 2.0 * v);
        let sample = |q, r| {
            ((self.hash(q, r, 0, scale as u64) >> 40) as f64 / ((1_u64 << 24) - 1) as f64) * 2.0
                - 1.0
        };
        let lower = sample(a, b) * (1.0 - u) + sample(a + 1, b) * u;
        let upper = sample(a, b + 1) * (1.0 - u) + sample(a + 1, b + 1) * u;
        lower * (1.0 - v) + upper * v
    }

    pub fn height(self, hex: Hex) -> i32 {
        (self.lattice(hex.q, hex.r, 64) * 26.0 + self.lattice(hex.q, hex.r, 16) * 7.0 + 8.0).floor()
            as i32
    }

    pub fn sample(self, voxel: Voxel) -> Material {
        let height = self.height(voxel.hex);
        if voxel.layer > height {
            if voxel.layer <= self.sea_level {
                Material::Water
            } else {
                Material::Air
            }
        } else if voxel.layer == height && height > self.sea_level {
            Material::Grass
        } else if voxel.layer >= height - 3 {
            Material::Soil
        } else if self
            .hash(voxel.hex.q, voxel.hex.r, voxel.layer, 0x0a_e1)
            .is_multiple_of(127)
        {
            Material::Ore
        } else {
            Material::Stone
        }
    }
}

/// The material a saved code names, and the code it is saved as: the save's
/// journal, a region's snapshot and its digest (`multiplayer` 3.1) all
/// write a material by this code.
///
/// Written out both ways rather than transmuted: a code nothing names must not
/// become a material, and a material added tomorrow must not silently take
/// another's number.
pub fn material_code(material: Material) -> u8 {
    match material {
        Material::Air => 0,
        Material::Stone => 1,
        Material::Soil => 2,
        Material::Grass => 3,
        Material::Water => 4,
        Material::Ore => 5,
        Material::Sand => 6,
        Material::DryGrass => 7,
        Material::JungleGrass => 8,
        Material::Snow => 9,
        Material::Rock => 10,
        Material::Dirt => 11,
        Material::Torch => 12,
        Material::LanternPost => 13,
        Material::LanternWall => 14,
        Material::LanternHanging => 15,
        Material::Brazier => 16,
        Material::Candle => 17,
        Material::Tilled => 18,
        Material::Forge => 19,
        Material::Workbench => 20,
        Material::Campfire => 21,
        // The sea's cells (`ocean-environments` B24); 22 and 23 are held for
        // the kitchen. An older build reads none of them, as it reads no
        // code it does not know.
        Material::ReefRock => 24,
        Material::SeaIce => 25,
        Material::Basalt => 26,
        Material::MusselBed => 27,
        Material::Sticks => 28,
        // Desert sand (`sand-tiles` design 5): the next free code.
        Material::DesertSand => 29,
        // `townsfolk` design 41: the gravel stations.
        Material::Grinder => 30,
        Material::Mixer => 31,
        Material::Carpenter => 40,
        Material::DraftingTable => 41,
        // A chest (`storage-chests` design 1): the next free code.
        Material::Chest => 42,
        // `survival-mode` design 17 and 18: Survival's stations, 48 on.
        Material::TanningRack => 48,
        Material::Still => 49,
        // Flowing water (`water-flow` E4): 64 + level, 72 + level falling,
        // the material's own code, far from the blocks' run.
        flowing @ crate::flowing_water!() => flowing as u16 as u8,
    }
}

pub fn material_of(code: u8) -> Option<Material> {
    Some(match code {
        0 => Material::Air,
        1 => Material::Stone,
        2 => Material::Soil,
        3 => Material::Grass,
        4 => Material::Water,
        5 => Material::Ore,
        6 => Material::Sand,
        7 => Material::DryGrass,
        8 => Material::JungleGrass,
        9 => Material::Snow,
        10 => Material::Rock,
        11 => Material::Dirt,
        12 => Material::Torch,
        13 => Material::LanternPost,
        14 => Material::LanternWall,
        15 => Material::LanternHanging,
        16 => Material::Brazier,
        17 => Material::Candle,
        18 => Material::Tilled,
        19 => Material::Forge,
        20 => Material::Workbench,
        21 => Material::Campfire,
        24 => Material::ReefRock,
        25 => Material::SeaIce,
        26 => Material::Basalt,
        27 => Material::MusselBed,
        28 => Material::Sticks,
        29 => Material::DesertSand,
        30 => Material::Grinder,
        31 => Material::Mixer,
        40 => Material::Carpenter,
        41 => Material::DraftingTable,
        42 => Material::Chest,
        48 => Material::TanningRack,
        49 => Material::Still,
        64..=79 => Material::flowing((code - 64) % 8, code >= 72),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The planar reference makes the voxels it made when its salt was the
    /// generator version (4): the digest was taken then. The spherical
    /// generator's version can move without moving it (`bigger-biomes`
    /// decision 6).
    #[test]
    fn the_planar_sampler_is_fixed_whatever_the_generator_version() {
        let generator = TerrainGenerator {
            seed: 42,
            sea_level: 0,
        };
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for q in -40..40 {
            for r in -40..40 {
                for layer in -8..40 {
                    let voxel = Voxel {
                        hex: Hex { q, r },
                        layer,
                    };
                    hash =
                        (hash ^ generator.sample(voxel) as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        assert_eq!(hash, PLANAR_DIGEST);
    }

    const PLANAR_DIGEST: u64 = 17_835_469_625_199_367_657;

    /// `ALL` is every material once, in its discriminant's order: the
    /// first twenty with no gap, and after them only rising codes.
    #[test]
    fn all_lists_every_material_in_order() {
        for (index, material) in Material::ALL.iter().enumerate().take(20) {
            assert_eq!(*material as usize, index);
        }
        for pair in Material::ALL.windows(2) {
            assert!((pair[0] as u16) < (pair[1] as u16), "{pair:?}");
        }
    }

    #[test]
    fn terrain_is_independent_of_visit_order_and_changes_with_seed() {
        let generator = TerrainGenerator {
            seed: 42,
            sea_level: 0,
        };
        let voxels: Vec<_> = (-100..100)
            .map(|q| Voxel {
                hex: Hex { q, r: -q / 3 },
                layer: 5,
            })
            .collect();
        let forwards: Vec<_> = voxels
            .iter()
            .map(|&voxel| generator.sample(voxel))
            .collect();
        let mut backwards: Vec<_> = voxels
            .iter()
            .rev()
            .map(|&voxel| generator.sample(voxel))
            .collect();
        backwards.reverse();
        assert_eq!(forwards, backwards);
        let different = TerrainGenerator {
            seed: 43,
            ..generator
        };
        assert!(
            voxels
                .iter()
                .any(|&voxel| generator.sample(voxel) != different.sample(voxel))
        );
    }

    #[test]
    fn chunk_boundaries_reconstruct_the_same_sample_and_water_is_bounded() {
        let generator = TerrainGenerator {
            seed: 1234,
            sea_level: 0,
        };
        for q in -65..65 {
            for layer in [-64, -33, -1, 0, 1, 31, 32, 64] {
                let voxel = Voxel {
                    hex: Hex { q, r: -q },
                    layer,
                };
                let (chunk, local) = voxel.split();
                let material = generator.sample(voxel);
                assert_eq!(material, generator.sample(Voxel::join(chunk, local)));
                if material == Material::Water {
                    assert!(layer <= generator.sea_level);
                }
                if layer == 64 {
                    assert_eq!(material, Material::Air);
                }
            }
        }
    }

    #[test]
    fn adjacent_columns_remain_continuous_across_negative_chunk_edges() {
        let generator = TerrainGenerator {
            seed: 9,
            sea_level: 0,
        };
        for q in -200..200 {
            let height = generator.height(Hex { q, r: -32 });
            let neighbor = generator.height(Hex { q: q + 1, r: -32 });
            assert!((height - neighbor).abs() <= 3);
        }
    }
}
