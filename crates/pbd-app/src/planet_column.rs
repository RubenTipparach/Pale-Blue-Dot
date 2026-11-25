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
use super::terrain::{PLANET_RADIUS, render_code, terrain_config, tileset_slot};
use crate::config::ColumnSettings;
use bevy::prelude::Vec3;
use bytemuck::{Pod, Zeroable};
use pbd_core::column::{self, Column, LAYERS, MAX_RUNS};
use pbd_core::edits::Edits;
use pbd_core::flora;
use pbd_core::light;
use pbd_core::liquid::{self, Cell as LiquidCell, Grid};
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
/// How far the top water layer's surface stands under that layer's top, in
/// eighths of a metre, three bits above the water's layer: nought for a
/// source, more for a flowing cell's lower level (`liquid::top_fraction`).
/// So the terrain's underwater tint stops at the water's own surface, not a
/// whole layer over a thin film (owner, 2026-11-16: "underwater ground tint
/// is happening before the water line").
pub const WATER_SHORT_SHIFT: u32 = 13;
pub const WATER_SHORT_MASK: u32 = 0x7;
const _: () = assert!(LAYERS < WATER_MASK as usize);
const _: () = assert!(WATER_SHIFT + 9 <= WATER_SHORT_SHIFT);
const _: () = assert!(WATER_SHORT_SHIFT + 3 <= TORCH_SHIFT);

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
/// Where the placed sheets start in the material buffer, in words: after
/// every slot's materials ([`ColumnTier::gpu_placed_sheets`]).
pub const PLACED_SHEETS_BASE: usize = COLUMN_CAPACITY as usize * MATERIAL_WORDS;

/// One column's render codes, eight to a word, into `out` (`MATERIAL_WORDS`
/// words): one slot of [`ColumnTier::gpu_materials`].
pub(crate) fn pack_materials(column: &Column, out: &mut [u32]) {
    out.fill(0);
    for layer in 0..LAYERS {
        let shift = (layer % MATERIAL_PER_WORD) * 4;
        // As drawn: a drowned top is a bed (`flora-v2` 12.10).
        out[layer / MATERIAL_PER_WORD] |=
            (render_code(column.drawn_material(layer)) & 0xf) << shift;
    }
}

/// One column's placed sheets into `out`: one slot of
/// [`ColumnTier::gpu_placed_sheets`], zeroes for a column nobody built on
/// and with no ore in it ([`drawn_sheet_mark`]).
pub(crate) fn pack_placed_sheets(column: &Column, out: &mut [u32]) {
    out.fill(0);
    for layer in 0..LAYERS {
        let sheet = drawn_sheet_mark(column, layer);
        if sheet != 0 {
            out[layer / MATERIAL_PER_WORD] |= sheet << ((layer % MATERIAL_PER_WORD) * 4);
        }
    }
}

/// A record as the terrain shader reads it, its reach top in `more[2]`
/// ([`ColumnTier::gpu_records_with_reach`]).
pub(crate) fn with_reach_top(record: &GpuColumn, top: u16) -> GpuColumn {
    let mut out = *record;
    out.more[2] = (record.more[2] & DEGREE_MASK) | u32::from(top) << REACH_TOP_SHIFT;
    out
}

/// The sheet a layer is drawn from if somebody PLACED it, plus one, or
/// nought for generated ground (`placed-blocks-keep-material`): the tileset
/// of the block's own home ([`column::home_biome`]), never the biome it
/// stands in. A nibble, as the materials are.
pub fn placed_sheet(column: &Column, layer: usize) -> u32 {
    if !column.placed(layer) {
        return 0;
    }
    let material = column.material(layer);
    if let Some(own) = column::own_sheet(material) {
        return super::terrain::sheet_index(own) + 1;
    }
    column::home_biome(material).map_or(0, |home| tileset_slot(home) + 1)
}
// Every sheet, plus one, fits its nibble, and so the record's four bits.
const _: () = assert!(super::terrain::TILESETS.len() < 16);

/// What the per-layer sheet field and the record's sheet bits carry for a
/// layer: [`ORE_SHEET`] for ore, generated in a vein or placed (`flora-v2`
/// design 12.12), else [`placed_sheet`]. The ore keeps the stone's render
/// code; the mark is what the shader draws its flecks from, on the stone of
/// the biome it stands in.
///
/// [`ORE_SHEET`]: super::terrain::ORE_SHEET
pub fn drawn_sheet_mark(column: &Column, layer: usize) -> u32 {
    let material = column.material(layer);
    if material == Material::Ore {
        return super::terrain::ORE_SHEET;
    }
    // A sea cell is drawn from its own sheet, generated or placed
    // (`ocean-environments` B24).
    if let Some(own) = column::own_sheet(material) {
        return super::terrain::sheet_index(own) + 1;
    }
    placed_sheet(column, layer)
}

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
    /// How many times the field has been baked: what a town's outside,
    /// baked from it vertex by vertex, watches to bake again
    /// (`cities-in-the-world`, "Finding: the stair towers have no light").
    relit: u64,
    /// Every light the last bake lit from, where it stands: what the new
    /// lighting model's highlights, reflections and rain glow read
    /// (`lighting-model`), so they find the same lamps the field does.
    lamps: Vec<LampSpot>,
    /// The water's queue and what it needs to name a slot's cell
    /// (`water-flow` E3).
    pub water: TierWater,
}

/// The tier's water (`water-flow` E3, `volumetric-water` phase 2): the
/// rule's queue, and each slot's cell key and finest record, which a change
/// the rule makes is saved and drawn by.
#[derive(Clone, Default)]
pub struct TierWater {
    pub queue: liquid::Queue,
    /// Each slot's exact cell key.
    pub keys: Vec<u32>,
    /// Each slot's finest record.
    pub records: Vec<usize>,
    /// Each slot: whether a player's edit put something other than water in
    /// its column (dug it, or built in it). The sea's reach never runs down
    /// through such a column's air under the sea's level (`dry-holes`
    /// design 3). Out of range reads as not dug.
    pub dug: Vec<bool>,
    /// The cells under the sea's level the sea is joined to through what is
    /// not solid, `slot * sea_layer + layer` (`dry-holes` design 4): where
    /// the rule's water is the sea's. Empty until the tier's sea is marked
    /// (so a build's river settle runs on the rule alone), and again after
    /// every edit.
    pub joined: Vec<bool>,
}

/// Layers a slot's water ids are spaced by: a cell is `slot * 512 + layer`.
pub const WATER_ID_LAYERS: u32 = 512;
const _: () = assert!(LAYERS < WATER_ID_LAYERS as usize);

/// The id `pbd_core::liquid` names a slot's layer by.
pub fn water_id(slot: usize, layer: usize) -> u32 {
    slot as u32 * WATER_ID_LAYERS + layer as u32
}

/// The slot and layer of a water id.
pub fn water_cell(id: u32) -> (usize, usize) {
    (
        (id / WATER_ID_LAYERS) as usize,
        (id % WATER_ID_LAYERS) as usize,
    )
}

/// The first layer at or above sea level: the flood fills only below it.
pub fn sea_layer() -> usize {
    column::layer_at(terrain_config().sea_level_m).unwrap_or(0)
}

/// Whether the edits put water in this cell's layer: a source a player
/// poured, or water the rule wrote. Neither seeds the sea's flood.
fn edited_water(edits: &Edits, key: u32, layer: usize) -> bool {
    edits
        .for_cell(key)
        .iter()
        .any(|&(l, m)| usize::from(l) == layer && m.is_water())
}

/// The tier as the water rule's grid: a slot's layer, the rim and every cell
/// off the tier neutral.
pub struct TierGrid<'a>(pub &'a mut ColumnTier);

impl ColumnTier {
    pub(crate) fn rim(&self, slot: usize) -> bool {
        self.records
            .get(slot)
            .is_none_or(|record| record.more[3] & RIM_BIT != 0)
    }

    /// A slot's neighbours across its sides, `None` off the tier.
    /// How many sides the cell at `slot` has: 6, or 5 at a pentagon.
    pub fn degree_of(&self, slot: usize) -> usize {
        self.records
            .get(slot)
            .map_or(6, |record| ((record.more[2] & DEGREE_MASK) as usize).min(6))
    }

    pub fn sides_of(&self, slot: usize) -> [Option<usize>; 6] {
        let mut out = [None; 6];
        let (Some(record), Some(sides)) = (self.records.get(slot), self.sides.get(slot)) else {
            return out;
        };
        let degree = ((record.more[2] & DEGREE_MASK) as usize).min(6);
        for (o, &s) in out.iter_mut().zip(sides).take(degree) {
            if s != NO_NEIGHBOR && (s as usize) < self.columns.len() {
                *o = Some(s as usize);
            }
        }
        out
    }

    fn water_at(&self, id: u32) -> LiquidCell {
        let (slot, layer) = water_cell(id);
        if layer == 0 || layer >= LAYERS || self.rim(slot) {
            return LiquidCell::Neutral;
        }
        self.columns
            .get(slot)
            .map_or(LiquidCell::Neutral, |c| LiquidCell::of(c.material(layer)))
    }

    fn water_around(&self, id: u32) -> [Option<u32>; 8] {
        let (slot, layer) = water_cell(id);
        let mut out = [None; 8];
        if layer + 1 < LAYERS {
            out[liquid::UP] = Some(water_id(slot, layer + 1));
        }
        for (k, side) in self.sides_of(slot).into_iter().enumerate() {
            out[liquid::SIDE + k] = side.map(|s| water_id(s, layer));
        }
        if layer > 0 {
            out[liquid::DOWN] = Some(water_id(slot, layer - 1));
        }
        out
    }

    /// Repack one slot's record from its column.
    pub fn repack_slot(&mut self, slot: usize) {
        if let Some(&record) = self.water.records.get(slot) {
            self.repack(record);
        }
    }

    /// The sea's flood (`volumetric-water` phase 2) from `seeds`: every air
    /// cell below sea level that is joined to them across the sides or
    /// downward, inside the rim, filled as a source. Returns the slots it
    /// changed, each once.
    fn flood_from(&mut self, seeds: Vec<u32>) -> Vec<usize> {
        let sea = sea_layer();
        let rim: Vec<bool> = (0..self.columns.len()).map(|s| self.rim(s)).collect();
        let filled = liquid::flood(
            &mut TierGrid(self),
            seeds,
            |id| {
                let (slot, layer) = water_cell(id);
                layer < sea && !rim.get(slot).copied().unwrap_or(true)
            },
            FLOOD_LIMIT,
        );
        let mut slots: Vec<usize> = filled.into_iter().map(|id| water_cell(id).0).collect();
        slots.sort_unstable();
        slots.dedup();
        slots
    }

    /// Every generated water cell below sea level beside air the flood may
    /// fill: where a build's flood starts.
    fn sea_seeds(&self, edits: &Edits) -> Vec<u32> {
        let sea = sea_layer();
        let mut seeds = Vec::new();
        for slot in 0..self.columns.len() {
            if self.rim(slot) {
                continue;
            }
            let key = self.water.keys.get(slot).copied().unwrap_or(u32::MAX);
            let sides = self.sides_of(slot);
            for layer in 1..sea.min(LAYERS) {
                if self.columns[slot].material(layer) != Material::Water
                    || edited_water(edits, key, layer)
                {
                    continue;
                }
                let open = |s: usize, l: usize| {
                    !self.rim(s) && self.columns[s].material(l) == Material::Air
                };
                if open(slot, layer - 1) || sides.iter().flatten().any(|&s| open(s, layer)) {
                    seeds.push(water_id(slot, layer));
                }
            }
        }
        seeds
    }

    /// An edit at a finest record's layer, for the water (`dry-holes` design
    /// 4): the cell the edit opened fills with the sea at once if the sea's
    /// own water is over it or beside it under the sea's level, and the cell
    /// and its neighbours are queued for the rule (`water-flow` decision 5),
    /// which brings the sea in from there at its own pace. A column dug or
    /// built in is marked so (design 3). Returns the slots the edit's fill
    /// changed, which the caller repacks.
    pub fn water_after_edit(&mut self, record: usize, layer: usize, edits: &Edits) -> Vec<usize> {
        let Some(&slot) = self.slots.get(record) else {
            return Vec::new();
        };
        if slot == usize::MAX || slot >= self.columns.len() {
            return Vec::new();
        }
        if !self.columns[slot].material(layer).is_water() {
            if self.water.dug.len() <= slot {
                self.water.dug.resize(slot + 1, false);
            }
            self.water.dug[slot] = true;
        }
        let at = water_id(slot, layer);
        let sea = sea_layer();
        let around = self.water_around(at);
        // The sea's own water: generated, under its level, not the edits'.
        let sea_water = |id: u32| {
            let (s, l) = water_cell(id);
            let key = self.water.keys.get(s).copied().unwrap_or(u32::MAX);
            l < sea
                && !self.rim(s)
                && self.columns.get(s).map(|c| c.material(l)) == Some(Material::Water)
                && !edited_water(edits, key, l)
        };
        let opened = layer < sea
            && !self.rim(slot)
            && self.columns[slot].material(layer) == Material::Air
            && around[liquid::UP..liquid::DOWN]
                .iter()
                .flatten()
                .any(|&id| sea_water(id));
        let filled = if opened {
            // Derived, as the build's flood is: never saved, and a reload's
            // flood fills the same cell from the same water.
            self.columns[slot].set(layer, Material::Water);
            vec![slot]
        } else {
            Vec::new()
        };
        let mut queue = std::mem::take(&mut self.water.queue);
        queue.touch(&TierGrid(self), at);
        self.water.queue = queue;
        filled
    }

    /// Queue every cell of the tier that holds water the edits put there, and
    /// its neighbours: water still moving when the world was closed or the
    /// tier moved picks up, and a settled field changes nothing.
    pub fn queue_edited_water(&mut self, edits: &Edits) {
        let mut queue = std::mem::take(&mut self.water.queue);
        for slot in 0..self.columns.len() {
            let key = self.water.keys.get(slot).copied().unwrap_or(u32::MAX);
            for &(layer, material) in edits.for_cell(key) {
                if material.is_water() {
                    queue.touch(&TierGrid(self), water_id(slot, usize::from(layer)));
                }
            }
        }
        self.water.queue = queue;
    }

