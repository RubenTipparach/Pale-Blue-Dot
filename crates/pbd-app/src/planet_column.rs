//! The column tier: voxel columns for the cells a player could be standing
//! inside of.
//!
//! The heightfield tiers answer one surface per direction, so a cave, an
//! overhang and a block to remove are all inexpressible in them.
//! `pbd_core::column` answers a material per metre instead, and this module is
//! where that answer becomes something the frame can draw.
//!
//! **Why a sub-band rather than the whole finest band.** One column is 22.3 us
//! to build (`pbd_core::column`'s `column_cost` report), so the level-11 band's
//! 40,670 cells would be 0.91 s of generation on every rebuild. What a column
//! buys is a cave a player can be inside, and most of that band is rock nobody
//! is standing in, so the tier is `COLUMN_M` of great circle around the same
//! anchor the fine set uses.
//!
//! At the tier's edge a cell has no column and the heightfield answer stands,
//! which leaves no hole: a cell whose neighbour has no column treats that
//! neighbour as solid below its cap, and that is precisely what the heightfield
//! already assumes.

use super::GpuCell;
use super::terrain::{PLANET_RADIUS, render_code, terrain_config};
use crate::config::ColumnSettings;
use bevy::prelude::Vec3;
use bytemuck::{Pod, Zeroable};
use pbd_core::column::{self, Column, LAYERS, MAX_RUNS};
use pbd_core::edits::Edits;
use pbd_core::flora;
use pbd_core::light;
use pbd_core::terrain::Material;
use pbd_core::worms;

/// Slots in the column buffer. The default tier is about 3,100 cells; the
/// headroom is for a configured reach larger than that, and a reach that
/// overruns it is truncated rather than silently wrapping.
///
/// Sixteen bits less one, because a slot plus one rides the high half of the
/// cell record's skylight word.
pub const COLUMN_CAPACITY: u32 = 16_384;

/// Where a column record carries its lamp's layer, plus one: the high half of
/// `more[3]`, whose low bit is the rim flag. Nine bits, as the water's.
pub const TORCH_SHIFT: u32 = 16;
/// The lamp layer's width in the word.
pub const LAMP_LAYER_MASK: u32 = 0x1ff;
/// Where the record carries WHICH lamp it is, as its index in
/// `Material::LAMPS`: three bits above the layer. The torch is nought, so a
/// record packed before there were other lamps reads as a torch.
pub const LAMP_KIND_SHIFT: u32 = 25;
const _: () = assert!(TORCH_SHIFT + 9 <= LAMP_KIND_SHIFT);
/// Set where the record's lamp is burning: always for a torch, a hanging
/// lantern, a brazier and a candle, and from dusk to dawn for a street or wall
/// lantern. The shader draws a lantern's glass alight only then, so the
/// lantern drawn and the light the field holds agree.
pub const LAMP_LIT_BIT: u32 = 1 << 28;
const _: () = assert!(Material::LAMPS.len() <= 8);

/// Where a column record carries the top of its water, as a layer plus one:
/// the middle of `more[3]`, between the rim flag and the torch. Nought is a
/// column with no water in it at all, which is every column under dry land.
///
/// Nine bits, because a layer index is at most `LAYERS`, and a test holds
/// that against the shader's own copy of the shift.
pub const WATER_SHIFT: u32 = 4;
pub const WATER_MASK: u32 = 0x1ff;
const _: () = assert!(LAYERS < WATER_MASK as usize);
const _: () = assert!(WATER_SHIFT + 9 <= TORCH_SHIFT);

/// Where a cell record carries its column slot, plus one: the HIGH half of
/// `metadata.z`, whose low half is the baked sky occlusion in 0..65535.
///
/// An integer field rather than the record's spare `f32`, which is where this
/// started. A slot plus one written as `f32::from_bits` is a DENORMAL for every
/// slot this tier can hold, and a driver is free to flush a denormal to zero on
/// load: the whole tier would read as "no column" and nothing underground would
/// draw at all. It happens to survive on lavapipe, which is exactly the kind of
/// thing that survives every test and fails on somebody's machine.
const SLOT_SHIFT: u32 = 16;
const SKYLIGHT_MASK: u32 = 0xffff;

/// The slot word a neighbour reference carries when that neighbour has no
/// column: off the tier, so treated as solid, so a flank draws nothing there.
pub const NO_NEIGHBOR: u32 = u32::MAX;

/// One column as the vertex shader reads it: the runs to draw and where to look
/// up the neighbour whose rock decides how much of a flank is exposed.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub(crate) struct GpuColumn {
    /// `MAX_RUNS` packed runs, bottom up, absent ones last and zero.
    pub runs: [u32; MAX_RUNS],
    /// Column slots of sides 0 to 3, `NO_NEIGHBOR` off the tier.
    pub neighbors: [u32; 4],
    /// Sides 4 and 5, the cell's own degree, then the rim flag in the low bit
    /// with the TORCH layer plus one in the high half.
    ///
    /// A torch is not solid, so it is in no run, so the shader has no other
    /// way to know where one is: the runs are the whole of what it reads about
    /// a column. Plus one, so the zero a record is born with means "no torch"
    /// and nothing has to be cleared to say so.
    pub more: [u32; 4],
}

// A slot plus one has to fit the high half of the skylight word.
const _: () = assert!(COLUMN_CAPACITY < (1 << 16) - 1);

// Match the WGSL ColumnRec storage ABI.
const _: [(); 48] = [(); size_of::<GpuColumn>()];
const _: [(); 16] = [(); std::mem::offset_of!(GpuColumn, neighbors)];
const _: [(); 32] = [(); std::mem::offset_of!(GpuColumn, more)];

/// Layers packed into one `u32` of the light buffer. Four nibble-ranged levels
/// fit a byte each, and a byte array is not a thing WGSL can index.
pub const LIGHT_PER_WORD: usize = 4;
/// Words of light per column.
pub const LIGHT_WORDS: usize = LAYERS / LIGHT_PER_WORD;
/// Layers packed into one `u32` of the material buffer: a render code is four
/// bits, so eight to a word and forty words a column.
pub const MATERIAL_PER_WORD: usize = 8;
/// Words of material per column.
pub const MATERIAL_WORDS: usize = LAYERS / MATERIAL_PER_WORD;
// Every render code has to fit its nibble.
const _: () = assert!(super::terrain::DIRT < 16);

/// The columns around one anchor, and the map from a finest record to its slot.
#[derive(Clone)]
pub struct ColumnTier {
    /// The authoritative stacks, one per slot. Collision and mining read these;
    /// the GPU only ever sees the packed runs.
    pub columns: Vec<Column>,
    /// Slot per finest record index, or `usize::MAX` where there is none.
    pub slots: Vec<usize>,
    /// What the vertex shader reads, one per slot.
    pub(crate) records: Vec<GpuColumn>,
    /// The sky level of every cell, one array per slot. This is the field a
    /// face's corner samples; the contact darkening is the shader's, because
    /// there is no CPU mesh here to bake a vertex colour into.
    light: light::Baked,
    /// Whether the lamps that burn from dusk to dawn are lit in the field
    /// (`lamps-and-lanterns` decision 1). A tier is built with them out;
    /// `set_dusk` re-bakes when the clock says otherwise.
    dusk: bool,
    /// Whether each slot's cell grows a glowing flower: the record's
    /// `GLOW_FLOWER_BIT`, kept per slot so the bake need not look the record
    /// up (`lamps-and-lanterns` decision 8).
    glow: Vec<bool>,
    /// Each slot's unit direction, and the neighbour table the last bake
    /// joined on: what `sample` blends a point across (decision 10).
    centres: Vec<Vec3>,
    sides: Vec<[u32; 6]>,
}