    /// Settle a version 8 world's generated fresh water by the rule (design
    /// 13, `water-flow`): every generated source above the sea with air
    /// beside or under it is queued, and the rule runs until nothing is (at
    /// most `RIVER_SETTLE_TICKS`), so what flows between a river's reaches,
    /// down its falls and over its riffles is exactly what the rule keeps.
    /// Derived, like the sea's flood: never saved, and the same tier settles
    /// the same way. A world of an older version is left as it was made.
    /// Returns how many cells the rule changed.
    pub fn settle_rivers(&mut self, edits: &Edits) -> usize {
        if terrain_config().rivers.is_none() {
            return 0;
        }
        let sea = sea_layer();
        // The settled water kept from earlier tiers (`river-cache` design
        // 11), stamped where this tier's rim cannot reach and no edit is
        // near; the rule then settles what nobody has kept yet.
        let cache = pbd_core::rivers::field(terrain_config()).filter(|_| settle_cache_on());
        let keep = cache.map(|_| self.settle_keepable(edits));
        if let (Some(cache), Some(keep)) = (cache, keep.as_ref()) {
            let mut stamped = Vec::new();
            for (slot, _) in keep.iter().enumerate().filter(|(_, k)| **k) {
                let key = self.water.keys.get(slot).copied().unwrap_or(u32::MAX);
                let Some(cells) = cache.settled.get(&key) else {
                    continue;
                };
                for &(layer, code) in cells.iter() {
                    if let Some(material) = pbd_core::river_cache::liquid_of(code).material() {
                        self.columns[slot].set(usize::from(layer), material);
                    }
                }
                stamped.push(slot);
            }
            for slot in stamped {
                self.repack_slot(slot);
            }
        }
        let mut queue = liquid::Queue::new();
        for slot in 0..self.columns.len() {
            if self.rim(slot) {
                continue;
            }
            let key = self.water.keys.get(slot).copied().unwrap_or(u32::MAX);
            for layer in sea.max(1)..LAYERS - 1 {
                if self.columns[slot].material(layer) != Material::Water
                    || edited_water(edits, key, layer)
                {
                    continue;
                }
                let id = water_id(slot, layer);
                let around = self.water_around(id);
                let open = around[liquid::SIDE..]
                    .iter()
                    .flatten()
                    .any(|&n| self.water_at(n) == LiquidCell::Air);
                if open {
                    queue.push(id);
                }
            }
        }
        #[cfg(test)]
        let queued = queue.len();
        let mut changed = Vec::new();
        let mut ticks = 0;
        #[cfg(test)]
        let mut sweeps = 0;
        // Settled, then swept: every water cell over the sea looked at again,
        // because the rule does not requeue the cell above one that changes
        // (Luanti's, kept), and a sweep that changes nothing is the field the
        // rule keeps. Two or three sweeps at most in practice.
        for sweep in 0..6 {
            #[cfg(test)]
            {
                sweeps = sweep + 1;
            }
            let before = changed.len();
            while !queue.is_empty() && ticks < RIVER_SETTLE_TICKS {
                queue.tick(
                    &mut TierGrid(self),
                    liquid::Rules::default(),
                    liquid::LOOP_MAX,
                    &mut changed,
                );
                ticks += 1;
            }
            if sweep > 0 && changed.len() == before {
                break;
            }
            for slot in 0..self.columns.len() {
                if self.rim(slot) {
                    continue;
                }
                for layer in sea.max(1)..LAYERS - 1 {
                    if self.columns[slot].material(layer).is_water() {
                        queue.push(water_id(slot, layer));
                    }
                }
            }
        }
        let mut slots: Vec<usize> = changed.iter().map(|&(id, _)| water_cell(id).0).collect();
        slots.sort_unstable();
        slots.dedup();
        for &slot in &slots {
            self.repack_slot(slot);
        }
        // Keep what the rule settled, where the rim could not reach: each
        // changed layer and the liquid it holds now, merged with what was
        // kept there before.
        if let (Some(cache), Some(keep)) = (cache, keep.as_ref()) {
            let mut layers: std::collections::BTreeMap<usize, Vec<u16>> = Default::default();
            for &(id, _) in &changed {
                let (slot, layer) = water_cell(id);
                if keep[slot] {
                    layers.entry(slot).or_default().push(layer as u16);
                }
            }
            for (slot, mut list) in layers {
                let key = self.water.keys.get(slot).copied().unwrap_or(u32::MAX);
                if key == u32::MAX {
                    continue;
                }
                if let Some(old) = cache.settled.get(&key) {
                    list.extend(old.iter().map(|&(l, _)| l));
                }
                list.sort_unstable();
                list.dedup();
                let cells: Box<[(u16, u8)]> = list
                    .into_iter()
                    .filter_map(|layer| {
                        let cell = LiquidCell::of(self.columns[slot].material(usize::from(layer)));
                        pbd_core::river_cache::liquid_code(cell).map(|c| (layer, c))
                    })
                    .collect();
                cache.settled.put(key, cells);
            }
        }
        #[cfg(test)]
        instrument(|p| {
            p.settle_queued = queued;
            p.settle_ticks = ticks;
            p.settle_sweeps = sweeps;
            p.settle_changed = changed.len();
        });
        changed.len()
    }

    /// Each slot's distance from the tier's rim in cells, counted to
    /// `limit` (slots at least that far in read `limit`): what says a cell
    /// was meshed against its real neighbours (`river-cache` design 14).
    pub(crate) fn rim_depth(&self, limit: u16) -> Vec<u16> {
        let n = self.columns.len();
        let mut depth = vec![limit; n];
        let mut front = std::collections::VecDeque::new();
        for (s, d) in depth.iter_mut().enumerate() {
            if self.rim(s) {
                *d = 0;
                front.push_back(s);
            }
        }
        while let Some(s) = front.pop_front() {
            let next = depth[s] + 1;
            if next >= limit {
                continue;
            }
            for t in self.sides_of(s).into_iter().flatten() {
                if depth[t] > next {
                    depth[t] = next;
                    front.push_back(t);
                }
            }
        }
        depth
    }

    /// Which slots' settled water may be stamped from the cache and kept in
    /// it (`river-cache` design 11): at least [`SETTLE_KEEP_CELLS`] from the
    /// tier's rim, where the rim could not have shaped it, and as far from
    /// any edited column, so a dug bank never meets water kept from before.
    fn settle_keepable(&self, edits: &Edits) -> Vec<bool> {
        let n = self.columns.len();
        let walk = |seeds: Vec<usize>| {
            let mut depth = vec![u16::MAX; n];
            let mut front = std::collections::VecDeque::new();
            for s in seeds {
                depth[s] = 0;
                front.push_back(s);
            }
            while let Some(s) = front.pop_front() {
                let next = depth[s] + 1;
                if usize::from(next) >= SETTLE_KEEP_CELLS {
                    continue;
                }
                for t in self.sides_of(s).into_iter().flatten() {
                    if depth[t] > next {
                        depth[t] = next;
                        front.push_back(t);
                    }
                }
            }
            depth
        };
        let rim = walk((0..n).filter(|&s| self.rim(s)).collect());
        let edited = walk(
            (0..n)
                .filter(|&s| {
                    let key = self.water.keys.get(s).copied().unwrap_or(u32::MAX);
                    !edits.for_cell(key).is_empty()
                })
                .collect(),
        );
        (0..n)
            .map(|s| {
                rim[s] == u16::MAX
                    && edited[s] == u16::MAX
                    && self.water.keys.get(s).is_some_and(|&k| k != u32::MAX)
            })
            .collect()
    }

    /// One tick of the rule over the tier: every cell that changed, by slot,
    /// layer and the material it holds now, in the order they changed. The
    /// changed slots' records are repacked.
    pub fn tick_water(&mut self, max: usize) -> Vec<(usize, usize, Material)> {
        let mut queue = std::mem::take(&mut self.water.queue);
        let mut changed = Vec::new();
        queue.tick(
            &mut TierGrid(self),
            liquid::Rules::default(),
            max,
            &mut changed,
        );
        self.water.queue = queue;
        let out: Vec<(usize, usize, Material)> = changed
            .into_iter()
            .filter_map(|(id, cell)| {
                let (slot, layer) = water_cell(id);
                Some((slot, layer, cell.material()?))
            })
            .collect();
        let mut slots: Vec<usize> = out.iter().map(|c| c.0).collect();
        slots.sort_unstable();
        slots.dedup();
        for slot in slots {
            self.repack_slot(slot);
        }
        out
    }
}

/// A measurement instrument (`river-cache`, design 1): what the last tier
/// built on this thread spent in each phase, and what its river settle did.
/// Test builds only; the game's build is untouched by it.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BuildPhases {
    pub columns_ms: f64,
    pub flood_ms: f64,
    pub settle_ms: f64,
    pub relight_ms: f64,
    pub settle_queued: usize,
    pub settle_ticks: usize,
    pub settle_sweeps: usize,
    pub settle_changed: usize,
}

#[cfg(test)]
thread_local! {
    pub(crate) static LAST_BUILD: std::cell::Cell<BuildPhases> =
        std::cell::Cell::new(BuildPhases::default());
}

#[cfg(test)]
fn instrument(f: impl FnOnce(&mut BuildPhases)) {
    LAST_BUILD.with(|c| {
        let mut p = c.get();
        f(&mut p);
        c.set(p);
    });
}