impl ColumnTier {
    /// An empty tier, which is every cell answering from the heightfield.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            slots: Vec::new(),
            records: Vec::new(),
            light: Vec::new(),
            dusk: false,
            glow: Vec::new(),
            centres: Vec::new(),
            sides: Vec::new(),
        }
    }

    /// Whether the dusk-lit lamps are lit in this tier's field.
    pub fn dusk(&self) -> bool {
        self.dusk
    }

    /// Light or put out the dusk-lit lamps, re-baking the field when that
    /// changes it. Nothing else changes: the columns, the runs and what is
    /// solid are the same by day and by night.
    pub fn set_dusk(&mut self, lit: bool) {
        if self.dusk != lit {
            self.dusk = lit;
            self.relight();
            // And every lantern's glass and glowing flower, which the record
            // carries.
            for (slot, (column, record)) in
                self.columns.iter().zip(self.records.iter_mut()).enumerate()
            {
                let glow = self.glow.get(slot).copied().unwrap_or(false);
                record.more[3] = (record.more[3] & RIM_BIT) | state_word(column, lit, glow);
            }
        }
    }

    /// The column of a finest record, if that record is in the tier.
    pub fn column(&self, record: usize) -> Option<&Column> {
        let slot = *self.slots.get(record)?;
        self.columns.get(slot)
    }

    /// The column of a finest record, mutably: what an edit writes through.
    pub fn column_mut(&mut self, record: usize) -> Option<&mut Column> {
        let slot = *self.slots.get(record)?;
        self.columns.get_mut(slot)
    }

    /// Write one layer of one record's column. `false` where there is no
    /// column, or where the column refused it - bedrock does.
    pub fn set_layer(&mut self, record: usize, layer: usize, material: Material) -> bool {
        self.column_mut(record)
            .is_some_and(|column| column.set(layer, material))
    }

    /// Repack a record's runs from its column.
    ///
    /// Called for the edited cell AND for each of its neighbours: a flank is
    /// clipped against the air gaps of the column next to it, so a neighbour's
    /// drawn side changes when this column does even though its own stack did
    /// not. Everything else in the tier is left alone, which is the whole
    /// reason an edit is not a rebuild.
    pub fn repack(&mut self, record: usize) {
        let Some(&slot) = self.slots.get(record) else {
            return;
        };
        let Some(column) = self.columns.get(slot) else {
            return;
        };
        let runs = column.packed_runs(render_code);
        let glow = self.glow.get(slot).copied().unwrap_or(false);
        let state = state_word(column, self.dusk, glow);
        if let Some(entry) = self.records.get_mut(slot) {
            entry.runs = runs;
            // The rim flag is the tier's to know and everything else in the
            // word is the column's, derived in one place so a build and an
            // edit cannot pack it differently.
            entry.more[3] = (entry.more[3] & RIM_BIT) | state;
        }
    }

    /// Take a column into the tier for a finest record that has none, which
    /// is what an edit past the tier's edge does rather than wait for the next
    /// rebuild (`openspec/changes/fine-set-in-a-second/design.md`).
    ///
    /// The column is the caller's, generated SOLID with the cell's edits: a
    /// cell outside the tier's disc has a neighbour off it, which makes it rim
    /// by the rule `build` applies, and the rim is solid. It takes the next
    /// slot, names its resident neighbours, and each of them names it back, so
    /// both flanks clip against the real columns. The light is extended dark;
    /// the edit's whole-tier relight, which follows, is what lights it.
    ///
    /// `false`, changing nothing, where the record is not a finest record,
    /// already has a column, or the tier is full.
    pub fn adopt(
        &mut self,
        finest: &mut [GpuCell],
        neighbors: &[[u32; 6]],
        index: usize,
        column: Column,
    ) -> bool {
        if self.slots.get(index) != Some(&usize::MAX)
            || self.columns.len() >= COLUMN_CAPACITY as usize
        {
            return false;
        }
        let slot = self.columns.len();
        let degree = finest[index].degree();
        let table = neighbors.get(index).copied().unwrap_or([u32::MAX; 6]);
        let mut sides = [NO_NEIGHBOR; 6];
        for side in 0..degree {
            let neighbor = table[side] as usize;
            let Some(&there) = self.slots.get(neighbor) else {
                continue;
            };
            if there == usize::MAX {
                continue;
            }
            sides[side] = there as u32;
            let back = neighbors[neighbor]
                .iter()
                .take(finest[neighbor].degree())
                .position(|&id| id as usize == index);
            if let (Some(back), Some(record)) = (back, self.records.get_mut(there)) {
                if back < 4 {
                    record.neighbors[back] = slot as u32;
                } else {
                    record.more[back - 4] = slot as u32;
                }
            }
        }
        self.slots[index] = slot;
        finest[index].metadata[2] =
            (finest[index].metadata[2] & SKYLIGHT_MASK) | (slot as u32 + 1) << SLOT_SHIFT;
        self.records.push(GpuColumn {
            runs: column.packed_runs(render_code),
            neighbors: [sides[0], sides[1], sides[2], sides[3]],
            more: [
                sides[4],
                sides[5],
                degree as u32,
                RIM_BIT | state_word(&column, self.dusk, glows(&finest[index])),
            ],
        });
        self.glow.push(glows(&finest[index]));
        self.centres
            .push(Vec3::from_slice(&finest[index].direction_height[..3]).normalize_or(Vec3::Y));
        self.columns.push(column);
        self.light.push([light::Light::DARK; LAYERS]);
        true
    }

    /// What the vertex shader reads, one record per slot.
    pub(crate) fn gpu_records(&self) -> &[GpuColumn] {
        &self.records
    }

    /// What one cell is lit to, both channels.
    pub fn light_at(&self, slot: usize, layer: usize) -> light::Light {
        self.light
            .get(slot)
            .and_then(|levels| levels.get(layer))
            .copied()
            .unwrap_or(light::Light::DARK)
    }

    /// The sky level of one cell, 0 to `light::MAX`.
    pub fn sky(&self, slot: usize, layer: usize) -> u8 {
        self.light_at(slot, layer).sky()
    }

    /// The light field packed for the GPU: four layers to a word, `LIGHT_WORDS`
    /// words per slot, slots end to end.
    ///
    /// A word rather than a byte array because WGSL indexes `u32`, and four to
    /// a word rather than eight nibbles because a byte is what a level fits in
    /// and unpacking a nibble in the vertex shader buys nothing: the buffer is
    /// 5 MiB at full capacity either way against the 96 MiB the cell records
    /// already hold.
    pub fn gpu_light(&self) -> Vec<u32> {
        let mut words = vec![0u32; self.light.len() * LIGHT_WORDS];
        for (slot, levels) in self.light.iter().enumerate() {
            for (layer, &level) in levels.iter().enumerate() {
                let word = slot * LIGHT_WORDS + layer / LIGHT_PER_WORD;
                let shift = (layer % LIGHT_PER_WORD) * 8;
                words[word] |= (level.0 as u32) << shift;
            }
        }
        words
    }

    /// Every layer's render code, eight to a word, `MATERIAL_WORDS` words per
    /// slot, slots end to end: what lets a face be drawn in its own voxel's
    /// material rather than in a rule on depth. A placed stone is stone on
    /// every face because the layer says so, and a hillside is sod, earth
    /// and stone because those are the layers the generator wrote.
    pub fn gpu_materials(&self) -> Vec<u32> {
        let mut words = vec![0u32; self.columns.len() * MATERIAL_WORDS];
        for (slot, column) in self.columns.iter().enumerate() {
            for layer in 0..LAYERS {
                let word = slot * MATERIAL_WORDS + layer / MATERIAL_PER_WORD;
                let shift = (layer % MATERIAL_PER_WORD) * 4;
                words[word] |= (render_code(column.material(layer)) & 0xf) << shift;
            }
        }
        words
    }

    /// Re-light the whole tier from its columns.
    ///
    /// The whole tier rather than the reference's bounded incremental pass,
    /// and that is a measurement rather than a preference: see the tier's
    /// startup line for what a bake costs here. A dig that relights everything
    /// in a few milliseconds is simpler than one that relights a sphere of
    /// fifteen cells and has to get the removal pass right, and the removal
    /// pass is where the reference records its own scar - a dug cell that
    /// "stayed dark forever".
    pub fn relight(&mut self) {
        self.sides = self.neighbor_slots();
        let emitters = self.emitters();
        self.light = light::bake(
            &light::Region {
                columns: &self.columns,
                neighbors: &self.sides,
            },
            &emitters,
        );
    }

    /// Both channels at a point, 0..1 `(sky, block)`: what lights the things
    /// the terrain pass does not draw (`lamps-and-lanterns` decision 10).
    /// `record` is the finest record the point is over, `direction` its unit
    /// direction and `altitude` its metres above the planet's radius. Off the
    /// tier it answers the open sky and no lamp.
    pub fn sample(&self, record: Option<usize>, direction: Vec3, altitude: f32) -> (f32, f32) {
        let slot = record
            .and_then(|record| self.slots.get(record).copied())
            .filter(|&slot| slot != usize::MAX && slot < self.sides.len());
        light::sample(
            &light::Region {
                columns: &self.columns,
                neighbors: &self.sides,
            },
            &self.light,
            &self.centres,
            slot.map(|slot| slot as u32),
            direction.normalize_or(Vec3::Y),
            altitude - column::BASE_M as f32,
        )
    }

    /// Every cell in the tier that gives out light.
    ///
    /// DERIVED from the columns rather than kept beside them, which is what
    /// makes a torch need no bookkeeping: placing one is an ordinary edit, and
    /// the next relight finds it because it is in the column. A list kept
    /// alongside would be a second place a dug-up torch had to be removed
    /// from, and the one that was forgotten would light an empty cell for
    /// ever.
    fn emitters(&self) -> Vec<light::Emitter> {
        let mut found = Vec::new();
        for (slot, column) in self.columns.iter().enumerate() {
            for layer in 0..LAYERS {
                let material = column.material(layer);
                let level = material.emission();
                // A street lantern by day is an unlit lantern.
                if level > 0 && (self.dusk || !material.dusk_lit()) {
                    found.push(light::Emitter {
                        column: slot as u32,
                        layer: layer as u16,
                        level,
                    });
                }
            }
            // A glowing flower, from dusk to dawn, while its ground is sod.
            if self.dusk
                && self.glow.get(slot).copied().unwrap_or(false)
                && let Some(layer) = flower_layer(column)
            {
                found.push(light::Emitter {
                    column: slot as u32,
                    layer: layer as u16,
                    level: flora::GLOW_FLOWER_LEVEL,
                });
            }
        }
        found
    }

    /// The neighbour table in SLOT space, which is what the light field joins
    /// on. `NO_NEIGHBOR` and `light::OFF_REGION` are the same value, and a test
    /// holds them together.
    fn neighbor_slots(&self) -> Vec<[u32; 6]> {
        self.records
            .iter()
            .map(|record| {
                [
                    record.neighbors[0],
                    record.neighbors[1],
                    record.neighbors[2],
                    record.neighbors[3],
                    record.more[0],
                    record.more[1],
                ]
            })
            .collect()
    }
}

/// Make a cell RECORD agree with the column under it.
///
/// **There are two representations of where the ground is**, and this is the
/// only place they are reconciled. The column says what stands at every metre;
/// the cell record carries the surface HEIGHT, the cap's material, and - in
/// each neighbour's `corners[side].w` - how far down that neighbour draws its
/// wall. The terrain pass draws from the record and the column pass draws from
/// the runs, so a record that disagrees with its column is a meadow drawn over
/// a hole and a wall drawn across it.
///
/// The reference has nothing like this because it has nothing to reconcile:
/// `tenebris-core`'s `World::set` writes one `blocks` array and dirties the
/// tile and its lateral neighbours, and its mesher derives the surface from
/// that array. Everything it draws comes from the one representation. Ours has
/// a heightfield tier that a column tier is embedded in, which is what buys a
/// 4,800 m planet at 2.8 m cells - and the price is exactly this function.
pub fn reconcile_surface(
    finest: &mut [GpuCell],
    neighbors: &[[u32; 6]],
    index: usize,
    column: &Column,
) {
    let Some(top) = column.surface() else {
        return;
    };
    let top_m = column::layer_altitude(top) + 1.0;
    finest[index].direction_height[3] = top_m;
    let code = render_code(column.material(top));
    finest[index].metadata[1] = (finest[index].metadata[1] & !0xff) | code;
    let Some(table) = neighbors.get(index).copied() else {
        return;
    };
    for &neighbor in table.iter().take(finest[index].degree()) {
        if neighbor == u32::MAX {
            continue;
        }
        let n = neighbor as usize;
        // The neighbour's side that faces back at this cell.
        let back = neighbors[n]
            .iter()
            .take(finest[n].degree())
            .position(|&id| id as usize == index);
        if let Some(side) = back {
            finest[n].corners[side][3] = top_m;
        }
    }
}

/// The lamp word of a column: the topmost lamp's layer plus one and its kind,
/// shifted, or zero where there is none.
///
/// The TOPMOST, and placement refuses a second in the same column, so the one
/// drawn and the one lighting are the same lamp. A record that could hold one
/// while the field lit two would be a lamp burning in an empty cell.
fn lamp_word(column: &Column, dusk: bool) -> u32 {
    for layer in (1..LAYERS).rev() {
        let material = column.material(layer);
        if let Some(kind) = Material::LAMPS.iter().position(|&lamp| lamp == material) {
            let lit = if dusk || !material.dusk_lit() {
                LAMP_LIT_BIT
            } else {
                0
            };
            return (layer as u32 + 1) << TORCH_SHIFT | (kind as u32) << LAMP_KIND_SHIFT | lit;
        }
    }
    0
}

/// Whether this column already holds a lamp of any kind.
pub fn has_lamp(column: &Column) -> bool {
    lamp_word(column, false) != 0
}

/// The rim flag's bit in `more[3]`.
pub const RIM_BIT: u32 = 1;

/// The topmost water layer of a column, plus one, shifted: nought where the
/// column holds no water. The shader turns it into the altitude of the water's
/// surface, which is what decides whether a face under sea level is wet.
fn water_word(column: &Column) -> u32 {
    for layer in (1..LAYERS).rev() {
        if column.material(layer) == Material::Water {
            return (layer as u32 + 1) << WATER_SHIFT;
        }
    }
    0
}

/// Everything in `more[3]` that is the COLUMN's rather than the tier's: where
/// its water tops out, where its torch is, and whether its glowing flower
/// burns. One function, because a record packed at build and a record
/// repacked by an edit that disagreed would be a column whose lamp or
/// waterline depended on when it was last touched.
fn state_word(column: &Column, dusk: bool, glow: bool) -> u32 {
    let flower = if dusk && glow && flower_layer(column).is_some() {
        GLOW_LIT_BIT
    } else {
        0
    };
    water_word(column) | lamp_word(column, dusk) | flower
}

/// Set in `more[3]` while this column's glowing flower burns: the cell grows
/// one, its ground is sod, and it is night. The shader draws the flower's head
/// lit where this is set (`lamps-and-lanterns` decision 8).
pub const GLOW_LIT_BIT: u32 = 1 << 29;
const _: () = assert!(GLOW_LIT_BIT > LAMP_LIT_BIT);

/// Set in a finest record's `spare[2]` where its cell grows a glowing flower,
/// chosen once from the cell's exact key (`pbd_core::flora::glows`).
pub const GLOW_FLOWER_BIT: u32 = 1;

/// Whether a record's cell grows a glowing flower.
fn glows(cell: &GpuCell) -> bool {
    cell.spare[2] & GLOW_FLOWER_BIT != 0
}

/// The render codes the shader grows flowers on: its `grassy`. A test reads
/// the shader's function and holds the two to one list.
pub const GRASSY_CODES: [u32; 3] = [2, 3, 7];

/// The air layer a flower stands in, over sod: where a glowing flower's light
/// comes from. `None` where the ground is not grassy or something stands on
/// it, which is how digging the sod out or building over it puts the flower
/// out with no bookkeeping.
fn flower_layer(column: &Column) -> Option<usize> {
    let top = column.surface()?;
    let above = top + 1;
    (above < LAYERS
        && column.material(above) == Material::Air
        && GRASSY_CODES.contains(&render_code(column.material(top))))
    .then_some(above)
}

/// The topmost water layer of a column, if it holds any: what `more[3]` packs.
pub fn water_top_layer(column: &Column) -> Option<usize> {
    let word = (water_word(column) >> WATER_SHIFT) & WATER_MASK;
    (word != 0).then(|| word as usize - 1)
}

/// Build the tier for an anchor, and stamp each finest record with its slot.
///
/// Two passes, because a column's record names its neighbours' SLOTS and a slot
/// does not exist until every cell in the tier has one.
pub fn build(
    anchor: Vec3,
    finest: &mut [GpuCell],
    neighbors: &[[u32; 6]],
    settings: &ColumnSettings,
    edits: &Edits,
) -> ColumnTier {
    let anchor = anchor.normalize_or(Vec3::Y);
    // No finest level (the player is too high for it to be live): no column
    // to build, and no worms to gather for them.
    if finest.is_empty() {
        return ColumnTier::empty();
    }
    // The region's worms, gathered ONCE: every worm that could reach any
    // column of the tier, so each column's carve is complete whatever tier
    // built it. This is the regional pre-pass the design said worms need,
    // done at the one place columns are built in bulk.
    let field = settings.worms();
    let region = worms::gather(&field, terrain_config(), anchor, settings.reach_m.max(0.));
    let reach = (settings.reach_m.max(0.) / PLANET_RADIUS).cos();
    let mut slots = vec![usize::MAX; finest.len()];
    let mut members = Vec::new();
    for (index, cell) in finest.iter_mut().enumerate() {
        // Whether the cell grows a glowing flower, decided here once from its
        // key, for every finest cell: the shader grows the flower wherever
        // the bit is, and the bake lights the ones in the tier.
        let key = cell.key();
        let glowing =
            key != crate::planet::NO_KEY && flora::glows(key, settings.glow_flower_chance);
        cell.spare[2] = (cell.spare[2] & !GLOW_FLOWER_BIT) | glowing as u32;
        let direction = Vec3::from_slice(&cell.direction_height[..3]);
        if direction.dot(anchor) <= reach || members.len() >= COLUMN_CAPACITY as usize {
            cell.metadata[2] &= SKYLIGHT_MASK;
            continue;
        }
        slots[index] = members.len();
        // The slot plus one, so the zero a record is born with means no column
        // and nothing has to be cleared to say so.
        cell.metadata[2] =
            (cell.metadata[2] & SKYLIGHT_MASK) | (members.len() as u32 + 1) << SLOT_SHIFT;
        members.push(index);
    }
    let mut columns = Vec::with_capacity(members.len());
    let mut records = Vec::with_capacity(members.len());
    let mut glow = Vec::with_capacity(members.len());
    let mut centres = Vec::with_capacity(members.len());
    for &index in &members {
        let cell = &finest[index];
        let direction = Vec3::from_slice(&cell.direction_height[..3]);
        let degree = cell.degree();
        let mut sides = [NO_NEIGHBOR; 6];
        let table = neighbors.get(index).copied().unwrap_or([u32::MAX; 6]);
        for (slot, &neighbor) in sides.iter_mut().zip(&table).take(degree) {
            if neighbor != u32::MAX {
                let found = slots.get(neighbor as usize).copied().unwrap_or(usize::MAX);
                if found != usize::MAX {
                    *slot = found as u32;
                }
            }
        }
        // The RIM is generated SOLID. A cell with a neighbour off the tier is
        // where the column world meets the heightfield world, and the
        // heightfield's one assumption is that the ground under a cap is rock.
        // Carving the rim makes that assumption false exactly where nothing can
        // correct it: an off-tier cell draws no face under its cap, so a cave
        // reaching the rim is a hole a player looks out of, at open sky and a
        // sea two kilometres away.
        //
        // A face cannot fix it, and that was the other thing tried: a flank
        // drawn on the rim's outward side points away from everyone inside the
        // tier, and this pipeline culls back faces, so it draws nothing anybody
        // can see. Rock can fix it, because rock is what the assumption says is
        // there. One cell thick is enough to occlude, and the ring moves out
        // with the tier every `REGEN_DISTANCE_M`, so a player walking toward it
        // never arrives.
        let rim = sides[..degree].contains(&NO_NEIGHBOR);
        // A player's edits come last, so a dug shelf survives the tier being
        // rebuilt at a new anchor: the column is a pure function of its
        // direction, the worms and what somebody did to it. The rim is solid
        // by the rule above and takes its edits too - a wall you dug in is
        // still a wall you dug in when the tier moves and it becomes the rim.
        let made = &edits.for_cell(cell.key());
        let column = if rim {
            column::generate_edited_solid(terrain_config(), direction, made)
        } else {
            column::generate_edited(&region, &field, terrain_config(), direction, made)
        };
        records.push(GpuColumn {
            runs: column.packed_runs(render_code),
            neighbors: [sides[0], sides[1], sides[2], sides[3]],
            more: [
                sides[4],
                sides[5],
                degree as u32,
                rim as u32 | state_word(&column, false, glows(cell)),
            ],
        });
        glow.push(glows(cell));
        centres.push(direction);
        columns.push(column);
    }
    // A MOUTH lowers the ground. Where the carve broke the surface, the
    // column's top is below the height the record was built with, and the
    // record is what the terrain pass draws its cap from, what the neighbours
    // draw their walls down to, and what the walker stands on outside a cave.
    // Left alone it would draw a meadow over the hole and a wall across it.
    // So the record takes the column's top, its cap wears the material that
    // is actually there, and every neighbour's wall height for this side
    // follows. Outside a mouth the column top IS the record's height - both
    // read `column::surface_m` - and the record is left exactly as it was.
    for (slot, &index) in members.iter().enumerate() {
        let column = &columns[slot];
        let Some(top) = column.surface() else {
            continue;
        };
        let top_m = column::layer_altitude(top) + 1.0;
        // Only DOWNWARD at build time: a carve can only take the surface
        // away, and a column top above the record is a fault a test holds
        // out. An EDIT is the other case and reconciles both ways - see
        // `reconcile_surface`.
        if top_m >= finest[index].direction_height[3] {
            continue;
        }
        reconcile_surface(finest, neighbors, index, column);
    }
    let mut tier = ColumnTier {
        columns,
        slots,
        records,
        light: Vec::new(),
        dusk: false,
        glow,
        centres,
        sides: Vec::new(),
    };
    // The light comes last, because it is a function of the finished columns:
    // the rim's solid ring and every edit are already in them, and lighting
    // before either would light a world that is not the one being drawn.
    tier.relight();
    tier
}