/// A test turns the settle's cache off to build a tier fresh.
#[cfg(test)]
pub(crate) static SETTLE_CACHE_OFF: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn settle_cache_on() -> bool {
    #[cfg(test)]
    {
        !SETTLE_CACHE_OFF.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// How many cells from a tier's rim (and from an edited column) its settled
/// river water is kept and stamped (`river-cache` design 11): past the
/// rule's seven-cell reach with room for a fall's run below it.
pub const SETTLE_KEEP_CELLS: usize = 12;

/// The most ticks a tier's rivers are settled for at its build: longer than
/// any reach's sheet and any fall in it takes to settle.
pub const RIVER_SETTLE_TICKS: usize = 2048;

/// The most cells one flood fills: a sea cave the size of the tier is about
/// a hundred thousand; this bounds a flood whatever the world.
pub const FLOOD_LIMIT: usize = 400_000;

impl Grid for TierGrid<'_> {
    fn cell(&self, at: u32) -> LiquidCell {
        self.0.water_at(at)
    }
    fn set(&mut self, at: u32, cell: LiquidCell) {
        let (slot, layer) = water_cell(at);
        if let (Some(material), Some(column)) = (cell.material(), self.0.columns.get_mut(slot)) {
            column.set(layer, material);
        }
    }
    fn around(&self, at: u32) -> [Option<u32>; 8] {
        self.0.water_around(at)
    }
    fn sea(&self, at: u32) -> bool {
        let (slot, layer) = water_cell(at);
        let joined = &self.0.water.joined;
        if joined.is_empty() {
            return false;
        }
        let sea = sea_layer();
        layer < sea && joined.get(slot * sea + layer).copied().unwrap_or(false)
    }
}

/// One emitting cell: its column's unit direction, the middle of its layer
/// in metres above the planet's radius, and its level, 0..`light::MAX`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LampSpot {
    pub slot: usize,
    pub direction: Vec3,
    pub altitude: f32,
    pub level: u8,
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
            relit: 0,
            lamps: Vec::new(),
            water: TierWater::default(),
        }
    }

    /// Whether any of the tier's columns lies within `angle` radians of
    /// `direction`: where none does, every point there samples open sky
    /// whatever the tier holds (`light::sample`), so a bake from it changes
    /// nothing (`frame-budget` 4.3).
    pub fn reaches(&self, direction: Vec3, angle: f32) -> bool {
        let cos = angle.min(std::f32::consts::PI).cos();
        self.centres.iter().any(|c| c.dot(direction) >= cos)
    }

    /// Every light of the last bake.
    pub fn lamps(&self) -> &[LampSpot] {
        &self.lamps
    }

    /// How many times the field has been baked.
    pub fn light_version(&self) -> u64 {
        self.relit
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

    /// The slot of a finest record, if that record is in the tier.
    pub fn slot(&self, record: usize) -> Option<usize> {
        self.slots
            .get(record)
            .copied()
            .filter(|&s| s < self.columns.len())
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

    /// Write one layer of one record's column as an EDIT writes it: placed
    /// ([`Column::place`]), so it is drawn as the block it is wherever it
    /// stands (`placed-blocks-keep-material`). `false` where there is no
    /// column, or where the column refused it - bedrock does.
    pub fn set_layer(&mut self, record: usize, layer: usize, material: Material) -> bool {
        self.column_mut(record)
            .is_some_and(|column| column.place(layer, material))
    }

    /// Repack a record's runs from its column.
    ///
    /// Called for the edited cell AND for each of its neighbours: a flank is
    /// clipped against the air gaps of the column next to it, so a neighbour's
    /// drawn side changes when this column does even though its own stack did
    /// not. Everything else in the tier is left alone, which is the whole
    /// reason an edit is not a rebuild.
    /// Whether the cell of finest record `record` grows a burning glowing
    /// mushroom now (`glowing-mushrooms` design 4: a pick puts it out), its
    /// record repacked. `true` when the tier's answer changed, and the field
    /// wants relighting.
    pub fn set_glow(&mut self, record: usize, glowing: bool) -> bool {
        let Some(&slot) = self.slots.get(record) else {
            return false;
        };
        match self.glow.get_mut(slot) {
            Some(was) if *was != glowing => *was = glowing,
            _ => return false,
        }
        self.repack(record);
        true
    }

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
        self.water.keys.push(finest[index].key());
        self.water.records.push(index);
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
        for (column, out) in self
            .columns
            .iter()
            .zip(words.chunks_exact_mut(MATERIAL_WORDS))
        {
            pack_materials(column, out);
        }
        words
    }

    /// One slot's light, as [`ColumnTier::gpu_light`] lays it, into `out`
    /// (`LIGHT_WORDS` words; `fine-set-upload` D1). Four one-byte levels to
    /// a word, low byte first, which is `u32::from_le_bytes` of the levels as
    /// they lie: no shifting layer by layer.
    pub(crate) fn pack_light(&self, slot: usize, out: &mut [u32]) {
        let Some(levels) = self.light.get(slot) else {
            out.fill(0);
            return;
        };
        for (word, four) in out.iter_mut().zip(levels.chunks_exact(LIGHT_PER_WORD)) {
            *word = u32::from_le_bytes([four[0].0, four[1].0, four[2].0, four[3].0]);
        }
    }

    /// Each slot's unit direction (`fine-set-upload`: a layout is the same
    /// when these are).
    pub(crate) fn centre_dirs(&self) -> &[Vec3] {
        &self.centres
    }

    /// Every layer's PLACED sheet, eight to a word, `MATERIAL_WORDS` words per
    /// slot, laid after the materials of every slot the buffer can hold
    /// ([`PLACED_SHEETS_BASE`]): nought for generated ground, which takes its
    /// biome's sheet, and for a block somebody placed its own sheet plus one
    /// ([`placed_sheet`]), so it is drawn as the block it is whatever biome
    /// it stands in (`placed-blocks-keep-material`). Only a column with a
    /// placed layer costs more than its zeroes.
    pub fn gpu_placed_sheets(&self) -> Vec<u32> {
        let mut words = vec![0u32; self.columns.len() * MATERIAL_WORDS];
        for (column, out) in self
            .columns
            .iter()
            .zip(words.chunks_exact_mut(MATERIAL_WORDS))
        {
            pack_placed_sheets(column, out);
        }
        words
    }

    /// Every slot's unit direction, w spare: what the shader's walk over the
    /// columns steps between cells by (`lighting-model` L8).
    pub fn gpu_centres(&self) -> Vec<[f32; 4]> {
        self.centres.iter().map(|c| [c.x, c.y, c.z, 0.0]).collect()
    }

    /// The records as the terrain shader reads them (`lighting-perf` 2.1):
    /// each carries, above its degree in `more[2]`, its [`reach top`]: the
    /// highest solid layer, plus one, of every column a reflected ray of
    /// `reach_m` metres that is in this column could still enter. A ray
    /// that is rising and already at or above it can hit nothing, so the
    /// walk ends there with the miss it would have ended with anyway.
    ///
    /// Made at upload from the records as they stand, so an edit that lands
    /// a block is in the bound the moment its records go up.
    ///
    /// [`reach top`]: REACH_TOP_SHIFT
    /// [`TierImage::build`](super::tier_gpu::TierImage::build) makes the
    /// same records for the upload; this is their reference.
    #[cfg(test)]
    pub(crate) fn gpu_records_with_reach(&self, reach_m: f32) -> Vec<GpuColumn> {
        let records = self.gpu_records();
        let rings = reach_rings(records, &self.centres, reach_m);
        let tops = reach_tops(records, rings);
        records
            .iter()
            .zip(tops)
            .map(|(record, top)| with_reach_top(record, top))
            .collect()
    }

    /// `lighting-model` L4: the sun's bounce over this tier for a sun toward
    /// `sun` (planet frame) of colour `light`, slot for slot.
    pub fn bake_bounce(&self, sun: Vec3, light: Vec3) -> pbd_core::bounce::Field {
        pbd_core::bounce::bake(
            &pbd_core::bounce::Region {
                columns: &self.columns,
                neighbors: &self.sides,
                centres: &self.centres,
                radius: PLANET_RADIUS,
            },
            sun,
            light,
        )
    }

    /// `lighting-model` L7: the lamp bake over this tier, from the lamps of
    /// its last light bake (not the glowing flowers, which keep the flood).
    pub fn bake_lamps(&self) -> pbd_core::lamp_bake::LampField {
        let lamps: Vec<pbd_core::lamp_bake::Lamp> = self
            .lamps
            .iter()
            .filter(|l| l.level > flora::GLOW_FLOWER_LEVEL)
            .map(|l| pbd_core::lamp_bake::Lamp {
                slot: l.slot,
                at: l.direction * (PLANET_RADIUS + l.altitude),
                level: l.level as f32 / 14.0,
            })
            .collect();
        pbd_core::lamp_bake::bake(
            &pbd_core::bounce::Region {
                columns: &self.columns,
                neighbors: &self.sides,
                centres: &self.centres,
                radius: PLANET_RADIUS,
            },
            &lamps,
        )
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
        self.relit += 1;
        self.sides = self.neighbor_slots();
        let emitters = self.emitters();
        self.lamps = emitters
            .iter()
            .filter_map(|e| {
                let direction = *self.centres.get(e.column as usize)?;
                Some(LampSpot {
                    slot: e.column as usize,
                    direction,
                    altitude: column::layer_altitude(e.layer as usize) + 0.5,
                    level: e.level,
                })
            })
            .collect();
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

#[cfg(test)]
impl ColumnTier {
    /// A tier of these columns, slot `i` finest record `i` and key `100 + i`,
    /// side `s` of slot `i` being `sides[i][s]` (`NO_NEIGHBOR` off it), every
    /// cell a hexagon and none rim: what a test outside this module (the
    /// water mesher's) builds a small tier with.
    pub(crate) fn for_tests(columns: Vec<Column>, sides: &[[u32; 6]]) -> Self {
        let n = columns.len();
        let records = (0..n)
            .map(|i| GpuColumn {
                runs: columns[i].packed_runs(render_code),
                neighbors: [sides[i][0], sides[i][1], sides[i][2], sides[i][3]],
                more: [
                    sides[i][4],
                    sides[i][5],
                    6,
                    state_word(&columns[i], false, false),
                ],
            })
            .collect();
        let mut tier = ColumnTier {
            columns,
            slots: (0..n).collect(),
            records,
            light: Vec::new(),
            dusk: false,
            glow: vec![false; n],
            centres: vec![Vec3::Y; n],
            sides: Vec::new(),
            relit: 0,
            lamps: Vec::new(),
            water: TierWater {
                queue: liquid::Queue::new(),
                keys: (0..n as u32).map(|i| 100 + i).collect(),
                records: (0..n).collect(),
                ..Default::default()
            },
        };
        tier.sides = tier.neighbor_slots();
        tier
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
    // A top with water standing on it is drawn as a bed, and grows nothing
    // (`flora-v2` 12.10); under the sea's own level the tier keeps drawing
    // its seabed as it did.
    // A PLACED top is the block placed (`placed-blocks-keep-material`): no
    // bed, and drawn from its own sheet, which the record carries.
    let placed = placed_sheet(column, top);
    let code = if placed == 0 && column.water_over(top) && top_m > terrain_config().sea_level_m {
        super::terrain::bed_code(column.drawn_material(top))
    } else {
        render_code(column.drawn_material(top))
    };
    // And whether water stands on it (`flora-v2` 12.11): the column's own
    // water on its top, or a top under the high-water line.
    // Set or cleared, so a pool that drains grows its plants again. The code
    // above is left as it was: under the sea it is the beach's sand, which
    // stood a beach tree in the sea before this bit.
    // An exposed ore top carries the ore mark instead (design 12.12).
    let mark = drawn_sheet_mark(column, top);
    let flooded = column.water_over(top) || top_m < super::terrain::high_water_m();
    let flood_bit = 1 << super::terrain::FLOODED_BIT;
    let sheet_bits = 0xf << super::terrain::PLACED_SHEET_SHIFT;
    let word = finest[index].metadata[1] & !0xff & !flood_bit & !sheet_bits;
    finest[index].metadata[1] = word
        | code
        | if flooded { flood_bit } else { 0 }
        | mark << super::terrain::PLACED_SHEET_SHIFT;
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

/// The degree's bits in `more[2]` (five or six); the rest of the word is free
/// in the tier's own records.
pub const DEGREE_MASK: u32 = 0xff;
/// Where the uploaded records carry their reach top (`lighting-perf` 2.1):
/// nine bits above the degree, a layer index plus one, nought where nothing
/// solid is in reach. Only the GPU's copy carries it
/// ([`ColumnTier::gpu_records_with_reach`]).
pub const REACH_TOP_SHIFT: u32 = 8;
const _: () = assert!(LAYERS < 0x1ff);

/// A record's own top: the highest solid layer its runs hold, plus one (a
/// run's top word is `altitude - COLUMN_BASE_M`, and the cell under that
/// altitude is the run's top cell), or nought for a column with none.
fn record_top(record: &GpuColumn) -> u16 {
    record
        .runs
        .iter()
        .map(|word| ((word >> 9) & 0x1ff) as u16)
        .max()
        .unwrap_or(0)
}

/// A record's neighbour across side `side`, if it has one in the tier.
fn record_side(record: &GpuColumn, side: usize) -> Option<usize> {
    let raw = if side < 4 {
        record.neighbors[side]
    } else {
        record.more[side - 4]
    };
    (raw != NO_NEIGHBOR).then_some(raw as usize)
}

/// Each record's top, raised to the highest top within `rings` steps of
/// it over the tier's own neighbour links (`lighting-perf` 2.1).
pub(crate) fn reach_tops(records: &[GpuColumn], rings: usize) -> Vec<u16> {
    let mut tops: Vec<u16> = records.iter().map(record_top).collect();
    let mut next = tops.clone();
    for _ in 0..rings {
        let mut changed = false;
        for (i, record) in records.iter().enumerate() {
            let degree = ((record.more[2] & DEGREE_MASK) as usize).min(6);
            let mut top = tops[i];
            for side in 0..degree {
                if let Some(n) = record_side(record, side).filter(|&n| n < tops.len()) {
                    top = top.max(tops[n]);
                }
            }
            changed |= top != tops[i];
            next[i] = top;
        }
        std::mem::swap(&mut tops, &mut next);
        if !changed {
            break;
        }
    }
    tops
}

/// How many neighbour steps hold every column a ray of `reach_m` metres can
/// enter after any point of its path, from the tier's own spacing: the
/// path's angle at the lowest radius a column reaches, plus a cell's
/// circumradius at each end, over the closest ring spacing a hex grid has
/// (its inradius, `sqrt 3 / 2` of the narrowest centre spacing), plus two
/// for the pentagons and rounding. Over-counting costs only a looser bound.
pub(crate) fn reach_rings(records: &[GpuColumn], centres: &[Vec3], reach_m: f32) -> usize {
    let (mut narrow, mut wide) = (f32::INFINITY, 0.0f32);
    for (i, record) in records.iter().enumerate().take(centres.len()) {
        let degree = ((record.more[2] & DEGREE_MASK) as usize).min(6);
        for side in 0..degree {
            if let Some(n) = record_side(record, side).filter(|&n| n < centres.len()) {
                // From the chord: `angle_between`'s arccosine of a dot
                // within f32's step of one reads neighbours 0.0006 rad apart
                // as nought.
                let angle = 2.0 * ((centres[i] - centres[n]).length() / 2.0).asin();
                narrow = narrow.min(angle);
                wide = wide.max(angle);
            }
        }
    }
    if !narrow.is_finite() || narrow <= 0.0 {
        // No spacing to read: every ring there is, which bounds nothing
        // and so cuts nothing short.
        return records.len();
    }
    let lowest = crate::planet::PLANET_RADIUS + pbd_core::column::BASE_M as f32;
    let span = reach_m.max(0.0) / lowest + 2.0 * wide / 3f32.sqrt();
    (span / (narrow * 3f32.sqrt() / 2.0)).ceil() as usize + 2
}

/// The topmost water layer of a column, plus one, shifted: nought where the
/// column holds no water. The shader turns it into the altitude of the water's
/// surface, which is what decides whether a face under sea level is wet.
fn water_word(column: &Column) -> u32 {
    for layer in (1..LAYERS).rev() {
        let material = column.material(layer);
        if material.is_water() {
            return (layer as u32 + 1) << WATER_SHIFT | water_short(material) << WATER_SHORT_SHIFT;
        }
    }
    0
}

/// [`WATER_SHORT_SHIFT`]'s eighths for a column's top water layer: how far
/// its surface stands under the layer's top, rounded down, so the tint stops
/// at most an eighth of a metre over the water's own surface.
pub fn water_short(top: Material) -> u32 {
    let fraction = pbd_core::liquid::top_fraction(top).clamp(0.0, 1.0);
    (((1.0 - fraction) * 8.0).floor() as u32).min(WATER_SHORT_MASK)
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

/// Set in a finest record's `spare[2]` where its cell's glowing mushroom is
/// picked (`glowing-mushrooms` design 4): the flower slot draws nothing
/// there, so no ordinary flower grows where the mushroom was. Bit 3, the
/// next free one (1 and 2 are the sea's).
pub const GLOW_PICKED_BIT: u32 = 8;

/// Set in a finest record's `spare[2]` where its cell's ground is under the
/// high water and yet no sea sheet lies over it: a pit or a hollow the sea's
/// water does not reach (`volumetric-water` phase 2, `ocean-environments`
/// B22), or a river's mouth whose own top is the cell's one surface (B18).
/// The visibility pass lists no sea sheet over it, so a sealed pocket is dry
/// on screen as it is in the column.
pub const DRY_SHEET_BIT: u32 = 2;

/// Set in a finest record's `spare[2]` where the sea's surface reaches the
/// cell (`ocean-environments` B22, [`ColumnTier::sea_reach`]):
/// the sheet is listed over it whatever its ground's or roof's height.
pub const SEA_REACH_BIT: u32 = 4;

/// Where `spare[2]` carries how much darker than the open sky the sea's top
/// is lit in the cell, `15 - sky light` over the water's top in its column,
/// four bits: nought (the open sky) outside a cave. The sheet is lit by it
/// under a roof, as the mesher's water there was.
pub const SHEET_SHADE_SHIFT: u32 = 8;
const SHEET_SHADE_MASK: u32 = 0xf << SHEET_SHADE_SHIFT;
// Each of `spare[2]`'s flags has its own bit.
const _: () = assert!(
    GLOW_PICKED_BIT & (GLOW_FLOWER_BIT | DRY_SHEET_BIT | SEA_REACH_BIT | SHEET_SHADE_MASK) == 0
);

/// Whether the sea's sheet is drawn over a finest record: the visibility
/// pass's listing (`planet_visibility.wgsl`), for the readers on the CPU that
/// must agree with it (the mesher leaves the sea's top to the sheet exactly
/// where this holds). A record off the tier is listed by its height alone.
pub fn sheet_listed(cell: &GpuCell) -> bool {
    let high = super::terrain::high_water_m();
    (cell.direction_height[3] < high && cell.spare[2] & DRY_SHEET_BIT == 0)
        || cell.spare[2] & SEA_REACH_BIT != 0
}

/// Whether a finest record is marked as reached by the sea's surface.
pub fn sea_reached(cell: &GpuCell) -> bool {
    cell.spare[2] & SEA_REACH_BIT != 0
}

/// Mark every record of the tier for the sea's sheet from its column and the
/// sea's reach: [`DRY_SHEET_BIT`], [`SEA_REACH_BIT`] and the sheet's shade.
/// Derived from the columns, the light and nothing else, like the flood: the
/// same tier marks the same way however often it is built, and nothing of it
/// is saved. Called at the build and after an edit, once the light is baked.
pub fn mark_sea(tier: &ColumnTier, finest: &mut [GpuCell]) {
    let sea = sea_layer();
    let high = super::terrain::high_water_m();
    let reach = tier.sea_reach();
    for (slot, &record) in tier.water.records.iter().enumerate() {
        let (Some(column), Some(cell)) = (tier.columns.get(slot), finest.get_mut(record)) else {
            continue;
        };
        // A river's water standing on over the sea's level (a mouth whose bed
        // is under the sea, or a bed over it): the river's own top is that
        // cell's one surface,
        // and the sheet under it was a second one inside the river's water
        // (`ocean-environments` B18, the junction check).
        let river_over = sea > 0 && (sea..LAYERS).any(|l| column.material(l).is_water());
        // Flowing water at the sea's top layer is a river's, whose own top
        // the mesher draws: the sheet under it would be a second surface.
        let flowing_top = sea > 0 && {
            let top = column.material(sea - 1);
            top.is_water() && top != Material::Water
        };
        let reached = reach.get(slot).copied().unwrap_or(false) && !river_over && !flowing_top;
        // Dry: under the high water, and the sea's surface does not reach it
        // (a pit or a hollow nothing joins to the sea), unless its
        // column holds water at the sea's top (a pool somebody poured), or
        // a river stands over it.
        let dry = sea > 0
            && cell.direction_height[3] < high
            && (river_over || (!reached && !column.material(sea - 1).is_water()));
        // The light over the water's top: the sea's top layer or the air
        // over it, whichever is lit more.
        let shade = if reached && sea > 0 {
            let sky = tier
                .light_at(slot, sea - 1)
                .sky()
                .max(tier.light_at(slot, sea).sky());
            u32::from(light::MAX.saturating_sub(sky)).min(0xf)
        } else {
            0
        };
        cell.spare[2] = (cell.spare[2] & !(DRY_SHEET_BIT | SEA_REACH_BIT | SHEET_SHADE_MASK))
            | if dry { DRY_SHEET_BIT } else { 0 }
            | if reached { SEA_REACH_BIT } else { 0 }
            | shade << SHEET_SHADE_SHIFT;
    }
}

impl ColumnTier {
    /// Which slots the sea's surface reaches (`ocean-environments` B22): a
    /// walk from the open sea (a column holding the sea's water at its top
    /// layer, open air over it, its record under sea level) through every
    /// layer that is not solid, up to the high water's and never short of the
    /// first layer over the sea's level (the waves' margin; `no-tides`),
    /// across the sides and up and down. A pit behind a beach at the sea's
    /// level, and a
    /// cavern under a bank opening onto the sea, are reached; a hollow inland
    /// that joins no water is not. The rim is walked like any column: its
    /// water is the sea's.
    pub fn sea_reach(&self) -> Vec<bool> {
        let sea = sea_layer();
        let n = self.columns.len();
        let mut reached = vec![false; n];
        if sea == 0 || n == 0 {
            return reached;
        }
        let high = super::terrain::high_water_m();
        let top = column::layer_at(high - 1e-3).unwrap_or(sea).max(sea);
        let band = top + 1;
        let mut seen = vec![false; n * band];
        let mut open: std::collections::VecDeque<(usize, usize)> = Default::default();
        // Air, or water under the sea's level: a river's or a pool's water
        // over it is its own, and the sea's surface does not run through it.
        // Not the air under the sea's level in a column a player dug or
        // built in (`dry-holes` design 3): a dug hole is the sea's only where
        // the sea's water has flowed into it, never because the sea's air
        // over ground beside it joins it.
        let dug = |slot: usize| self.water.dug.get(slot).copied().unwrap_or(false);
        let passable = |slot: usize, layer: usize| {
            let m = self.columns[slot].material(layer);
            layer >= 1
                && ((m == Material::Air && (layer >= sea || !dug(slot)))
                    || (m.is_water() && layer < sea))
        };
        for slot in self.open_sea() {
            seen[slot * band + sea - 1] = true;
            open.push_back((slot, sea - 1));
        }
        while let Some((slot, layer)) = open.pop_front() {
            reached[slot] = true;
            let mut visit = |s: usize, l: usize, open: &mut std::collections::VecDeque<_>| {
                if l < band && !seen[s * band + l] && passable(s, l) {
                    seen[s * band + l] = true;
                    open.push_back((s, l));
                }
            };
            visit(slot, layer + 1, &mut open);
            if layer > 1 {
                visit(slot, layer - 1, &mut open);
            }
            for side in self.sides_of(slot).into_iter().flatten() {
                visit(side, layer, &mut open);
            }
        }
        // A dug column the walk met only over the sea's level (the air over a
        // beach beside it) is the sea's where its top layer under the
        // sea's level is the sea's water or solid; with air there, it is a
        // dry hole, or one the sea is still flowing into.
        for (slot, reach) in reached.iter_mut().enumerate() {
            if *reach && dug(slot) && self.columns[slot].material(sea - 1) == Material::Air {
                *reach = false;
            }
        }
        reached
    }

    /// The open sea's columns: the sea's water at its top layer with open air
    /// over it, under sea level. Where the sea's reach and its joined walk
    /// start.
    fn open_sea(&self) -> Vec<usize> {
        let sea = sea_layer();
        if sea == 0 {
            return Vec::new();
        }
        (0..self.columns.len())
            .filter(|&slot| {
                let column = &self.columns[slot];
                let under_sea = self
                    .water
                    .records
                    .get(slot)
                    .and_then(|_| column.surface())
                    .is_none_or(|top| {
                        column::layer_altitude(top) + 1.0 <= terrain_config().sea_level_m
                    });
                column.material(sea - 1) == Material::Water
                    && !column.solid(sea)
                    && !column.material(sea).is_water()
                    && under_sea
            })
            .collect()
    }

    /// Which cells under the sea's level the sea is joined to (`dry-holes`
    /// design 4): a walk from the open sea's top layer through every cell
    /// under the sea's level that is not solid (air or water), across the
    /// sides and up and down. Indexed `slot * sea_layer + layer`. Where the
    /// rule's water is the sea's ([`liquid::transform_at`]): a pit whose wall
    /// is breached is joined, so the sea fills it; a sealed pit is not, so a
    /// bucket poured into it stays the bucket's. A function of the solids
    /// alone, so the water the rule moves never changes it.
    pub fn sea_joined(&self) -> Vec<bool> {
        let sea = sea_layer();
        let n = self.columns.len();
        let mut joined = vec![false; n * sea];
        if sea == 0 || n == 0 {
            return joined;
        }
        let mut open: std::collections::VecDeque<(usize, usize)> = Default::default();
        for slot in self.open_sea() {
            joined[slot * sea + sea - 1] = true;
            open.push_back((slot, sea - 1));
        }
        while let Some((slot, layer)) = open.pop_front() {
            let mut visit = |s: usize, l: usize, open: &mut std::collections::VecDeque<_>| {
                if l >= 1
                    && l < sea
                    && !joined[s * sea + l]
                    && !self.rim(s)
                    && !self.columns[s].solid(l)
                {
                    joined[s * sea + l] = true;
                    open.push_back((s, l));
                }
            };
            visit(slot, layer + 1, &mut open);
            visit(slot, layer.saturating_sub(1), &mut open);
            for side in self.sides_of(slot).into_iter().flatten() {
                visit(side, layer, &mut open);
            }
        }
        joined
    }
}

/// Whether a record's cell grows a glowing flower.
fn glows(cell: &GpuCell) -> bool {
    cell.spare[2] & GLOW_FLOWER_BIT != 0
}

/// A finest record's `spare[2]` with its glowing-flower bits for the cell
/// `key`, built `from_camera_m` metres from the camera: [`GLOW_FLOWER_BIT`]
/// where it grows one and no pick stands, [`GLOW_PICKED_BIT`] where a pick
/// stands (`glowing-mushrooms` design 4). Every other bit is kept.
pub fn glow_bits(
    spare: u32,
    key: u32,
    chance: f32,
    picks: &pbd_core::edits::GlowPicks,
    from_camera_m: f32,
) -> u32 {
    let grows = key != crate::planet::NO_KEY && flora::glows(key, chance);
    let picked = grows && picks.stands_picked(key, from_camera_m);
    let bits = if picked {
        GLOW_PICKED_BIT
    } else if grows {
        GLOW_FLOWER_BIT
    } else {
        0
    };
    (spare & !(GLOW_FLOWER_BIT | GLOW_PICKED_BIT)) | bits
}

/// Set in a finest record's `spare[2]` where the player has marked its cell
/// for paving (`townsfolk` design 39a): the cap draws a lime band between
/// the sides at [`MARK_SIDE_IN_SHIFT`] and [`MARK_SIDE_OUT_SHIFT`], three
/// bits each, [`MARK_NO_SIDE`] where the plan ends on the cell's middle.
/// `planet_surface.wgsl` reads them.
pub const MARK_BIT: u32 = 1 << 16;
pub const MARK_SIDE_IN_SHIFT: u32 = 17;
pub const MARK_SIDE_OUT_SHIFT: u32 = 20;
pub const MARK_NO_SIDE: u32 = 7;
const MARK_MASK: u32 = 0x7f << 16;
const _: () = assert!(
    MARK_MASK
        & (GLOW_PICKED_BIT | GLOW_FLOWER_BIT | DRY_SHEET_BIT | SEA_REACH_BIT | SHEET_SHADE_MASK)
        == 0
);

/// The side of a record's cell (its `corners[side]` to `corners[side + 1]`)
/// a neighbouring cell lies across: the side whose middle is nearest it.
pub fn side_toward(cell: &GpuCell, neighbour: u32) -> Option<u32> {
    let toward = pbd_core::roads::centre(neighbour)?;
    let degree = cell.degree().min(6);
    (0..degree)
        .map(|side| {
            let a = Vec3::from_slice(&cell.corners[side][..3]);
            let b = Vec3::from_slice(&cell.corners[(side + 1) % degree][..3]);
            (side as u32, (a + b).normalize_or_zero().dot(toward))
        })
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .map(|(side, _)| side)
}

/// A finest record's `spare[2]` with its mark bits for the cell `key`, from
/// the installed marks (`pbd_core::roads::paving::mark_of`). Every other bit
/// is kept.
pub fn mark_bits(cell: &GpuCell, key: u32) -> u32 {
    let spare = cell.spare[2] & !MARK_MASK;
    let Some((before, after)) = pbd_core::roads::paving::mark_of(key) else {
        return spare;
    };
    let side = |n: Option<u32>| n.and_then(|n| side_toward(cell, n)).unwrap_or(MARK_NO_SIDE);
    spare | MARK_BIT | side(before) << MARK_SIDE_IN_SHIFT | side(after) << MARK_SIDE_OUT_SHIFT
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
    let reach = (settings.reach_m.max(0.) / PLANET_RADIUS).cos();
    build_with(
        anchor,
        finest,
        neighbors,
        settings,
        edits,
        &Members {
            worms_m: settings.reach_m.max(0.),
            select: &|_, direction: Vec3| direction.dot(anchor) > reach,
            capacity: COLUMN_CAPACITY as usize,
            light: true,
        },
    )
}

/// Which finest cells a tier is built over, and how (`river-cache` design
/// 14): the column tier takes every cell within its reach and is lit; the
/// river band takes a river's cells out to the finest band, and needs no
/// caves (a river's valley is sealed from them) but its light, which its
/// water's faces are shaded by.
pub struct Members<'a> {
    /// How far round the anchor caves are gathered from, m.
    pub worms_m: f32,
    /// Whether the finest record at an index, at a direction, is a member.
    pub select: &'a dyn Fn(usize, Vec3) -> bool,
    /// The most members.
    pub capacity: usize,
    /// Whether the tier is lit (`relight`).
    pub light: bool,
}

/// [`build`] over the chosen members. The records' slot bits are written as
/// the column tier's are: a caller building anything but the column tier
/// passes a copy of the records.
pub fn build_with(
    anchor: Vec3,
    finest: &mut [GpuCell],
    neighbors: &[[u32; 6]],
    settings: &ColumnSettings,
    edits: &Edits,
    members_of: &Members,
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
    let region = worms::gather(&field, terrain_config(), anchor, members_of.worms_m);
    let mut slots = vec![usize::MAX; finest.len()];
    let mut members = Vec::new();
    for (index, cell) in finest.iter_mut().enumerate() {
        // Whether the cell grows a glowing flower, decided here once from its
        // key, for every finest cell: the shader grows the flower wherever
        // the bit is, and the bake lights the ones in the tier. A picked one
        // (`glowing-mushrooms` design 4) is neither drawn nor lit, and its
        // cell grows no other flower in its place while the pick stands.
        let key = cell.key();
        let direction = Vec3::from_slice(&cell.direction_height[..3]);
        cell.spare[2] = glow_bits(
            cell.spare[2],
            key,
            settings.glow_flower_chance,
            &edits.glow,
            direction.normalize_or(Vec3::Y).angle_between(anchor) * PLANET_RADIUS,
        );
        // A cell the player marked for paving (`townsfolk` design 39a).
        cell.spare[2] = mark_bits(cell, key);
        if !(members_of.select)(index, direction) || members.len() >= members_of.capacity {
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
    #[cfg(test)]
    let phase = std::time::Instant::now();
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
        // No caves where none are gathered (the river band, `worms_m` 0): a
        // river's valley is sealed from them, and a column is then the same
        // whatever anchor built it.
        let column = if rim || members_of.worms_m <= 0.0 {
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
        // Only DOWNWARD for the generator: a carve can only take the surface
        // away, and a generated column top above the record is a fault a
        // test holds out. A cell a player EDITED reconciles both ways, as the
        // edit did when it was made (`reconcile_surface`): a block placed on
        // the ground raises the cap and wears the block's material, so a
        // rebuild or a reload draws what the edit drew, and a tree the block
        // put out does not grow back through it and whatever stands on it
        // (`edit-pipeline`, the owner's vanished lamp, 2026-11-16).
        let edited = !edits.for_cell(finest[index].key()).is_empty();
        // A floe of sea ice (generator version 14, `ocean-environments` B24)
        // is the one generated top that stands over the record: it floats
        // on the sea, which the height field's record is the bed under. The
        // record takes its top, so its cap is drawn and the walls follow.
        let floe = column.material(top) == pbd_core::terrain::Material::SeaIce;
        if top_m >= finest[index].direction_height[3] && !edited && !floe {
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
        relit: 0,
        lamps: Vec::new(),
        water: TierWater {
            queue: liquid::Queue::new(),
            keys: members.iter().map(|&i| finest[i].key()).collect(),
            records: members.clone(),
            // A column the save digs or builds in (`dry-holes` design 3).
            dug: members
                .iter()
                .map(|&i| {
                    edits
                        .for_cell(finest[i].key())
                        .iter()
                        .any(|&(_, m)| !m.is_water())
                })
                .collect(),
            joined: Vec::new(),
        },
    };
    // The sea floods what a carve or an edit opened onto it (`volumetric-water`
    // phase 2): derived from the columns and the edits, so the same tier
    // floods the same way however often it is built, and never saved.
    #[cfg(test)]
    let phase = {
        let ms = phase.elapsed().as_secs_f64() * 1000.;
        instrument(|p| p.columns_ms = ms);
        std::time::Instant::now()
    };
    tier.sides = tier.neighbor_slots();
    let seeds = tier.sea_seeds(edits);
    let flooded = tier.flood_from(seeds);
    for slot in flooded {
        tier.repack_slot(slot);
    }
    #[cfg(test)]
    let phase = {
        let ms = phase.elapsed().as_secs_f64() * 1000.;
        instrument(|p| p.flood_ms = ms);
        std::time::Instant::now()
    };
    // A version 8 world's rivers, settled by the rule (design 13).
    let settled = tier.settle_rivers(edits);
    #[cfg(test)]
    let phase = {
        let ms = phase.elapsed().as_secs_f64() * 1000.;
        instrument(|p| p.settle_ms = ms);
        std::time::Instant::now()
    };
    if settled > 0 {
        bevy::log::debug!("rivers settled at build: {settled} cells");
    }
    // Water the edits hold picks up where it was (`water-flow` E4).
    tier.queue_edited_water(edits);
    // The light comes last, because it is a function of the finished columns:
    // the rim's solid ring and every edit are already in them, and lighting
    // before either would light a world that is not the one being drawn.
    if members_of.light {
        tier.relight();
    }
    #[cfg(test)]
    {
        let ms = phase.elapsed().as_secs_f64() * 1000.;
        instrument(|p| p.relight_ms = ms);
    }
    // And where the sea's sheet lies, from the sea's reach and the light
    // (`ocean-environments` B22): none over a pit the sea has not reached.
    mark_sea(&tier, finest);
    // And where the rule's water is the sea's (`dry-holes` design 4), after
    // the rivers have settled by the rule alone.
    tier.water.joined = tier.sea_joined();
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

    /// The owner, 2026-11-16: "underwater ground tint is happening before
    /// the water line". The record carries how far a flowing top's surface
    /// stands under its layer's top, so the shader's surface
    /// (`column_water_top_m`) is the water's own, within an eighth of a metre
    /// and never over it; a source's is the layer's top, as before.
    #[test]
    fn a_flowing_tops_surface_rides_its_record_to_the_eighth() {
        let read = |column: &Column| {
            let word = state_word(column, false, false);
            let packed = (word >> WATER_SHIFT) & WATER_MASK;
            let short = (word >> WATER_SHORT_SHIFT) & WATER_SHORT_MASK;
            pbd_core::column::BASE_M as f32 + packed as f32 - short as f32 * 0.125
        };
        let source = flooded(20, 40);
        assert_eq!(read(&source), pbd_core::column::layer_altitude(40) + 1.0);
        for level in 0..8u8 {
            let mut column = flooded(20, 39);
            column.set(40, Material::flowing(level, false));
            let surface = column.water_surface(pbd_core::column::layer_altitude(40) + 0.01);
            let surface = surface.expect("in the flowing layer");
            let drawn = read(&column);
            assert!(
                drawn >= surface && drawn - surface <= 0.125 + 1e-4,
                "level {level}: the tint stops at {drawn}, the water at {surface}"
            );
            // The rest of the word is untouched.
            assert_eq!(water_top_layer(&column), Some(40));
        }
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
    /// `lighting-perf` 2.1: a column's reach top is the highest solid layer,
    /// plus one, within the rings asked for and no further, and it rides
    /// above the degree, which the shader still reads whole.
    #[test]
    fn reach_tops_hold_the_highest_layer_in_reach() {
        // A chain of seven columns, each linked to the next and the last.
        let tops = [3u32, 0, 5, 2, 2, 9, 1];
        let records: Vec<GpuColumn> = (0..tops.len())
            .map(|i| {
                let mut runs = [column::Run::ABSENT; MAX_RUNS];
                if tops[i] > 0 {
                    runs[0] = column::Run {
                        from: 0,
                        to: tops[i] as usize,
                        material: Material::Stone,
                        body: Material::Stone,
                    }
                    .packed(1, 1);
                }
                let next = if i + 1 < tops.len() {
                    (i + 1) as u32
                } else {
                    NO_NEIGHBOR
                };
                let back = if i > 0 { (i - 1) as u32 } else { NO_NEIGHBOR };
                GpuColumn {
                    runs,
                    neighbors: [next, back, NO_NEIGHBOR, NO_NEIGHBOR],
                    more: [NO_NEIGHBOR, NO_NEIGHBOR, 2, 0],
                }
            })
            .collect();
        assert_eq!(reach_tops(&records, 0), vec![3, 0, 5, 2, 2, 9, 1]);
        assert_eq!(reach_tops(&records, 1), vec![3, 5, 5, 5, 9, 9, 9]);
        assert_eq!(reach_tops(&records, 2), vec![5, 5, 5, 9, 9, 9, 9]);
        assert_eq!(reach_tops(&records, 99), vec![9; 7]);
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        for line in [
            format!("const REACH_TOP_SHIFT: u32 = {REACH_TOP_SHIFT}u;"),
            format!("const DEGREE_MASK: u32 = {DEGREE_MASK}u;"),
        ] {
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should declare `{line}`"
            );
        }
    }

    /// The rings cover the reach: on the shipped tier's spacing (about 2.8
    /// m), 48 m is some twenty rings, and a longer reach is never fewer.
    #[test]
    fn reach_rings_cover_the_reach() {
        let spacing = 2.833 / crate::planet::PLANET_RADIUS;
        let centres = [
            Vec3::X,
            bevy::prelude::Quat::from_rotation_z(spacing) * Vec3::X,
        ];
        let pair = |other: u32| GpuColumn {
            runs: [column::Run::ABSENT; MAX_RUNS],
            neighbors: [other, NO_NEIGHBOR, NO_NEIGHBOR, NO_NEIGHBOR],
            more: [NO_NEIGHBOR, NO_NEIGHBOR, 1, 0],
        };
        let records = [pair(1), pair(0)];
        let rings = reach_rings(&records, &centres, 48.0);
        assert!(
            rings as f32 * 2.833 * 3f32.sqrt() / 2.0 >= 48.0,
            "{rings} rings"
        );
        assert!(rings < 40, "{rings} rings");
        assert!(reach_rings(&records, &centres, 96.0) > rings);
    }

    use super::*;
    use crate::planet::lod;

    /// The tier is built off the real fine set, so what is tested is what the
    /// frame is handed rather than a hand-built stand-in.
    fn tier(anchor: Vec3) -> (lod::FineSet, ColumnTier) {
        let mut set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let tier = set.take_columns();
        (set, tier)
    }

    /// `lighting-perf` 2.1 on a real tier (Holbrook's): every column's reach
    /// top is at least the top of every column whose centre is within the
    /// reach plus a cell on each side, measured straight, not by rings.
    #[test]
    fn a_real_tiers_reach_tops_cover_the_reach() {
        let anchor = Vec3::new(0.868_428, 0.495_384, 0.020_667).normalize();
        let (_, tier) = tier(anchor);
        let reach_m = 48.0;
        let records = tier.gpu_records_with_reach(reach_m);
        let own: Vec<u16> = tier.gpu_records().iter().map(record_top).collect();
        let rings = reach_rings(tier.gpu_records(), &tier.centres, reach_m);
        let lowest = crate::planet::PLANET_RADIUS + pbd_core::column::BASE_M as f32;
        let span = (reach_m + 2.0 * 3.2) / lowest;
        let mut worst = 0;
        for (i, record) in records.iter().enumerate() {
            let top = ((record.more[2] >> REACH_TOP_SHIFT) & 0x1ff) as u16;
            assert_eq!(record.more[2] & DEGREE_MASK, tier.gpu_records()[i].more[2]);
            for (j, c) in tier.centres.iter().enumerate() {
                let chord = (tier.centres[i] - *c).length();
                if 2.0 * (chord / 2.0).asin() <= span && own[j] > top {
                    worst = worst.max(own[j] - top);
                }
            }
        }
        assert_eq!(worst, 0, "{rings} rings fall short by up to {worst} layers");
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
            relit: 0,
            lamps: Vec::new(),
            water: TierWater::default(),
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
    fn a_picked_glowing_mushroom_is_neither_drawn_nor_lit() {
        // `glowing-mushrooms` 2.3: a pick, saved and read back, makes the
        // build mark its cell picked (no plant, no other flower) and adds no
        // emitter there at night; the resident set takes a pick at once.
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let settings = ColumnSettings::default();
        let mut set = lod::generate_fine(anchor, &settings, &Edits::new());
        set.columns.set_dusk(true);
        let lit_slot = |set: &lod::FineSet, slot: usize| {
            set.columns
                .emitters()
                .iter()
                .any(|e| e.level == flora::GLOW_FLOWER_LEVEL && e.column == slot as u32)
        };
        let (index, slot) = (0..set.finest_records().len())
            .filter(|&i| glows(&set.finest_records()[i]) && set.columns.slots[i] != usize::MAX)
            .map(|i| (i, set.columns.slots[i]))
            .find(|&(_, slot)| lit_slot(&set, slot))
            .expect("a burning glowing mushroom in the tier");
        let key = set.finest_records()[index].key();
        // Saved, and read back from the disk.
        let root = std::env::temp_dir().join(format!("pbd-glow-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot_file = crate::saves::create(&root, "Glow", 31).unwrap();
        {
            let mut save = crate::saves::WorldSave::open(root.clone(), slot_file);
            let mut slots = pbd_core::inventory::Slots::new();
            slots.give(
                pbd_core::inventory::Item::Good(pbd_core::goods::Good::GLOWING_MUSHROOM),
                1,
            );
            let line = pbd_core::records::Record::of(
                flora::GLOW_PLANT_RECORD,
                u64::from(key),
                flora::GLOW_PLANT_SCHEMA,
                &flora::GlowPicked { day: 3.0 },
            );
            assert!(save.store_with_hands(line, flora::GLOW_PLANT_SCHEMA, &slots));
            assert_eq!(save.edits.glow.picked_on(key), Some(3.0));
            save.drain();
        }
        let listed = crate::saves::list(&root);
        let reopened = crate::saves::WorldSave::open(root.clone(), listed[0].clone());
        let _ = std::fs::remove_dir_all(&root);
        let mut edits = reopened.edits.clone();
        assert_eq!(
            edits.glow.picked_on(key),
            Some(3.0),
            "the pick survives a reload"
        );
        edits.glow.day = 4.0;
        edits.glow.far_m = 120.0;
        // Built with the pick.
        let mut picked = lod::generate_fine(anchor, &settings, &edits);
        picked.columns.set_dusk(true);
        let cell = &picked.finest_records()[index];
        assert_eq!(cell.key(), key);
        assert_eq!(cell.spare[2] & GLOW_FLOWER_BIT, 0, "no plant");
        assert_ne!(
            cell.spare[2] & GLOW_PICKED_BIT,
            0,
            "and no flower in its place"
        );
        assert!(!lit_slot(&picked, picked.columns.slots[index]), "no light");
        // The resident set, built before the pick, takes it at once.
        let changed = set.apply_glow_picks(&edits.glow, settings.glow_flower_chance, |_| 0.0);
        assert_eq!(changed, 1);
        assert!(!lit_slot(&set, slot));
        // Due and in front of the player, it stays picked; out of sight it grows.
        edits.glow.day = 3.0 + flora::GLOW_PLANT_REGROW_DAYS;
        assert_eq!(
            set.apply_glow_picks(&edits.glow, settings.glow_flower_chance, |_| 20.0),
            0
        );
        assert_eq!(
            set.apply_glow_picks(&edits.glow, settings.glow_flower_chance, |_| 500.0),
            1
        );
        assert!(lit_slot(&set, slot), "grown back out of sight");
    }

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
            relit: 0,
            lamps: Vec::new(),
            water: TierWater::default(),
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

/// The tier's water (`volumetric-water` phase 2, `water-flow` E3 and E4) on a
/// line of columns: the sea at one end, land at the other, the ends rim.
#[cfg(test)]
mod water_tier_tests {
    use super::*;
    use pbd_core::edits::Edit;

    const N: usize = 12;
    /// The top of the seabed and of the land, layers.
    const BED: usize = 130;
    const LAND: usize = 150;

    /// Columns 1 to 4 the sea (water from the bed to sea level), 5 to 10
    /// land, 0 and 11 the rim. Slot `i` is finest record `i`, key `100 + i`.
    fn line() -> ColumnTier {
        let sea = sea_layer();
        let columns: Vec<Column> = (0..N)
            .map(|i| {
                let mut column = Column::bedrock();
                let top = if (1..=4).contains(&i) { BED } else { LAND };
                for layer in 1..=top {
                    column.set(layer, Material::Stone);
                }
                if (1..=4).contains(&i) {
                    for layer in BED + 1..sea {
                        column.set(layer, Material::Water);
                    }
                }
                column
            })
            .collect();
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
                    u32::from(i == 0 || i == N - 1) | state_word(&columns[i], false, false),
                ],
            })
            .collect();
        let mut tier = ColumnTier {
            columns,
            slots: (0..N).collect(),
            records,
            light: Vec::new(),
            dusk: false,
            glow: vec![false; N],
            centres: vec![Vec3::Y; N],
            sides: Vec::new(),
            relit: 0,
            lamps: Vec::new(),
            water: TierWater {
                queue: liquid::Queue::new(),
                keys: (0..N as u32).map(|i| 100 + i).collect(),
                records: (0..N).collect(),
                ..Default::default()
            },
        };
        tier.sides = tier.neighbor_slots();
        tier
    }

    fn carve(
        tier: &mut ColumnTier,
        slots: std::ops::RangeInclusive<usize>,
        layers: std::ops::RangeInclusive<usize>,
    ) {
        for slot in slots {
            for layer in layers.clone() {
                tier.columns[slot].set(layer, Material::Air);
            }
        }
    }

    /// Build's flood: a tunnel from the sea floods to sea level, never above
    /// it, and a sealed pocket beside it stays dry.
    #[test]
    fn a_tunnel_into_the_sea_floods_and_a_sealed_pocket_stays_dry() {
        let sea = sea_layer();
        let mut tier = line();
        // A tunnel under the land from the sea (column 4), three layers
        // high, with a shaft rising out of it in column 7 to two layers over
        // sea level; and a pocket two columns on, sealed by column 8's rock.
        carve(&mut tier, 5..=7, BED + 5..=BED + 7);
        carve(&mut tier, 7..=7, BED + 8..=sea + 1);
        carve(&mut tier, 9..=10, BED + 5..=BED + 7);
        let seeds = tier.sea_seeds(&Edits::new());
        assert!(!seeds.is_empty(), "the sea touches the tunnel");
        let flooded = tier.flood_from(seeds);
        assert_eq!(flooded, vec![5, 6, 7], "the tunnel's columns");
        for slot in 5..=7 {
            for layer in BED + 5..=BED + 7 {
                assert_eq!(tier.columns[slot].material(layer), Material::Water);
            }
        }
        // The shaft rising out of the tunnel stays dry: the flood never
        // climbs (design, phase 2), so it fills down and across from the sea,
        // and never over sea level.
        for layer in BED + 8..=sea + 1 {
            assert_eq!(tier.columns[7].material(layer), Material::Air, "{layer}");
        }
        for slot in 9..=10 {
            assert_eq!(
                tier.columns[slot].material(BED + 6),
                Material::Air,
                "sealed"
            );
        }
        assert_eq!(
            tier.columns[11].material(BED + 6),
            Material::Stone,
            "the rim is solid"
        );
    }

    /// `ocean-environments` B22, the owner's caverns beside the sea: a pit
    /// behind a beach at the sea's level and a cavern opening off it under a
    /// bank are reached by the sea's surface, so the sheet lies over both
    /// and the mesher leaves the sea's top to it; a pit inland that joins no
    /// water is not, and no sheet lies over it, though its ground is as low.
    #[test]
    fn the_sea_reaches_a_cavern_behind_the_beach_and_not_a_pit_inland() {
        let sea = sea_layer();
        let mut tier = line();
        // Column 5 a beach whose sand stands at sea level; 6 a pit in it,
        // open to the sky, its floor under sea level; 7 a cavern off the pit
        // under the bank, roofed to the land's height; 8 land; 9 and 10 a pit
        // as deep, open to the sky, beyond the land.
        carve(&mut tier, 5..=5, sea..=LAND);
        carve(&mut tier, 6..=6, BED + 5..=LAND);
        carve(&mut tier, 7..=7, BED + 5..=sea - 1);
        carve(&mut tier, 9..=10, BED + 5..=LAND);
        // The build's flood leaves them all dry: nothing joins them to the
        // sea under sea level.
        let seeds = tier.sea_seeds(&Edits::new());
        assert!(tier.flood_from(seeds).is_empty());
        let reach = tier.sea_reach();
        let reached: Vec<usize> = (0..N).filter(|&slot| reach[slot]).collect();
        assert_eq!(
            reached,
            (1..=7).collect::<Vec<_>>(),
            "the sea, the beach, the pit, the cavern"
        );
        // The records, their heights the columns' tops, marked as a build
        // marks them.
        tier.relight();
        let mut finest: Vec<GpuCell> = (0..N)
            .map(|slot| {
                let mut cell = GpuCell::zeroed();
                cell.direction_height[3] = tier.columns[slot]
                    .surface()
                    .map_or(-100.0, |top| column::layer_altitude(top) + 1.0);
                cell
            })
            .collect();
        mark_sea(&tier, &mut finest);
        for (slot, cell) in finest.iter().enumerate().take(8).skip(1) {
            assert!(sheet_listed(cell), "the sheet lies over slot {slot}");
        }
        // The cavern's roof stands over the high water, and the sheet lies
        // under it all the same, shaded as dark as the cavern is.
        assert!(finest[7].direction_height[3] > super::super::terrain::high_water_m());
        assert!(finest[7].spare[2] >> SHEET_SHADE_SHIFT & 0xf > 0, "shaded");
        assert_eq!(finest[4].spare[2] >> SHEET_SHADE_SHIFT & 0xf, 0, "open sea");
        for (slot, cell) in finest.iter().enumerate().take(11).skip(9) {
            assert!(!sheet_listed(cell), "no sheet over the inland pit {slot}");
            assert!(cell.spare[2] & DRY_SHEET_BIT != 0);
        }
        // The sea's top in the open sea is the sheet's, not the mesher's.
        let mesh = crate::water_flow::liquid_mesh(&tier, &finest, PLANET_RADIUS);
        assert_eq!(mesh.tops, 0, "no second surface under the sheet");
    }

    /// An edit into the sea (`dry-holes` design 4): digging the rock between
    /// the flooded tunnel and the pocket fills the opened cell from the sea at
    /// once, and the rule brings the sea on into the pocket, a cell a tick,
    /// to the same water the old flood made at once, never climbing past the
    /// hole it came through.
    #[test]
    fn digging_through_to_the_sea_lets_it_flow_in() {
        let mut tier = line();
        carve(&mut tier, 5..=7, BED + 5..=BED + 7);
        carve(&mut tier, 9..=10, BED + 5..=BED + 7);
        let seeds = tier.sea_seeds(&Edits::new());
        tier.flood_from(seeds);
        tier.water.joined = tier.sea_joined();
        let mut edits = Edits::new();
        edits.set(Edit {
            cell: 108,
            layer: (BED + 6) as u16,
            material: Material::Air,
        });
        tier.columns[8].set(BED + 6, Material::Air);
        let filled = tier.water_after_edit(8, BED + 6, &edits);
        assert_eq!(filled, vec![8], "the opened cell, from the sea beside it");
        assert_eq!(tier.columns[8].material(BED + 6), Material::Water);
        assert_eq!(
            tier.columns[9].material(BED + 6),
            Material::Air,
            "the pocket is not filled by the edit"
        );
        assert!(!tier.water.queue.is_empty(), "and the edit is queued");
        // The edit's caller marks the sea after it (`FineSet::mark_sea`).
        tier.water.joined = tier.sea_joined();
        let mut ticks = 0;
        while !tier.water.queue.is_empty() && ticks < 50 {
            tier.tick_water(liquid::LOOP_MAX);
            ticks += 1;
            if ticks == 1 {
                assert!(
                    tier.columns[10].material(BED + 6) != Material::Water,
                    "the far end is not reached on the first tick"
                );
            }
        }
        for slot in 9..=10 {
            for layer in BED + 5..=BED + 7 {
                assert_eq!(
                    tier.columns[slot].material(layer),
                    if layer == BED + 7 {
                        Material::Air
                    } else {
                        Material::Water
                    },
                    "slot {slot} layer {layer}: the sea to the hole's height, no higher"
                );
            }
        }
    }

    /// Dig `layers` out of `slot`'s column, top down, through the edit's
    /// water path, as `FineSet::write_layer` does, then mark the sea as the
    /// edit's caller does.
    fn dig(
        tier: &mut ColumnTier,
        edits: &mut Edits,
        slot: usize,
        layers: std::ops::RangeInclusive<usize>,
    ) {
        for layer in layers.rev() {
            edits.set(Edit {
                cell: 100 + slot as u32,
                layer: layer as u16,
                material: Material::Air,
            });
            tier.columns[slot].set(layer, Material::Air);
            tier.water_after_edit(slot, layer, edits);
            tier.water.joined = tier.sea_joined();
        }
    }

    /// The records of the line, their heights the columns' tops, marked for
    /// the sea's sheet as a build or an edit marks them.
    fn marked(tier: &mut ColumnTier) -> Vec<GpuCell> {
        tier.relight();
        let mut finest: Vec<GpuCell> = (0..N)
            .map(|slot| {
                let mut cell = GpuCell::zeroed();
                cell.direction_height[3] = tier.columns[slot]
                    .surface()
                    .map_or(-100.0, |top| column::layer_altitude(top) + 1.0);
                cell
            })
            .collect();
        mark_sea(tier, &mut finest);
        finest
    }

    fn settle(tier: &mut ColumnTier) -> usize {
        let mut ticks = 0;
        while !tier.water.queue.is_empty() && ticks < 200 {
            tier.tick_water(liquid::LOOP_MAX);
            ticks += 1;
        }
        ticks
    }

    /// `dry-holes` (the owner, 2026-11-18: "We should be able to dig holes
    /// below water line and not have it filled with water too"): a pit dug
    /// from the land beside a beach at the sea's level, its floor four
    /// layers under the sea's, joins no water. It stays air after the rule
    /// has run, the sea's reach does not take it, no sheet is listed over it
    /// and it is marked dry; the same hollow made by the generator (no edit)
    /// is still under the sea's sheet (`ocean-environments` B22).
    #[test]
    fn a_pit_dug_below_the_sea_beside_a_beach_stays_dry() {
        let sea = sea_layer();
        let mut tier = line();
        // Column 5 a beach whose sand tops out at the sea's level, the air
        // over it the sea's reach; 6 and 7 dug from the land to four layers
        // under the sea's level.
        carve(&mut tier, 5..=5, sea..=LAND);
        let generated = {
            let mut t = tier.clone();
            carve(&mut t, 6..=7, sea - 4..=LAND);
            t
        };
        let mut edits = Edits::new();
        for slot in 6..=7 {
            dig(&mut tier, &mut edits, slot, sea - 4..=LAND);
        }
        assert!(tier.water.dug[6] && tier.water.dug[7] && !tier.water.dug[5]);
        settle(&mut tier);
        for slot in 6..=7 {
            for layer in sea - 4..=LAND {
                assert_eq!(
                    tier.columns[slot].material(layer),
                    Material::Air,
                    "slot {slot} layer {layer} stays dry"
                );
            }
        }
        let reach = tier.sea_reach();
        assert!(reach[5], "the beach is the sea's reach");
        assert!(!reach[6] && !reach[7], "the dug pit is not");
        let finest = marked(&mut tier);
        for (slot, cell) in finest.iter().enumerate().take(8).skip(6) {
            assert!(!sheet_listed(cell), "no sheet over the pit {slot}");
            assert!(cell.spare[2] & DRY_SHEET_BIT != 0, "dry {slot}");
        }
        assert!(sheet_listed(&finest[5]), "the sea's sheet over the beach");
        // The generator's own hollow there keeps B22's sheet.
        let mut generated = generated;
        let finest = marked(&mut generated);
        assert!(sheet_listed(&finest[6]) && sheet_listed(&finest[7]));
    }

    /// `dry-holes` design 4: a pit dug behind a wall is dry; breaching the
    /// wall at the sea's top layer lets the sea in. It pours over the lip as
    /// falling water first, then fills the pit from its floor up to the sea's
    /// level and no higher, and only then is the sea's sheet listed over it.
    #[test]
    fn a_breached_pit_fills_from_the_sea_over_its_lip() {
        let sea = sea_layer();
        let mut tier = line();
        let mut edits = Edits::new();
        for slot in 6..=7 {
            dig(&mut tier, &mut edits, slot, sea - 4..=LAND);
        }
        settle(&mut tier);
        assert_eq!(tier.columns[6].material(sea - 4), Material::Air, "dry");
        // The wall, column 5, cut down to the sea's top layer.
        dig(&mut tier, &mut edits, 5, sea - 1..=LAND);
        assert_eq!(
            tier.columns[5].material(sea - 1),
            Material::Water,
            "the breach fills from the sea beside it"
        );
        tier.tick_water(liquid::LOOP_MAX);
        assert_eq!(
            tier.columns[6].material(sea - 1).flow(),
            Some((7, true)),
            "the sea falls over the lip"
        );
        // Part way: the pit's top layer is not yet the sea's, so no sheet.
        tier.tick_water(liquid::LOOP_MAX);
        let finest = marked(&mut tier);
        assert!(!sheet_listed(&finest[7]), "no sheet while it fills");
        let ticks = settle(&mut tier);
        assert!(ticks > 2, "it fills over ticks, not at once");
        for slot in 6..=7 {
            for layer in sea - 4..sea {
                assert_eq!(
                    tier.columns[slot].material(layer),
                    Material::Water,
                    "slot {slot} layer {layer}: the sea's"
                );
            }
            assert_eq!(tier.columns[slot].material(sea), Material::Air, "no higher");
        }
        let finest = marked(&mut tier);
        assert!(sheet_listed(&finest[6]) && sheet_listed(&finest[7]));
    }

    /// `dry-holes` design 4: a channel two layers deep dug inland from the
    /// shore, then opened to the sea, fills along its length by the rule,
    /// a cell every tick or two, to the sea's level.
    #[test]
    fn a_channel_cut_from_the_shore_fills_along_it() {
        let sea = sea_layer();
        let mut tier = line();
        let mut edits = Edits::new();
        for slot in 6..=10 {
            dig(&mut tier, &mut edits, slot, sea - 2..=LAND);
        }
        // The dry channel, its queue run out, as the game's ticks leave it.
        settle(&mut tier);
        dig(&mut tier, &mut edits, 5, sea - 2..=LAND);
        assert_eq!(tier.columns[5].material(sea - 2), Material::Water);
        assert_ne!(tier.columns[7].material(sea - 2), Material::Water);
        let mut ticks = 0;
        while tier.columns[10].material(sea - 2) != Material::Water && ticks < 100 {
            tier.tick_water(liquid::LOOP_MAX);
            ticks += 1;
        }
        assert!(
            (5..=12).contains(&ticks),
            "five cells in {ticks} ticks: the rule's pace"
        );
        settle(&mut tier);
        for slot in 5..=10 {
            assert_eq!(tier.columns[slot].material(sea - 1), Material::Water);
            assert_eq!(tier.columns[slot].material(sea - 2), Material::Water);
            assert_eq!(tier.columns[slot].material(sea), Material::Air);
        }
        assert_eq!(tier.columns[11].material(sea - 2), Material::Stone, "rim");
    }

    /// `dry-holes` design 4: a source poured into a sealed pit under the
    /// sea's level is the bucket's, not the sea's: it runs by Luanti's rule
    /// and makes no new source.
    #[test]
    fn a_bucket_in_a_sealed_pit_under_the_sea_is_not_the_sea() {
        let sea = sea_layer();
        let mut tier = line();
        let mut edits = Edits::new();
        for slot in 6..=9 {
            dig(&mut tier, &mut edits, slot, sea - 4..=LAND);
        }
        let floor = sea - 4;
        edits.set(Edit {
            cell: 106,
            layer: floor as u16,
            material: Material::Water,
        });
        tier.columns[6].set(floor, Material::Water);
        tier.water_after_edit(6, floor, &edits);
        tier.water.joined = tier.sea_joined();
        settle(&mut tier);
        assert_eq!(tier.columns[7].material(floor), Material::Flow7);
        assert_eq!(tier.columns[8].material(floor), Material::Flow6);
        assert_eq!(tier.columns[6].material(floor + 1), Material::Air);
    }

    /// A poured source on land runs by the rule over the tier, a level less
    /// a cell, and stops at the rim; a world whose edits hold no water queues
    /// nothing, so it does not move (`water-flow` decision 5, E4).
    #[test]
    fn a_poured_source_runs_by_the_rule_and_an_old_world_queues_nothing() {
        let mut old = line();
        let mut edits = Edits::new();
        edits.set(Edit {
            cell: 106,
            layer: (LAND - 3) as u16,
            material: Material::Air,
        });
        old.queue_edited_water(&edits);
        assert!(
            old.water.queue.is_empty(),
            "no water in the edits: nothing queued"
        );
        assert!(old.tick_water(liquid::LOOP_MAX).is_empty());

        let mut tier = line();
        let at = LAND + 1;
        tier.columns[7].set(at, Material::Water);
        edits.set(Edit {
            cell: 107,
            layer: at as u16,
            material: Material::Water,
        });
        tier.queue_edited_water(&edits);
        let mut ticks = 0;
        while !tier.water.queue.is_empty() && ticks < 50 {
            tier.tick_water(liquid::LOOP_MAX);
            ticks += 1;
        }
        let m = |slot: usize| tier.columns[slot].material(at);
        assert_eq!(m(6), Material::Flow7);
        assert_eq!(m(8), Material::Flow7);
        assert_eq!(m(5), Material::Flow6);
        assert_eq!(m(9), Material::Flow6);
        assert_eq!(m(10), Material::Flow5);
        assert_eq!(m(11), Material::Air, "the rim is neutral");
        // Column 4 is the sea, far below: the sheet spreads into the air
        // over it on its own layer, and falls from there.
        assert!(m(4).flow().is_some());
        assert!(
            tier.columns[4]
                .material(at - 1)
                .flow()
                .is_some_and(|(_, falling)| falling)
                || tier.columns[4].material(at - 1).is_water(),
            "and falls toward the sea"
        );
        // The record carries the new water top.
        assert_eq!(water_top_layer(&tier.columns[10]), Some(at));
    }
}

/// Version 8's rivers in a real tier (design 13, `water-flow`): what the
/// build leaves is what the rule keeps.
#[cfg(test)]
mod river_tier_tests {
    use super::*;
    use crate::planet::lod;

    /// A river with a fall of three layers or more, as the drainage map has
    /// it, for an anchor.
    fn falls() -> Vec<Vec3> {
        let cfg = terrain_config();
        let settings = cfg.rivers.expect("the default generator has rivers");
        let map = pbd_core::rivers::map(cfg, &settings);
        let falls: Vec<Vec3> = map
            .rivers()
            .map(|k| map.describe(k))
            .filter(|&(c, level, drop, _)| drop >= 3 && level >= 3 && c.y.abs() < 0.7)
            .map(|(c, ..)| c)
            .step_by(97)
            .take(4)
            .collect();
        assert!(!falls.is_empty(), "a river with a fall");
        falls
    }

    /// A freshly built tier over a river: its water does not move when the
    /// rule runs over all of it again, no water stands over open air, and
    /// some of it flows (the river is not still pools).
    #[test]
    fn a_freshly_built_tier_s_rivers_are_what_the_rule_keeps() {
        for anchor in falls() {
            a_tier_s_rivers_are_what_the_rule_keeps(anchor);
        }
    }

    /// The settled water kept from one tier and stamped by the next is the
    /// water that tier settles fresh (`river-cache` design 11): a tier 40 m
    /// on from one already built, cold and stamped, cell for cell.
    #[test]
    fn a_stamped_tier_s_rivers_are_the_fresh_ones() {
        use std::sync::atomic::Ordering;
        let mut stamped_any = 0;
        for anchor in falls() {
            let east = Vec3::Y.cross(anchor).normalize_or(Vec3::X);
            let on = (anchor + east * (40.0 / PLANET_RADIUS)).normalize();
            SETTLE_CACHE_OFF.store(true, Ordering::Relaxed);
            let fresh = lod::generate_fine(on, &ColumnSettings::default(), &Edits::new()).columns;
            SETTLE_CACHE_OFF.store(false, Ordering::Relaxed);
            let _ = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
            let stamped = lod::generate_fine(on, &ColumnSettings::default(), &Edits::new()).columns;
            assert_eq!(fresh.columns.len(), stamped.columns.len());
            let cache = pbd_core::rivers::field(terrain_config()).expect("the cache");
            stamped_any += stamped
                .water
                .keys
                .iter()
                .filter(|k| cache.settled.get(k).is_some())
                .count();
            let differ: Vec<(usize, usize)> = (0..fresh.columns.len())
                .flat_map(|slot| (0..LAYERS).map(move |layer| (slot, layer)))
                .filter(|&(slot, layer)| {
                    fresh.columns[slot].material(layer) != stamped.columns[slot].material(layer)
                })
                .collect();
            assert!(
                differ.is_empty(),
                "{} cells differ, first {:?}",
                differ.len(),
                &differ[..differ.len().min(6)]
            );
        }
        assert!(stamped_any > 0, "no tier found kept water to stamp");
    }

    fn a_tier_s_rivers_are_what_the_rule_keeps(anchor: Vec3) {
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let mut tier = set.columns.clone();
        let sea = sea_layer();
        let (mut sources, mut flowing, mut over_air) = (0, 0, Vec::new());
        let mut queue = liquid::Queue::new();
        for slot in 0..tier.columns.len() {
            if tier.rim(slot) {
                continue;
            }
            for layer in sea..LAYERS - 1 {
                let m = tier.columns[slot].material(layer);
                if !m.is_water() {
                    continue;
                }
                if m == Material::Water {
                    sources += 1;
                } else {
                    flowing += 1;
                }
                if tier.columns[slot].material(layer - 1) == Material::Air {
                    over_air.push((slot, layer, m));
                }
                queue.push(water_id(slot, layer));
            }
        }
        eprintln!("river tier: {sources} sources, {flowing} flowing over the sea's level");
        // No flowing water outside a river's banks (pass 5): the valley,
        // its levee and the sealed rock under both hold the river, at a
        // fall's foot as anywhere. Over the sea's layer: the sea's own
        // water spreading into a sea cave is the sea's, not a river's.
        let cfg = terrain_config();
        let outside: Vec<_> = (0..tier.columns.len())
            .filter(|&slot| !tier.rim(slot))
            .flat_map(|slot| (sea + 1..LAYERS - 1).map(move |layer| (slot, layer)))
            .filter(|&(slot, layer)| {
                tier.columns[slot].material(layer).flow().is_some()
                    && pbd_core::rivers::at(cfg, tier.centres[slot]).is_none()
            })
            .collect();
        assert!(
            outside.is_empty(),
            "{} flowing cells outside a river's banks, first {:?}",
            outside.len(),
            &outside[..outside.len().min(6)]
        );
        assert!(sources > 50, "a river in the tier: {sources}");
        assert!(flowing > 0, "the river flows somewhere: {flowing}");
        assert!(
            over_air.is_empty(),
            "water over open air: {:?}",
            &over_air[..over_air.len().min(8)]
        );
        let mut changed = Vec::new();
        let mut ticks = 0;
        while !queue.is_empty() && ticks < 64 {
            queue.tick(
                &mut TierGrid(&mut tier),
                liquid::Rules::default(),
                liquid::LOOP_MAX,
                &mut changed,
            );
            ticks += 1;
        }
        assert!(
            changed.is_empty(),
            "the rule changed {} cells of a fresh tier, first {:?}",
            changed.len(),
            changed
                .iter()
                .take(6)
                .map(|&(id, c)| (water_cell(id), c))
                .collect::<Vec<_>>()
        );
    }
}

/// A measurement instrument for `river-cache` (design 1): what a world's
/// rivers cost the CPU when a tier is built, split into the drainage map
/// (once a world), the river part of the columns, and the rule's settle at
/// the build. CPU time on whatever machine runs it, never a frame time.
/// Ignored because it takes seconds; run it with
/// `cargo test -p pbd-app --profile fast --lib river_cost -- --ignored --nocapture --test-threads 1`.
#[cfg(test)]
mod river_cost {
    use super::*;
    use crate::planet::lod;
    use std::time::Instant;

    fn ms(t: Instant) -> f64 {
        t.elapsed().as_secs_f64() * 1000.
    }

    #[test]
    #[ignore]
    fn what_a_world_s_rivers_cost_to_build() {
        let cfg = terrain_config();
        let settings = cfg.rivers.expect("the default generator has rivers");
        // The drainage map, built fresh (the game builds it once a world).
        for threads in [
            1,
            std::thread::available_parallelism().map_or(1, |n| n.get()),
        ] {
            let started = Instant::now();
            let fresh = pbd_core::rivers::RiverMap::build_on(cfg, &settings, threads);
            eprintln!(
                "drainage map: {} river cells, built in {:.0} ms ({threads} threads)",
                fresh.rivers().count(),
                ms(started)
            );
        }
        let map = pbd_core::rivers::map(cfg, &settings);
        // River anchors with a fall, as `river_tier_tests` picks them, and a
        // dry inland anchor for the floor.
        let mut anchors: Vec<(String, Vec3)> = map
            .rivers()
            .map(|k| map.describe(k))
            .filter(|&(c, level, drop, _)| drop >= 3 && level >= 3 && c.y.abs() < 0.7)
            .map(|(c, ..)| c)
            .step_by(97)
            .take(4)
            .enumerate()
            .map(|(i, c)| (format!("river {i}"), c))
            .collect();
        anchors.push((
            "dry".into(),
            Vec3::new(0.8772014, 0.48012277, 0.0).normalize(),
        ));
        let dry = pbd_core::planet_gen::TerrainConfig {
            rivers: None,
            ..*cfg
        };
        // The drainage map's ground: one altitude per lattice cell, which is
        // most of its build.
        let lattice: Vec<Vec3> = (0..720 * 360)
            .map(|k| {
                let (i, j) = (k % 720, k / 720);
                let lat = (-90.0 + (j as f32 + 0.5) * 0.5f32).to_radians();
                let lon = (-180.0 + (i as f32 + 0.5) * 0.5f32).to_radians();
                Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
            })
            .collect();
        let started = Instant::now();
        for &d in &lattice {
            std::hint::black_box(pbd_core::planet_gen::surface_altitude(&dry, d));
        }
        eprintln!(
            "drainage map's ground alone: {} altitudes in {:.0} ms (one thread)",
            lattice.len(),
            ms(started)
        );
        for (name, anchor) in anchors {
            let started = Instant::now();
            let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
            let whole = ms(started);
            // The same set again, 40 m on: a rebuild over ground the first
            // set's job already asked about (`river-cache` design 11: what
            // the cache saves a walker's next rebuild).
            let east = Vec3::Y.cross(anchor).normalize_or(Vec3::X);
            let on = (anchor + east * (40.0 / PLANET_RADIUS)).normalize();
            let started = Instant::now();
            std::hint::black_box(lod::generate_fine(
                on,
                &ColumnSettings::default(),
                &Edits::new(),
            ));
            eprintln!(
                "{name}: the next set 40 m on {:.0} ms (the first {whole:.0} ms), cache {}",
                ms(started),
                if pbd_core::river_cache::enabled() {
                    "on"
                } else {
                    "off"
                }
            );
            // Every fine record's altitude, with the river terms and without.
            let records: Vec<Vec3> = set
                .levels
                .iter()
                .flatten()
                .map(|c| Vec3::from_slice(&c.direction_height[..3]))
                .collect();
            let started = Instant::now();
            for &d in &records {
                std::hint::black_box(pbd_core::planet_gen::surface_altitude(cfg, d));
            }
            let rec_with = ms(started);
            let started = Instant::now();
            for &d in &records {
                std::hint::black_box(pbd_core::planet_gen::surface_altitude(&dry, d));
            }
            let rec_without = ms(started);
            let started = Instant::now();
            for &d in &records {
                std::hint::black_box(pbd_core::rivers::far_surface(cfg, d, 10.0));
            }
            eprintln!(
                "{name}: rivers::far_surface over the records {:.0} ms (one thread)",
                ms(started)
            );
            // The tier's columns generated with the river terms and without.
            let field = ColumnSettings::default().worms();
            let region = worms::gather(&field, cfg, anchor, ColumnSettings::default().reach_m);
            let started = Instant::now();
            for &d in &set.columns.centres {
                std::hint::black_box(column::generate_edited(&region, &field, cfg, d, &[]));
            }
            let col_with = ms(started);
            let started = Instant::now();
            for &d in &set.columns.centres {
                std::hint::black_box(column::generate_edited(&region, &field, &dry, d, &[]));
            }
            let col_without = ms(started);
            // The flowing water's mesh, made on the frame a set lands.
            let started = Instant::now();
            let mesh =
                crate::water_flow::liquid_mesh(&set.columns, set.finest_records(), PLANET_RADIUS);
            let meshed = ms(started);
            eprintln!(
                "{name}: generate_fine whole {whole:.0} ms ({} records); their altitudes \
                 {rec_with:.0} ms with rivers, {rec_without:.0} without (one thread); tier \
                 columns {col_with:.0} ms with rivers, {col_without:.0} without; water mesh \
                 {meshed:.1} ms, {} cells",
                records.len(),
                mesh.cells,
            );
            // Best of three, each from the records as the set was laid.
            let mut best: Option<(BuildPhases, f64, ColumnTier)> = None;
            for _ in 0..3 {
                let mut finest = set.levels[3].clone();
                let started = Instant::now();
                let tier = build(
                    anchor,
                    &mut finest,
                    &set.finest_neighbors,
                    &ColumnSettings::default(),
                    &Edits::new(),
                );
                let total = ms(started);
                let phases = LAST_BUILD.with(|c| c.get());
                if best.as_ref().is_none_or(|b| total < b.1) {
                    best = Some((phases, total, tier));
                }
            }
            let (p, total, mut tier) = best.expect("three builds");
            // The river part of the columns: the carve every altitude asks
            // and the water each column stands, over the tier's directions.
            let dirs = tier.centres.clone();
            let started = Instant::now();
            for &d in &dirs {
                std::hint::black_box(pbd_core::planet_gen::surface_altitude(cfg, d));
            }
            let with = ms(started);
            let started = Instant::now();
            for &d in &dirs {
                std::hint::black_box(pbd_core::planet_gen::surface_altitude(&dry, d));
            }
            let without = ms(started);
            let started = Instant::now();
            let mut sources = 0;
            for &d in &dirs {
                sources += pbd_core::rivers::water(cfg, d).len();
            }
            let water = ms(started);
            // A second settle over the settled tier: what a check costs.
            let started = Instant::now();
            let again = tier.settle_rivers(&Edits::new());
            let check = ms(started);
            eprintln!(
                "{name}: {} columns, build {total:.1} ms = columns {:.1} + flood {:.1} + \
                 settle {:.1} + relight {:.1}; settle queued {} cells, {} ticks, {} sweeps, \
                 changed {}; one altitude per column {with:.1} ms with rivers, {without:.1} \
                 without; rivers::water {water:.1} ms ({sources} sources); settle again \
                 {check:.1} ms, changed {again}",
                tier.columns.len(),
                p.columns_ms,
                p.flood_ms,
                p.settle_ms,
                p.relight_ms,
                p.settle_queued,
                p.settle_ticks,
                p.settle_sweeps,
                p.settle_changed,
            );
        }
    }
}

/// `placed-blocks-keep-material`: what the tier DRAWS for a block the player
/// placed, after the save has been written and read back (the owner,
/// 2026-11-16: "I placed dirt on sand and it became sand ... not just for
/// dirt block but any block in inventory").
#[cfg(test)]
mod placed_block_tests {
    use super::*;
    use crate::saves::{WorldSave, create};
    use pbd_core::edits::Edit;
    use pbd_core::inventory::Slots;
    use pbd_core::planet_gen::{self, Biome};

    /// Every block a pack holds and puts down: everything with a picture of
    /// its own but the sea, which is poured from a bucket.
    fn placeable() -> impl Iterator<Item = Material> {
        Material::ALL
            .into_iter()
            .filter(|&m| column::home_biome(m).is_some() && !m.is_water())
    }

    fn find(want: impl Fn(Vec3, &Column) -> bool) -> Vec3 {
        (0..60_000)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / 60_000.0;
                let r = (1.0 - y * y).sqrt();
                let phi = i as f32 * 2.399_963;
                Vec3::new(r * phi.cos(), y, r * phi.sin())
            })
            .find(|&d| want(d, &column::generate_solid(terrain_config(), d)))
            .expect("the shipped world has this ground")
    }

    /// The grounds the owner's rule names, on the shipped world. A town's
    /// lane is stood in for by the desert's ground with its top a placed
    /// lane's stone: the town is installed by the running game, and its
    /// ground reaches the record only through the column, as these do.
    fn grounds() -> Vec<(&'static str, Vec3)> {
        let cfg = terrain_config();
        let top = |c: &Column| c.surface().map(|t| c.material(t));
        let biome = |d: Vec3| planet_gen::biome_at(cfg, d, column::surface_m(cfg, d));
        vec![
            (
                "desert sand",
                find(|d, c| {
                    top(c) == Some(Material::Sand)
                        && biome(d) == Biome::Desert
                        && !c.water_over(c.surface().unwrap())
                }),
            ),
            ("snow", find(|_, c| top(c) == Some(Material::Snow))),
            (
                "a river bed",
                find(|d, c| {
                    pbd_core::rivers::floods(cfg, d)
                        && c.water_over(c.surface().unwrap())
                        && column::surface_m(cfg, d) > cfg.sea_level_m
                }),
            ),
            (
                "a beach under the sea",
                find(|d, c| {
                    top(c) == Some(Material::Sand)
                        && column::surface_m(cfg, d) < cfg.sea_level_m - 1.0
                }),
            ),
        ]
    }

    /// One nibble of a packed per-layer buffer.
    fn nibble(words: &[u32], layer: usize) -> u32 {
        words[layer / MATERIAL_PER_WORD] >> ((layer % MATERIAL_PER_WORD) * 4) & 0xf
    }

    /// Place every block on every ground, write the save and read it back,
    /// build the column the tier builds from it, and hold what is drawn: the
    /// cap's code and sheet in the record, and the layer's code and sheet in
    /// the column pass's buffers. The ground under it is drawn as it was.
    #[test]
    fn a_placed_block_is_drawn_as_itself_after_a_reload() {
        let root = std::env::temp_dir().join(format!("pbd-placed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = create(&root, "Placed", 4242).unwrap();
        let grounds = grounds();
        let mut cases = Vec::new();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            for (g, &(_, d)) in grounds.iter().enumerate() {
                let generated = column::generate_solid(terrain_config(), d);
                let under = generated.surface().unwrap();
                for (i, material) in placeable().enumerate() {
                    // A cell of its own each, and two blocks: the one on the
                    // ground and one placed on top of it (a dirt block, or
                    // stone on dirt).
                    let cell = (g * 100 + i) as u32 + 1;
                    let second = if material == Material::Dirt {
                        Material::Stone
                    } else {
                        Material::Dirt
                    };
                    for (layer, m) in [(under + 1, material), (under + 2, second)] {
                        assert!(save.accept(
                            Edit {
                                cell,
                                layer: layer as u16,
                                material: m,
                            },
                            None,
                            &Slots::new()
                        ));
                    }
                    cases.push((g, d, cell, under, material, second));
                }
            }
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), slot);
        for (g, d, cell, under, material, second) in cases {
            let what = format!("{material:?} on {}", grounds[g].0);
            let edits = reopened.edits.for_cell(cell);
            assert_eq!(
                edits,
                &[(under as u16 + 1, material), (under as u16 + 2, second)],
                "{what}: the save keeps what was placed"
            );
            let built = column::generate_edited_solid(terrain_config(), d, edits);
            let height = super::super::terrain::surface_height(d);
            let word = super::super::terrain::surface_code(d, height);
            for (layer, m, on_top) in [(under + 1, material, false), (under + 2, second, true)] {
                let what = format!("{what}, layer {layer} {m:?}");
                // The column the tier builds holds it, placed.
                assert_eq!(built.material(layer), m, "{what}: stored");
                assert!(built.placed(layer), "{what}: placed");
                // Ore carries the ore mark, not its home sheet (design 12.12).
                // A sea cell, its own sheet (`ocean-environments` B24).
                let sheet = if m == Material::Ore {
                    super::super::terrain::ORE_SHEET
                } else if let Some(own) = column::own_sheet(m) {
                    super::super::terrain::sheet_index(own) + 1
                } else {
                    tileset_slot(column::home_biome(m).unwrap()) + 1
                };
                // The column pass draws it in its own code, from its own sheet.
                let tier = ColumnTier::for_tests(vec![built.clone()], &[[NO_NEIGHBOR; 6]]);
                let codes = tier.gpu_materials();
                let sheets = tier.gpu_placed_sheets();
                assert_eq!(
                    nibble(&codes, layer),
                    render_code(m) & 0xf,
                    "{what}: its code"
                );
                assert_eq!(nibble(&sheets, layer), sheet, "{what}: its own sheet");
                assert_eq!(
                    nibble(&sheets, under),
                    0,
                    "{what}: the ground under it is the biome's"
                );
                // And the cap, which the record draws: the top block's.
                let mut column_top = built.clone();
                if !on_top {
                    column_top.set(under + 2, Material::Air);
                }
                assert_eq!(column_top.surface(), Some(layer));
                let mut finest = vec![<GpuCell as Zeroable>::zeroed()];
                finest[0].metadata[1] = word;
                reconcile_surface(&mut finest, &[], 0, &column_top);
                let record = finest[0].metadata[1];
                assert_eq!(record & 0xff, render_code(m), "{what}: the cap's code");
                assert_eq!(
                    record >> super::super::terrain::PLACED_SHEET_SHIFT & 0xf,
                    sheet,
                    "{what}: the cap's own sheet"
                );
                assert_eq!(
                    record & 0xff00,
                    word & 0xff00,
                    "{what}: the cell's biome stays its own"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A column of version 10's land that holds ore within 14 m of its top.
    fn ore_ground() -> (Vec3, Column, usize) {
        let cfg = terrain_config();
        let d = find(|d, c| {
            let top = c.surface().unwrap();
            column::surface_m(cfg, d) > cfg.sea_level_m + 2.0
                && !c.water_over(top)
                && (top.saturating_sub(14)..top).any(|l| c.material(l) == Material::Ore)
        });
        let column = column::generate_solid(terrain_config(), d);
        let top = column.surface().unwrap();
        let ore = (top.saturating_sub(14)..top)
            .rev()
            .find(|&l| column.material(l) == Material::Ore)
            .unwrap();
        (d, column, ore)
    }

    /// Design 12.12: a generated ore layer packs the ore mark (15) in the
    /// per-layer sheets and keeps the stone's render code; stone beside it
    /// packs nought; no sheet plus one reaches the mark.
    #[test]
    fn generated_ore_carries_the_ore_mark_and_the_stone_s_code() {
        use super::super::terrain::ORE_SHEET;
        let (_, column, ore) = ore_ground();
        let tier = ColumnTier::for_tests(vec![column.clone()], &[[NO_NEIGHBOR; 6]]);
        let codes = tier.gpu_materials();
        let sheets = tier.gpu_placed_sheets();
        assert_eq!(nibble(&sheets, ore), ORE_SHEET);
        assert_eq!(nibble(&codes, ore), render_code(Material::Stone));
        assert_eq!(drawn_sheet_mark(&column, ore), ORE_SHEET);
        assert_eq!(placed_sheet(&column, ore), 0, "nobody placed it");
        let stone = (0..LAYERS)
            .find(|&l| column.material(l) == Material::Stone)
            .unwrap();
        assert_eq!(nibble(&sheets, stone), 0);
        assert_eq!(nibble(&codes, stone), render_code(Material::Stone));
    }

    /// Design 12.12: an ore top laid bare by digging carries the mark in its
    /// record's sheet bits, keeps the stone's code and its cell's biome; a
    /// placed ore block carries it too, and placed stone still its own sheet.
    #[test]
    fn dug_and_placed_ore_carry_the_ore_mark() {
        use super::super::terrain::{ORE_SHEET, PLACED_SHEET_SHIFT};
        let (d, generated, ore) = ore_ground();
        let mut dug = generated.clone();
        let top = dug.surface().unwrap();
        for layer in ore + 1..=top {
            assert!(dug.place(layer, Material::Air));
        }
        assert_eq!(dug.surface(), Some(ore));
        let height = super::super::terrain::surface_height(d);
        let word = super::super::terrain::surface_code(d, height);
        let mut finest = vec![<GpuCell as Zeroable>::zeroed()];
        finest[0].metadata[1] = word;
        reconcile_surface(&mut finest, &[], 0, &dug);
        let record = finest[0].metadata[1];
        assert_eq!(
            record >> PLACED_SHEET_SHIFT & 0xf,
            ORE_SHEET,
            "the cap's mark"
        );
        assert_eq!(
            record & 0xff,
            render_code(Material::Stone),
            "the cap's code"
        );
        assert_eq!(record & 0xff00, word & 0xff00, "the cell's biome stays");

        // A block of ore placed on the dug top, and stone on that.
        let mut built = dug.clone();
        assert!(built.place(ore + 1, Material::Ore));
        assert!(built.place(ore + 2, Material::Stone));
        let tier = ColumnTier::for_tests(vec![built.clone()], &[[NO_NEIGHBOR; 6]]);
        let sheets = tier.gpu_placed_sheets();
        assert_eq!(nibble(&sheets, ore + 1), ORE_SHEET, "placed ore");
        assert_eq!(
            nibble(&sheets, ore + 2),
            tileset_slot(column::home_biome(Material::Stone).unwrap()) + 1,
            "placed stone keeps its own sheet"
        );
        assert_eq!(nibble(&sheets, ore + 3), 0, "air");
    }

    /// Ground nobody touched is drawn as it was: its record carries no
    /// placed sheet, and its column's layers none but the ore mark on its
    /// ore (design 12.12).
    #[test]
    fn untouched_ground_takes_its_biomes_sheet() {
        for (name, d) in grounds() {
            let column = column::generate_solid(terrain_config(), d);
            let tier = ColumnTier::for_tests(vec![column.clone()], &[[NO_NEIGHBOR; 6]]);
            let sheets = tier.gpu_placed_sheets();
            for layer in 0..LAYERS {
                let ore = column.material(layer) == Material::Ore;
                assert_eq!(
                    nibble(&sheets, layer),
                    if ore {
                        super::super::terrain::ORE_SHEET
                    } else {
                        0
                    },
                    "{name}, layer {layer}"
                );
            }
            let height = super::super::terrain::surface_height(d);
            let word = super::super::terrain::surface_code(d, height);
            let mut finest = vec![<GpuCell as Zeroable>::zeroed()];
            finest[0].metadata[1] = word;
            reconcile_surface(&mut finest, &[], 0, &column);
            assert_eq!(
                finest[0].metadata[1] >> super::super::terrain::PLACED_SHEET_SHIFT & 0xf,
                0,
                "{name}"
            );
        }
    }
}