/// The step a walker can take up or down, in metres: one cell of elevation
/// plus the contact skin. `WalkingConfig::step_height` is the same number and
/// reads it from here, because what a walker can climb and what counts as a
/// doorway are one fact - a mouth a walker cannot step into is not a mouth.
pub const STEP_M: f32 = crate::planet::terrain::ELEVATION_STEP + 0.05;

/// Where this column is open to the ground NEXT DOOR, if it is: the floor and
/// roof of the gap and the side it faces. A mouth is a gap whose FLOOR is
/// within a step of the neighbour's cap, so a walker standing there walks in;
/// a gap whose floor is five metres below their feet is a hole they fall into,
/// which is what the first rule counted and what put the capture's camera over
/// a pit looking down. The tallest such gap wins, and the count, the capture
/// and the walker all ask this one function.
pub fn mouth_of(cell: &GpuCell, runs: &[column::Run]) -> Option<(f32, f32, usize)> {
    let mut best: Option<(f32, f32, usize)> = None;
    for pair in runs.windows(2) {
        let floor = column::layer_altitude(pair[0].to);
        let roof = column::layer_altitude(pair[1].from);
        for side in 0..cell.degree() {
            let cap = cell.corners[side][3];
            if (floor - cap).abs() <= STEP_M
                && roof > cap + 0.5
                && best.is_none_or(|(had_floor, had_roof, _)| roof - floor > had_roof - had_floor)
            {
                best = Some((floor, roof, side));
            }
        }
    }
    best
}

/// The slot a cell record carries, or `None`. What the shaders read off the
/// same word, written here so a test can hold the two together.
#[cfg(test)]
pub(crate) fn slot_of(cell: &GpuCell) -> Option<u32> {
    // Zero is what a record is born with, so it is what "no column" means and
    // nothing has to be cleared to say it.
    let word = cell.metadata[2] >> SLOT_SHIFT;
    (word != 0).then(|| word - 1)
}

#[cfg(test)]
mod water_word_tests {
    use super::*;

    fn flooded(ground: usize, sea: usize) -> Column {
        let mut column = Column::bedrock();
        for layer in 1..=ground {
            column.set(layer, Material::Stone);
        }
        for layer in ground + 1..=sea {
            column.set(layer, Material::Water);
        }
        column
    }

    /// The record carries the top of the column's water where the shader
    /// looks for it, and nought where the column holds none: that nought is
    /// what makes a dry cave dry, so it is worth a test of its own.
    #[test]
    fn a_columns_water_top_rides_its_record_and_dry_is_nought() {
        let wet = flooded(20, 40);
        assert_eq!(water_top_layer(&wet), Some(40));
        let packed = (state_word(&wet, false, false) >> WATER_SHIFT) & WATER_MASK;
        assert_eq!(packed, 41, "the layer plus one, as the shader reads it");
        // The shader turns the packed word into the water's SURFACE, which is
        // the altitude of the top layer plus one.
        assert_eq!(
            pbd_core::column::BASE_M as f32 + packed as f32,
            pbd_core::column::layer_altitude(40) + 1.0
        );

        let dry = flooded(20, 20);
        assert_eq!(water_top_layer(&dry), None);
        assert_eq!(
            state_word(&dry, false, false) >> WATER_SHIFT & WATER_MASK,
            0
        );

        // The torch shares the word and must survive beside it.
        let mut lit = wet;
        lit.set(45, Material::Torch);
        assert_eq!(
            (state_word(&lit, false, false) >> TORCH_SHIFT) & LAMP_LAYER_MASK,
            46
        );
        assert_ne!(
            state_word(&lit, false, false) & LAMP_LIT_BIT,
            0,
            "a torch always burns"
        );
        let mut candle = Column::bedrock();
        candle.set(60, Material::Candle);
        let word = state_word(&candle, false, false);
        assert_eq!(
            (word >> TORCH_SHIFT) & LAMP_LAYER_MASK,
            61,
            "the candle's layer"
        );
        assert_eq!(
            Material::LAMPS[((word >> LAMP_KIND_SHIFT) & 7) as usize],
            Material::Candle,
            "and which lamp it is"
        );
        assert_eq!(
            (state_word(&lit, false, false) >> WATER_SHIFT) & WATER_MASK,
            41
        );
        assert_eq!(
            state_word(&lit, false, false) & RIM_BIT,
            0,
            "the rim is the tier's"
        );
    }
}

#[cfg(test)]
mod water_below_sea {
    //! A measurement instrument for the volumetric-water change: how much of
    //! the spawn tier is air below sea level, split by whether the ground
    //! over it stands above the sea (a dry cave, which is right) or under it
    //! (a dry tube under the ocean, which is not). Ignored because it builds
    //! the fine set; run it with
    //! `cargo test -p pbd-app --release --lib water_below_sea -- --ignored --nocapture`.
    use super::*;
    use crate::planet::lod;

    use super::super::terrain::nearest_ground_near;

    fn report(label: &str, anchor: Vec3) {
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::default());
        let tier = &set.columns;
        let sea = terrain_config().sea_level_m;
        let (mut dry_cave_columns, mut dry_cave_layers) = (0usize, 0usize);
        let (mut dry_tube_columns, mut dry_tube_layers) = (0usize, 0usize);
        let (mut water_columns, mut water_layers) = (0usize, 0usize);
        for column in &tier.columns {
            let surface = column
                .surface()
                .map_or(f32::MIN, |top| column::layer_altitude(top) + 1.0);
            let mut air_below_sea = 0usize;
            let mut water = 0usize;
            for layer in 1..column::LAYERS {
                let altitude = column::layer_altitude(layer);
                if altitude >= sea {
                    break;
                }
                match column.material(layer) {
                    Material::Air => air_below_sea += 1,
                    Material::Water => water += 1,
                    _ => {}
                }
            }
            if water > 0 {
                water_columns += 1;
                water_layers += water;
            }
            if air_below_sea > 0 {
                if surface >= sea {
                    dry_cave_columns += 1;
                    dry_cave_layers += air_below_sea;
                } else {
                    dry_tube_columns += 1;
                    dry_tube_layers += air_below_sea;
                }
            }
        }
        eprintln!(
            "{label}: {} columns, ground at the anchor {:.1} m, sea level {sea:.1} m; \
             {water_columns} columns hold {water_layers} water layers; \
             {dry_cave_columns} columns under LAND carry {dry_cave_layers} air layers below sea level (dry caves, right); \
             {dry_tube_columns} columns under the SEA carry {dry_tube_layers} air layers below sea level (dry tubes, wrong)",
            tier.columns.len(),
            super::super::terrain::surface_height(anchor)
        );
    }

    #[test]
    #[ignore]
    fn water_below_sea_in_the_spawn_tier() {
        let spawn = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        report("spawn", spawn);
        match nearest_ground_near(spawn, 0.5..4.0) {
            Some(shore) => report("nearest shore", shore),
            None => eprintln!("no shore within 3 km of the spawn"),
        }
        match nearest_ground_near(spawn, -12.0..-2.0) {
            Some(shallows) => report("nearest shallows", shallows),
            None => eprintln!("no shallows within 3 km of the spawn"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::lod;

    /// The tier is built off the real fine set, so what is tested is what the
    /// frame is handed rather than a hand-built stand-in.
    fn tier(anchor: Vec3) -> (lod::FineSet, ColumnTier) {
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = set.take_columns();
        (set, tier)
    }

    /// Measurement for `overcast-and-rain` section 5: how many cave floors in
    /// the tier around the spawn are under rock yet reached by sky LIGHT,
    /// which is what used to wet them. Run with `--ignored --nocapture`.
    #[test]
    #[ignore]
    fn rain_under_rock_report() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let (_, tier) = tier(anchor);
        let (mut floors, mut lit, mut wet_sum, mut brightest) = (0usize, 0usize, 0.0f32, 0u8);
        for (slot, column) in tier.columns.iter().enumerate() {
            let runs = column.drawn_runs();
            for run in &runs[..runs.len().saturating_sub(1)] {
                if run.from == 0 && run.to <= 1 {
                    continue;
                }
                let air = run.to.min(LAYERS - 1);
                assert!(!column.open_to_sky(column::layer_altitude(run.to)));
                floors += 1;
                let sky = tier.light[slot][air].sky();
                if sky > 0 {
                    lit += 1;
                    wet_sum += f32::from(sky) / 15.0;
                    brightest = brightest.max(sky);
                }
            }
        }
        println!(
            "cave floors under rock: {floors}; reached by sky light (so wetted before): {lit} \
             ({:.1}%), mean wetness there {:.2}, brightest sky level {brightest}/15",
            100.0 * lit as f32 / floors.max(1) as f32,
            wet_sum / lit.max(1) as f32
        );
    }

    #[test]
    fn the_tier_covers_its_radius_and_stops_there() {
        let anchor = Vec3::new(0.3, 0.7, 0.2).normalize();
        let (set, tier) = tier(anchor);
        let finest = set.finest_records();
        assert!(!tier.columns.is_empty(), "the tier must hold columns");
        let reach_m = ColumnSettings::default().reach_m;
        let reach = reach_m / PLANET_RADIUS;
        for (index, cell) in finest.iter().enumerate() {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            let angle = direction.dot(anchor).clamp(-1., 1.).acos();
            // A cell sitting exactly on the boundary is either, so the claim is
            // made a tile clear of it on both sides. Pinning the boundary cell
            // itself would be pinning a float comparison, not the tier.
            let margin = lod::tile_width_m(lod::FINEST_LEVEL) / PLANET_RADIUS;
            if (angle - reach).abs() < margin {
                continue;
            }
            let inside = angle < reach;
            assert_eq!(
                tier.column(index).is_some(),
                inside,
                "a cell {:.0} m out against a {reach_m} m tier",
                angle * PLANET_RADIUS
            );
            assert_eq!(
                slot_of(cell).is_some(),
                inside,
                "the record's own slot word must agree with the tier"
            );
            // And the slot must not have eaten the sky occlusion sharing the
            // word with it: a cell in the tier would go black.
            assert!(
                (cell.metadata[2] & SKYLIGHT_MASK) > 0,
                "the skylight half of the word survived the slot"
            );
        }
    }

    #[test]
    fn a_neighbour_slot_points_at_the_neighbouring_column() {
        let anchor = Vec3::new(-0.4, 0.2, 0.9).normalize();
        let (set, tier) = tier(anchor);
        let finest = set.finest_records();
        let mut checked = 0;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let record = tier.records[slot];
            let degree = record.more[2] as usize;
            let sides = [
                record.neighbors[0],
                record.neighbors[1],
                record.neighbors[2],
                record.neighbors[3],
                record.more[0],
                record.more[1],
            ];
            for (side, &named) in sides.iter().enumerate().take(degree) {
                if named == NO_NEIGHBOR {
                    continue;
                }
                // The slot named is the slot of the cell that record's own
                // neighbour table names, which is what the flank clip walks.
                let neighbor = set.finest_neighbors[index][side] as usize;
                assert_eq!(tier.slots[neighbor], named as usize);
                // And it really is adjacent: one tile width apart, not a
                // stale index into a set that has moved.
                let here = Vec3::from_slice(&finest[index].direction_height[..3]);
                let there = Vec3::from_slice(&finest[neighbor].direction_height[..3]);
                let gap = here.distance(there) * PLANET_RADIUS;
                assert!(
                    gap < 2.0 * lod::tile_width_m(lod::FINEST_LEVEL),
                    "side {side} is {gap:.2} m away"
                );
                checked += 1;
            }
        }
        assert!(
            checked > 1_000,
            "only {checked} neighbour slots were checked"
        );
    }

    /// The record's corner `s` carries the height of the neighbour across side
    /// `s`, and the flank clip asks the neighbour table for side `s`. If those
    /// two are not the same neighbour, every clip consults the wrong column.
    #[test]
    fn side_s_of_the_record_and_side_s_of_the_neighbour_table_are_one_neighbour() {
        let anchor = Vec3::new(0.2, 0.6, -0.77).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let _ = set.take_columns();
        let finest = set.finest_records();
        let mut checked = 0;
        for (index, cell) in finest.iter().enumerate() {
            let degree = cell.degree();
            for side in 0..degree {
                let neighbor = set.finest_neighbors[index][side];
                if neighbor == u32::MAX {
                    continue;
                }
                assert_eq!(
                    finest[neighbor as usize].direction_height[3], cell.corners[side][3],
                    "cell {index} side {side}: the record's wall height and the neighbour \
                     table disagree about which neighbour this is"
                );
                checked += 1;
            }
        }
        assert!(checked > 1_000, "only {checked} sides were checked");
    }

    /// A cell well inside the tier has a column on every side of it. Without
    /// this, a record that names no neighbour is indistinguishable from one at
    /// the rim, and the flank clip reads "solid" and draws nothing - which is a
    /// world you can see out of from underground.
    #[test]
    fn an_interior_cell_names_a_column_on_every_side() {
        let anchor = Vec3::new(0.51, -0.33, 0.79).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = set.take_columns();
        let finest = set.finest_records();
        let reach = ColumnSettings::default().reach_m;
        let inner = (reach - 3.0 * lod::tile_width_m(lod::FINEST_LEVEL)) / PLANET_RADIUS;
        let mut checked = 0;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let direction = Vec3::from_slice(&finest[index].direction_height[..3]);
            if direction.dot(anchor).clamp(-1., 1.).acos() > inner {
                continue;
            }
            let record = tier.records[slot];
            let degree = record.more[2] as usize;
            let sides = [
                record.neighbors[0],
                record.neighbors[1],
                record.neighbors[2],
                record.neighbors[3],
                record.more[0],
                record.more[1],
            ];
            for (side, named) in sides.iter().enumerate().take(degree) {
                assert_ne!(
                    *named, NO_NEIGHBOR,
                    "cell {index} side {side} names no column although it is inside the tier"
                );
            }
            checked += 1;
        }
        assert!(
            checked > 1_000,
            "only {checked} interior cells were checked"
        );
    }

    /// How much of the tier is AIR at each altitude, and how far a walk across
    /// it gets. The carve is sampled once per cell, so a crest that runs nearly
    /// flat is a whole LAYER of air over a wide area rather than the thin sheet
    /// the continuous field would give: `pbd_core`'s own `sight_lines` report
    /// measures the field, and this measures what the renderer actually has.
    #[test]
    #[ignore = "a report: cargo test -p pbd-app --lib air_by_layer -- --ignored --nocapture"]
    fn air_by_layer() {
        use pbd_core::column::{LAYERS, layer_altitude};
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = set.take_columns();
        let total = tier.columns.len();
        println!("\n{total} columns in the tier");
        let mut worst = (0usize, 0.0f32);
        for index in 0..LAYERS {
            let altitude = layer_altitude(index);
            let underground = tier
                .columns
                .iter()
                .filter(|c| {
                    c.surface()
                        .is_some_and(|top| layer_altitude(top) + 1.0 > altitude)
                })
                .count();
            if underground < total / 4 {
                continue;
            }
            let air = tier
                .columns
                .iter()
                .filter(|c| {
                    c.surface()
                        .is_some_and(|top| layer_altitude(top) + 1.0 > altitude)
                        && !c.solid(index)
                })
                .count();
            let share = air as f32 / underground as f32;
            if share > worst.1 {
                worst = (index, share);
            }
            if index % 16 == 0 || share > 0.25 {
                println!(
                    "  {altitude:>5.0} m: {underground:>5} underground, {air:>5} air ({:.1}%)",
                    100.0 * share
                );
            }
        }
        println!(
            "worst layer {:.0} m at {:.1}% air",
            layer_altitude(worst.0),
            100.0 * worst.1
        );
    }

    /// How many caves in the tier OPEN onto the surface. A walker can only
    /// walk into a cave that has a mouth: an air gap in one column standing
    /// higher than a neighbour's cap, so the gap is reachable from open air
    /// without digging. The carve is damped toward the surface on purpose, so
    /// this decides whether "walk into a cave" is a collision problem or a
    /// worldgen one.
    #[test]
    #[ignore = "a report: cargo test -p pbd-app --lib cave_mouths -- --ignored --nocapture"]
    fn cave_mouths() {
        // The DEFAULT spawn, with the damping the sweep in pbd-core picked and
        // no mouth patches at all: if openings are there, the patch rule is
        // machinery for a question one number answers.
        let settings = ColumnSettings::default();
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &settings, &Edits::new());
        let tier = set.take_columns();
        let finest = set.finest_records();
        let mut caves = 0;
        let mut mouths = 0;
        let mut walkable = 0;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let runs = tier.columns[slot].drawn_runs();
            if runs.len() < 2 {
                continue;
            }
            caves += 1;
            let mouth = mouth_of(&finest[index], &runs);
            mouths += mouth.is_some() as usize;
            walkable += mouth.is_some_and(|(floor, roof, _)| roof - floor >= 1.8) as usize;
        }
        println!(
            "\n{} columns, {caves} with a cave, {mouths} open to the surface, \
             {walkable} of those tall enough to walk into",
            tier.columns.len()
        );
    }

    /// A mouth is a change to the RECORD: the cap it draws is at the column's
    /// top, and every neighbour's wall for that side goes down to it.
    #[test]
    fn a_mouth_lowers_its_record_and_its_neighbours_walls_follow() {
        use pbd_core::column::layer_altitude;
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = set.take_columns();
        let finest = set.finest_records();
        let mut mouths = 0;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let column = &tier.columns[slot];
            let top_m = layer_altitude(column.surface().unwrap()) + 1.0;
            let height = finest[index].direction_height[3];
            // The column top is the record's height, whether the carve
            // lowered the record or not: never a cap drawn over air, never
            // rock drawn over a cap.
            assert!(
                (top_m - height).abs() < 1e-4,
                "cell {index}: column top {top_m} against record height {height}"
            );
            if (top_m - height).abs() < 1e-4 && top_m < planet_gen_height(finest, index) {
                mouths += 1;
                for (side, &neighbor) in set.finest_neighbors[index]
                    .iter()
                    .enumerate()
                    .take(finest[index].degree())
                {
                    if neighbor == u32::MAX {
                        continue;
                    }
                    let n = neighbor as usize;
                    let back = set.finest_neighbors[n]
                        .iter()
                        .position(|&id| id as usize == index)
                        .expect("adjacency is symmetric");
                    assert_eq!(
                        finest[n].corners[back][3], top_m,
                        "neighbour {n} side {back} must draw its wall down to the mouth (our side {side})"
                    );
                }
            }
        }
        println!("{mouths} mouth columns in the tier");
    }

    /// The surface the generator would have given a record, before any mouth
    /// lowered it.
    fn planet_gen_height(finest: &[GpuCell], index: usize) -> f32 {
        crate::planet::surface_height(Vec3::from_slice(&finest[index].direction_height[..3]))
    }

    #[test]
    fn every_packed_run_reads_back_as_the_column_says() {
        let (_, tier) = tier(Vec3::new(0.8, -0.1, 0.6).normalize());
        for (slot, column) in tier.columns.iter().enumerate() {
            let drawn = column.drawn_runs();
            for (word, run) in tier.records[slot].runs.iter().zip(&drawn) {
                assert_eq!(*word & 0x1ff, run.from as u32);
                assert_eq!((*word >> 9) & 0x1ff, run.to as u32);
                assert_eq!((*word >> 18) & 0xf, render_code(run.material));
                assert_eq!((*word >> 22) & 0xf, render_code(run.body));
            }
            for word in &tier.records[slot].runs[drawn.len()..] {
                assert_eq!(*word, 0, "an absent run is the zero word");
            }
        }
    }

    // ---- The side-face audit.
    //
    // Tenebris's `build_tile` has one rule for a side face: it is drawn
    // wherever a solid voxel meets a neighbour that does not cover it, at the
    // same depth. That is the SPEC. What this renderer draws on a side is two
    // passes' worth of bands - the terrain wall and the column flanks - and
    // every hole and every double wall so far has been those two passes
    // disagreeing about who owns a band. So the audit computes both: the bands
    // the reference says must be there, from the columns alone, and the bands
    // the shader's own arithmetic yields, transcribed from `planet_surface.wgsl`
    // (the run words, `column_gap`, the clamps), and holds them equal on every
    // shared side of the real tier, before and after edits.
    //
    // A transcription rather than the artifact, and said so: the vertex shader
    // cannot be run here. The constants it uses are pinned against the shipped
    // WGSL by `the_shader_carries_the_reference_light_constants`; the RULE is
    // pinned by this, and a change to one without the other is what these two
    // tests exist to fail on.

    use pbd_core::column::{BASE_M, LAYERS, layer_altitude};

    const TOP_M: f32 = BASE_M as f32 + LAYERS as f32;

    fn run_present(word: u32) -> bool {
        (word >> 9) & 0x1ff != 0
    }
    fn run_lo(word: u32) -> f32 {
        BASE_M as f32 + (word & 0x1ff) as f32
    }
    fn run_hi(word: u32) -> f32 {
        BASE_M as f32 + ((word >> 9) & 0x1ff) as f32
    }
    fn column_side(rec: &GpuColumn, side: usize) -> u32 {
        if side < 4 {
            rec.neighbors[side]
        } else {
            rec.more[side - 4]
        }
    }
    /// `column_gap` in the shader: the `g`th stretch of air in a neighbouring
    /// column, bottom up, zero-sized past the last.
    fn column_gap(records: &[GpuColumn], neighbor: u32, g: usize) -> (f32, f32) {
        if neighbor == NO_NEIGHBOR {
            return (0.0, 0.0);
        }
        let rec = &records[neighbor as usize];
        let count = rec.runs.iter().filter(|w| run_present(**w)).count();
        if g > count {
            return (0.0, 0.0);
        }
        let lo = if g > 0 {
            run_hi(rec.runs[g - 1])
        } else {
            BASE_M as f32
        };
        let hi = if g < count {
            run_lo(rec.runs[g])
        } else {
            TOP_M
        };
        (lo, hi)
    }

    /// The bands the shader draws on `side` of record `index`: the terrain
    /// wall where the neighbour has no column, the flanks where it has one.
    /// This is `planet_surface.wgsl`'s vertex branch for kinds 1 and 4, in
    /// Rust, and it must be changed WITH it.
    fn drawn_bands(set: &lod::FineSet, index: usize, side: usize) -> Vec<(f32, f32)> {
        let finest = set.finest_records();
        let cell = &finest[index];
        let height = cell.direction_height[3];
        let neighbor_cap = cell.corners[side][3];
        let slot = set.columns.slots[index];
        let records = set.columns.gpu_records();
        let mut bands = Vec::new();
        let rec = &records[slot];
        let across = column_side(rec, side);
        if across == NO_NEIGHBOR {
            // The terrain wall: from our cap down to the neighbour's.
            if height > neighbor_cap {
                bands.push((neighbor_cap, height));
            }
            return bands;
        }
        for &word in &rec.runs {
            if !run_present(word) {
                continue;
            }
            let lo = run_lo(word);
            let hi = run_hi(word);
            for g in 0..=MAX_RUNS {
                let (gap_lo, gap_hi) = column_gap(records, across, g);
                let bottom = lo.max(gap_lo);
                let top = hi.min(gap_hi);
                if top - bottom > 0.001 {
                    bands.push((bottom, top));
                }
            }
        }
        bands
    }

    /// The bands the reference's rule requires on `side` of record `index`:
    /// every layer where this column is solid and the neighbour's is not.
    fn exposed_bands(set: &lod::FineSet, index: usize, side: usize) -> Vec<(f32, f32)> {
        let finest = set.finest_records();
        let cell = &finest[index];
        let height = cell.direction_height[3];
        let neighbor_cap = cell.corners[side][3];
        let column = set.columns.column(index).expect("a tier cell");
        let neighbor = set.finest_neighbors[index][side];
        let Some(other) = set.columns.column(neighbor as usize) else {
            // Off the tier: solid below its cap, as the heightfield assumes.
            return if height > neighbor_cap {
                vec![(neighbor_cap, height)]
            } else {
                vec![]
            };
        };
        (0..LAYERS)
            .filter(|&layer| column.solid(layer) && !other.solid(layer))
            .map(|layer| (layer_altitude(layer), layer_altitude(layer) + 1.0))
            .collect()
    }

    /// A column's top and its record's height are ONE number: both read
    /// `column::surface_m`. A column standing a layer proud of its cap is the
    /// invisible metre of rock that the aim ray dug first, that a first edit
    /// reconciled the cap UP into, and that both wall rules had to dodge.
    #[test]
    fn every_column_top_is_its_records_height() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let finest = set.finest_records();
        let mut checked = 0;
        for (index, &slot) in set.columns.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let top = set.columns.columns[slot]
                .surface()
                .expect("bedrock at least");
            let top_m = layer_altitude(top) + 1.0;
            assert_eq!(
                top_m, finest[index].direction_height[3],
                "cell {index}: the column's top and the record's height disagree"
            );
            checked += 1;
        }
        assert!(checked > 3_000, "only {checked} cells checked");
    }

    /// Sorted, merged, so two lists that cover the same metres compare equal
    /// however they were cut.
    fn merged(mut bands: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
        bands.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut out: Vec<(f32, f32)> = Vec::new();
        for (lo, hi) in bands {
            match out.last_mut() {
                Some(last) if lo <= last.1 + 1e-3 => last.1 = last.1.max(hi),
                _ => out.push((lo, hi)),
            }
        }
        out
    }

    /// A band drawn twice is a double wall, which is a picture (a slab standing
    /// in a meadow) before it is a number.
    fn overlaps(bands: &[(f32, f32)]) -> bool {
        let mut sorted = bands.to_vec();
        sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
        sorted.windows(2).any(|w| w[1].0 < w[0].1 - 1e-3)
    }

    fn audit_side(set: &lod::FineSet, index: usize, side: usize, what: &str) {
        let drawn = drawn_bands(set, index, side);
        assert!(
            !overlaps(&drawn),
            "{what}: cell {index} side {side} draws a band twice: {drawn:?}"
        );
        let drawn = merged(drawn);
        let exposed = merged(exposed_bands(set, index, side));
        assert_eq!(
            drawn.len(),
            exposed.len(),
            "{what}: cell {index} side {side} draws {drawn:?} where the reference exposes {exposed:?}"
        );
        for (d, e) in drawn.iter().zip(&exposed) {
            assert!(
                (d.0 - e.0).abs() < 1e-3 && (d.1 - e.1).abs() < 1e-3,
                "{what}: cell {index} side {side} draws {drawn:?} where the reference exposes {exposed:?}"
            );
        }
    }

    fn audit_cell(set: &lod::FineSet, index: usize, what: &str) {
        let degree = set.finest_records()[index].degree();
        for side in 0..degree {
            audit_side(set, index, side, what);
        }
    }

    /// An edit as `apply_edit` makes it, without the hands and the save.
    fn edit(set: &mut lod::FineSet, record: usize, layer: usize, material: Material) {
        assert!(set.columns.set_layer(record, layer, material));
        set.columns.repack(record);
        for &neighbor in set.finest_neighbors[record].iter() {
            if neighbor != u32::MAX {
                set.columns.repack(neighbor as usize);
            }
        }
        set.reconcile(record);
    }

    /// A cell three tiles inside the tier, on dry land, with a column on every
    /// side and a surface at least three layers above the bedrock's worth of
    /// soil: the ordinary cell every edit test starts from.
    fn interior_cell(set: &lod::FineSet, anchor: Vec3) -> usize {
        let finest = set.finest_records();
        let reach = ColumnSettings::default().reach_m;
        let inner = (reach - 4.0 * lod::tile_width_m(lod::FINEST_LEVEL)) / PLANET_RADIUS;
        (0..finest.len())
            .find(|&index| {
                let cell = &finest[index];
                let direction = Vec3::from_slice(&cell.direction_height[..3]);
                direction.dot(anchor).clamp(-1., 1.).acos() < inner
                    && cell.direction_height[3] > 5.0
                    && set
                        .columns
                        .column(index)
                        .is_some_and(|c| c.drawn_runs().len() == 1)
                    && set.finest_neighbors[index]
                        .iter()
                        .take(cell.degree())
                        .all(|&n| n != u32::MAX && set.columns.column(n as usize).is_some())
            })
            .expect("an ordinary interior cell")
    }

    /// Every shared side of the tier as it is built - which includes every
    /// mouth the carve opened - draws exactly the bands the reference exposes.
    #[test]
    fn every_side_of_the_tier_draws_what_the_reference_exposes() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let mut sides = 0;
        for (index, &slot) in set.columns.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            audit_cell(&set, index, "the built tier");
            sides += set.finest_records()[index].degree();
        }
        assert!(sides > 10_000, "only {sides} sides audited");
    }

    /// The pit, the tunnel and the block: a dig from the top, a dig into a
    /// SIDE that leaves the surface where it was, a dig that opens a column
    /// into its neighbour's cave, and a block placed back. After each, the
    /// edited cell and every neighbour draw exactly what is exposed.
    #[test]
    fn a_dug_or_placed_cell_and_its_neighbours_draw_what_is_exposed() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let index = interior_cell(&set, anchor);
        let top = set.columns.column(index).unwrap().surface().unwrap();
        let ring: Vec<usize> = set.finest_neighbors[index]
            .iter()
            .take(set.finest_records()[index].degree())
            .map(|&n| n as usize)
            .collect();
        let audit = |set: &lod::FineSet, what: &str| {
            audit_cell(set, index, what);
            for &n in &ring {
                audit_cell(set, n, what);
            }
        };
        // A two-layer pit from the top.
        edit(&mut set, index, top, Material::Air);
        audit(&set, "one layer dug from the top");
        edit(&mut set, index, top - 1, Material::Air);
        audit(&set, "two layers dug from the top");
        // The neighbour's SIDE, two layers under its own unchanged cap: the
        // player standing in the pit digging sideways. This is the case where
        // the neighbour's rock no longer fills the step in one run.
        let side_cell = ring[0];
        let side_top = set.columns.column(side_cell).unwrap().surface().unwrap();
        edit(&mut set, side_cell, side_top - 1, Material::Air);
        audit(&set, "a layer dug into a neighbour's side");
        edit(&mut set, side_cell, side_top - 2, Material::Air);
        audit(&set, "two layers dug into a neighbour's side");
        // And a block placed back on the pit's floor.
        edit(&mut set, index, top - 1, Material::Stone);
        audit(&set, "a block placed in the pit");
        // Then one on top of the neighbour, standing proud of the meadow.
        edit(&mut set, side_cell, side_top + 1, Material::Stone);
        audit(&set, "a block placed on the meadow");
    }

    /// A dig past the tier's edge lands: the cell just outside the rim is
    /// adopted, the layer is taken, its resident neighbours name it and it
    /// names them, and every side of it and of them draws what is exposed.
    #[test]
    fn a_dig_past_the_tiers_edge_adopts_the_column_and_lands() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let before = set.columns.columns.len();
        let index = (0..set.finest_records().len())
            .find(|&index| {
                set.columns.column(index).is_none()
                    && set.finest_records()[index].direction_height[3] > 5.0
                    && set.finest_neighbors[index]
                        .iter()
                        .take(set.finest_records()[index].degree())
                        .any(|&n| n != u32::MAX && set.columns.column(n as usize).is_some())
            })
            .expect("a dry cell just outside the tier");
        let column = set.adoptable(index, &Edits::new()).unwrap();
        let top = column.surface().unwrap();
        assert_eq!(
            layer_altitude(top) + 1.0,
            set.finest_records()[index].direction_height[3],
            "the adopted column's top is the cap the player aimed at"
        );
        assert!(set.adopt(index, column));
        assert!(
            !set.adopt(index, Column::bedrock()),
            "a column is adopted once"
        );
        assert_eq!(set.columns.columns.len(), before + 1);
        assert_eq!(slot_of(&set.finest_records()[index]), Some(before as u32));
        edit(&mut set, index, top, Material::Air);
        assert!(!set.columns.column(index).unwrap().solid(top));
        let slot = set.columns.slots[index];
        let mut named = 0;
        for (side, &neighbor) in set.finest_neighbors[index]
            .iter()
            .enumerate()
            .take(set.finest_records()[index].degree())
        {
            let Some(&there) = set.columns.slots.get(neighbor as usize) else {
                continue;
            };
            if there == usize::MAX {
                continue;
            }
            assert_eq!(column_side(&set.columns.records[slot], side), there as u32);
            let back = set.finest_neighbors[neighbor as usize]
                .iter()
                .position(|&id| id as usize == index)
                .unwrap();
            assert_eq!(
                column_side(&set.columns.records[there], back),
                slot as u32,
                "the neighbour names the adopted column back"
            );
            audit_cell(&set, neighbor as usize, "a neighbour of an adopted column");
            named += 1;
        }
        assert!(named > 0);
        audit_cell(&set, index, "an adopted column");
        set.columns.relight();
        assert_eq!(set.columns.gpu_light().len(), (before + 1) * LIGHT_WORDS);
        assert_eq!(
            set.columns.sky(slot, top),
            light::MAX,
            "the pit is open to the sky"
        );
    }

    /// The material buffer says what every layer is, and a stone placed in a
    /// column reads back as stone at exactly that layer, with the layers
    /// around it what they were: the face drawn from it wears the voxel.
    #[test]
    fn a_placed_stone_reads_back_as_stone_at_its_own_layer() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let index = interior_cell(&set, anchor);
        let top = set.columns.column(index).unwrap().surface().unwrap();
        edit(&mut set, index, top + 1, Material::Stone);
        edit(&mut set, index, top + 2, Material::Stone);
        let slot = set.columns.slots[index];
        let words = set.columns.gpu_materials();
        let code_at = |layer: usize| {
            (words[slot * MATERIAL_WORDS + layer / MATERIAL_PER_WORD]
                >> ((layer % MATERIAL_PER_WORD) * 4))
                & 0xf
        };
        assert_eq!(code_at(top + 1), render_code(Material::Stone));
        assert_eq!(code_at(top + 2), render_code(Material::Stone));
        assert_eq!(
            code_at(top),
            render_code(set.columns.column(index).unwrap().material(top))
        );
        assert_eq!(code_at(top + 3), render_code(Material::Air));
        assert_eq!(words.len(), set.columns.columns.len() * MATERIAL_WORDS);
    }

    /// A pit dug from the top is OPEN to the sky, so its floor is at full
    /// daylight the moment it is re-baked: the field's own sky seed walks every
    /// column down to its first solid layer, and a dug layer is not one.
    #[test]
    fn a_pit_dug_from_the_top_is_lit_by_the_sky_once_rebaked() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let index = interior_cell(&set, anchor);
        let top = set.columns.column(index).unwrap().surface().unwrap();
        for layer in [top, top - 1, top - 2] {
            edit(&mut set, index, layer, Material::Air);
        }
        set.columns.relight();
        let slot = set.columns.slots[index];
        for layer in [top, top - 1, top - 2] {
            assert_eq!(
                set.columns.sky(slot, layer),
                light::MAX,
                "layer {layer} of the pit should be at full sky"
            );
        }
        // And the floor it stands on, which is rock, holds no light of its
        // own - the face above it samples the air, not the rock.
        assert_eq!(set.columns.sky(slot, top - 3), 0);
    }

    /// A report: every cell within a few metres of the default spawn, with
    /// its record height against its column's top, for reading a picture of
    /// the spawn against the numbers that drew it.
    #[test]
    #[ignore = "a report: cargo test -p pbd-app --lib spawn_neighbourhood -- --ignored --nocapture"]
    fn spawn_neighbourhood() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let finest = set.finest_records();
        for (index, cell) in finest.iter().enumerate() {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            let m = direction.dot(anchor).clamp(-1., 1.).acos() * PLANET_RADIUS;
            if m > 7.0 {
                continue;
            }
            let Some(column) = set.columns.column(index) else {
                continue;
            };
            let top = column.surface().map(|t| layer_altitude(t) + 1.0);
            let walls: Vec<f32> = (0..cell.degree()).map(|s| cell.corners[s][3]).collect();
            println!(
                "cell {index} key {} at {m:.1} m: height {:.2}, column top {top:?}, runs {:?}, gen {:.2}, walls {walls:?}",
                cell.key(),
                cell.direction_height[3],
                column
                    .drawn_runs()
                    .iter()
                    .map(|r| (r.from, r.to))
                    .collect::<Vec<_>>(),
                crate::planet::surface_height(direction),
            );
        }
    }

    /// A street lantern is dark by day and lit at night, and a torch beside
    /// it burns at both (`lamps-and-lanterns` task 4.2).
    #[test]
    fn a_dusk_lit_lantern_lights_only_at_night_and_a_torch_always() {
        // A line of twenty columns of flat ground, a street lantern standing
        // in the first and a torch in the last, out of each other's reach.
        const N: u32 = 20;
        let mut columns: Vec<Column> = (0..N)
            .map(|_| {
                let mut column = Column::bedrock();
                for layer in 1..=40 {
                    column.set(layer, Material::Stone);
                }
                column
            })
            .collect();
        columns[0].set(41, Material::LanternPost);
        columns[N as usize - 1].set(41, Material::Torch);
        let records = (0..N)
            .map(|i| GpuColumn {
                runs: columns[i as usize].packed_runs(render_code),
                neighbors: [
                    if i + 1 < N { i + 1 } else { NO_NEIGHBOR },
                    if i > 0 { i - 1 } else { NO_NEIGHBOR },
                    NO_NEIGHBOR,
                    NO_NEIGHBOR,
                ],
                more: [
                    NO_NEIGHBOR,
                    NO_NEIGHBOR,
                    6,
                    state_word(&columns[i as usize], false, false),
                ],
            })
            .collect();
        let mut tier = ColumnTier {
            columns,
            slots: (0..N as usize).collect(),
            records,
            light: Vec::new(),
            dusk: false,
            glow: vec![false; N as usize],
            centres: vec![Vec3::Y; N as usize],
            sides: Vec::new(),
        };
        tier.relight();
        assert_eq!(tier.light_at(0, 41).block(), 0, "the lantern is out by day");
        assert_eq!(
            tier.light_at(N as usize - 1, 41).block(),
            14,
            "the torch burns"
        );
        assert_eq!(tier.records[0].more[3] & LAMP_LIT_BIT, 0, "its glass dark");
        assert_ne!(tier.records[N as usize - 1].more[3] & LAMP_LIT_BIT, 0);
        tier.set_dusk(true);
        assert_eq!(tier.light_at(0, 41).block(), 13, "lit at night");
        assert_ne!(
            tier.records[0].more[3] & LAMP_LIT_BIT,
            0,
            "its glass alight"
        );
        assert_eq!(
            tier.light_at(1, 41).block(),
            13 - light::ACROSS,
            "and lights its street"
        );
        assert_eq!(
            tier.light_at(N as usize - 1, 41).block(),
            14,
            "the torch burns still"
        );
        tier.set_dusk(false);
        assert_eq!(tier.light_at(0, 41).block(), 0, "out again at dawn");
    }

    /// Over every cell of a real tier the three answers about a glowing
    /// flower agree (`lamps-and-lanterns` task 6.1): the record's bit is the
    /// key's choice, the tier's flag is the record's bit, and at night a slot
    /// is an emitter exactly where its column record says the flower burns.
    #[test]
    fn a_tiers_glowing_flowers_agree_between_record_bake_and_bit() {
        // The desktop's spawn: a meadow, where flowers grow.
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let settings = ColumnSettings::default();
        let mut set = lod::generate_fine(anchor, &settings, &Edits::new());
        set.columns.set_dusk(true);
        let emitting: std::collections::HashSet<u32> = set
            .columns
            .emitters()
            .iter()
            .filter(|emitter| emitter.level == flora::GLOW_FLOWER_LEVEL)
            .map(|emitter| emitter.column)
            .collect();
        let (mut chosen, mut burning) = (0, 0);
        for (index, cell) in set.finest_records().iter().enumerate() {
            let key = cell.key();
            let bit = glows(cell);
            assert_eq!(
                bit,
                key != crate::planet::NO_KEY && flora::glows(key, settings.glow_flower_chance),
                "record {index}: the bit is the key's choice"
            );
            chosen += bit as usize;
            let slot = set.columns.slots[index];
            if slot == usize::MAX {
                continue;
            }
            assert_eq!(
                set.columns.glow[slot], bit,
                "slot {slot}: the tier keeps the bit"
            );
            let lit = set.columns.records[slot].more[3] & GLOW_LIT_BIT != 0;
            assert_eq!(
                lit,
                emitting.contains(&(slot as u32)),
                "slot {slot}: the record burns exactly where the bake lights"
            );
            burning += lit as usize;
        }
        assert!(chosen > 50, "only {chosen} cells chose a glowing flower");
        assert!(burning > 0, "and none of the tier's burned at night");
    }

    /// The tier's sampler (`lamps-and-lanterns` task 3.2): the eight corners
    /// of a box buried in the rock under the spawn meadow read dark, the
    /// same box lifted into the open air reads the sky, and a point off the
    /// tier reads the open sky.
    #[test]
    fn a_sealed_cave_gives_eight_dark_corners_and_the_open_air_the_sky() {
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = &set.columns;
        let record = (0..set.finest_records().len())
            .find(|&index| {
                tier.slots[index] != usize::MAX
                    && Vec3::from_slice(&set.finest_records()[index].direction_height[..3])
                        .dot(anchor)
                        > 0.999_999
            })
            .expect("a column at the anchor");
        let cell = &set.finest_records()[record];
        let centre = Vec3::from_slice(&cell.direction_height[..3]);
        let ground = column::layer_altitude(tier.column(record).unwrap().surface().unwrap()) + 1.0;
        let corners = |altitude: f32| -> Vec<(f32, f32)> {
            let east = centre.cross(Vec3::Y).normalize_or(Vec3::X);
            let north = east.cross(centre);
            [-0.4_f32, 0.4]
                .iter()
                .flat_map(|&a| [-0.4_f32, 0.4].map(move |b| (a, b)))
                .flat_map(|(a, b)| [altitude, altitude + 0.8].map(move |h| (a, b, h)))
                .map(|(a, b, h)| {
                    let direction = (centre + (east * a + north * b) / PLANET_RADIUS).normalize();
                    tier.sample(Some(record), direction, h)
                })
                .collect()
        };
        let buried = corners(ground - 20.0);
        assert_eq!(buried.len(), 8);
        assert!(
            buried
                .iter()
                .all(|&(sky, block)| sky == 0.0 && block == 0.0),
            "twenty metres under the meadow: {buried:?}"
        );
        let open = corners(ground + 2.0);
        assert!(
            open.iter().all(|&(sky, block)| sky > 0.99 && block == 0.0),
            "two metres over it: {open:?}"
        );
        assert_eq!(
            tier.sample(None, centre, ground + 2.0),
            (1.0, 0.0),
            "off the tier"
        );
    }

    /// A light placed in the tier lights at once, and taken back leaves the
    /// field exactly as it was (`lamps-and-lanterns`, "A light is placed and
    /// lights at once" and "A light is taken back"). The same three steps the
    /// edit path takes: write the layer, repack the record, relight the tier.
    #[test]
    fn a_lantern_placed_lights_at_once_and_taken_back_leaves_the_field_as_it_was() {
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = &mut set.columns;
        tier.set_dusk(true);
        let record = (0..tier.slots.len())
            .find(|&index| tier.slots[index] != usize::MAX && tier.column(index).is_some())
            .expect("a column");
        let slot = tier.slots[record];
        let top = tier.column(record).unwrap().surface().unwrap() + 1;
        let before: Vec<light::Light> = (0..tier.columns.len())
            .flat_map(|s| (0..LAYERS).map(move |l| (s, l)))
            .map(|(s, l)| tier.light_at(s, l))
            .collect();
        for lamp in Material::LAMPS {
            assert!(tier.set_layer(record, top, lamp), "{lamp:?} placed");
            tier.repack(record);
            tier.relight();
            assert_eq!(
                tier.light_at(slot, top).block(),
                lamp.emission(),
                "{lamp:?} lights its own cell at once"
            );
            assert_ne!(
                tier.records[slot].more[3] >> TORCH_SHIFT & LAMP_LAYER_MASK,
                0
            );
            assert!(
                tier.set_layer(record, top, Material::Air),
                "{lamp:?} taken back"
            );
            tier.repack(record);
            tier.relight();
            let after: Vec<light::Light> = (0..tier.columns.len())
                .flat_map(|s| (0..LAYERS).map(move |l| (s, l)))
                .map(|(s, l)| tier.light_at(s, l))
                .collect();
            assert!(before == after, "{lamp:?}: the field is as it was");
            assert_eq!(
                tier.records[slot].more[3] >> TORCH_SHIFT & LAMP_LAYER_MASK,
                0
            );
        }
    }

    /// A glowing flower (`lamps-and-lanterns` task 6.1): a cell the key chose
    /// lights the air over its sod from dusk to dawn, its record says it
    /// burns exactly while the field has it as an emitter, and digging the
    /// sod out puts it out. A cell the key did not choose never lights.
    #[test]
    fn a_glowing_flower_lights_its_sod_at_night_and_is_put_out_by_digging_it() {
        const N: usize = 3;
        let columns: Vec<Column> = (0..N)
            .map(|_| {
                let mut column = Column::bedrock();
                for layer in 1..40 {
                    column.set(layer, Material::Stone);
                }
                column.set(40, Material::Grass);
                column
            })
            .collect();
        let glow = vec![true, false, false];
        let records = (0..N)
            .map(|i| GpuColumn {
                runs: columns[i].packed_runs(render_code),
                neighbors: [
                    if i + 1 < N { i as u32 + 1 } else { NO_NEIGHBOR },
                    if i > 0 { i as u32 - 1 } else { NO_NEIGHBOR },
                    NO_NEIGHBOR,
                    NO_NEIGHBOR,
                ],
                more: [
                    NO_NEIGHBOR,
                    NO_NEIGHBOR,
                    6,
                    state_word(&columns[i], false, glow[i]),
                ],
            })
            .collect();
        let mut tier = ColumnTier {
            columns,
            slots: (0..N).collect(),
            records,
            light: Vec::new(),
            dusk: false,
            glow,
            centres: vec![Vec3::Y; N],
            sides: Vec::new(),
        };
        tier.relight();
        let emits = |tier: &ColumnTier, slot: u32| {
            tier.emitters().iter().any(|emitter| emitter.column == slot)
        };
        assert!(!emits(&tier, 0), "by day it is a flower");
        assert_eq!(tier.light_at(0, 41).block(), 0);
        assert_eq!(tier.records[0].more[3] & GLOW_LIT_BIT, 0);

        tier.set_dusk(true);
        assert!(emits(&tier, 0), "at night it is a light");
        assert_eq!(
            tier.light_at(0, 41).block(),
            flora::GLOW_FLOWER_LEVEL,
            "in the air over its sod"
        );
        assert_ne!(tier.records[0].more[3] & GLOW_LIT_BIT, 0, "and says so");
        for slot in 1..N {
            assert!(!emits(&tier, slot as u32), "a cell it was not chosen for");
            assert_eq!(tier.records[slot].more[3] & GLOW_LIT_BIT, 0);
        }
        // The record and the emitters agree for every slot.
        for slot in 0..N {
            assert_eq!(
                tier.records[slot].more[3] & GLOW_LIT_BIT != 0,
                emits(&tier, slot as u32),
                "slot {slot}"
            );
        }

        // Dig the sod out: no ground for it, so no flower and no light.
        tier.set_layer(0, 40, Material::Air);
        tier.repack(0);
        tier.relight();
        assert!(!emits(&tier, 0), "dug up with its sod");
        assert_eq!(tier.records[0].more[3] & GLOW_LIT_BIT, 0);
        assert_eq!(tier.light_at(0, 41).block(), 0);
    }
}
