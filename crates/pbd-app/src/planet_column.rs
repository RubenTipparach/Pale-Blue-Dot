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
use super::terrain::{PLANET_RADIUS, TERRAIN, render_code};
use crate::config::ColumnSettings;
use bevy::prelude::Vec3;
use bytemuck::{Pod, Zeroable};
use pbd_core::column::{self, Column, LAYERS, MAX_RUNS};
use pbd_core::edits::Edits;
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
    /// Sides 4 and 5, then the cell's own degree, then spare.
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
}

impl ColumnTier {
    /// An empty tier, which is every cell answering from the heightfield.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            slots: Vec::new(),
            records: Vec::new(),
            light: Vec::new(),
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
        if let Some(entry) = self.records.get_mut(slot) {
            entry.runs = runs;
        }
    }

    /// What the vertex shader reads, one record per slot.
    pub(crate) fn gpu_records(&self) -> &[GpuColumn] {
        &self.records
    }

    /// The sky level of one cell, 0 to `light::MAX`.
    pub fn sky(&self, slot: usize, layer: usize) -> u8 {
        self.light
            .get(slot)
            .and_then(|levels| levels.get(layer))
            .copied()
            .unwrap_or(0)
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
                words[word] |= (level as u32) << shift;
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
        let sides = self.neighbor_slots();
        self.light = light::bake(&light::Region {
            columns: &self.columns,
            neighbors: &sides,
        });
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
    // The region's worms, gathered ONCE: every worm that could reach any
    // column of the tier, so each column's carve is complete whatever tier
    // built it. This is the regional pre-pass the design said worms need,
    // done at the one place columns are built in bulk.
    let field = settings.worms();
    let region = worms::gather(&field, &TERRAIN, anchor, settings.reach_m.max(0.));
    let reach = (settings.reach_m.max(0.) / PLANET_RADIUS).cos();
    let mut slots = vec![usize::MAX; finest.len()];
    let mut members = Vec::new();
    for (index, cell) in finest.iter_mut().enumerate() {
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
        let made = &edits.for_cell(cell.metadata[3]);
        let column = if rim {
            column::generate_edited_solid(&TERRAIN, direction, made)
        } else {
            column::generate_edited(&region, &field, &TERRAIN, direction, made)
        };
        records.push(GpuColumn {
            runs: column.packed_runs(render_code),
            neighbors: [sides[0], sides[1], sides[2], sides[3]],
            more: [sides[4], sides[5], degree as u32, rim as u32],
        });
        columns.push(column);
    }
    // A MOUTH lowers the ground. Where the carve broke the surface, the
    // column's top is below the height the record was built with, and the
    // record is what the terrain pass draws its cap from, what the neighbours
    // draw their walls down to, and what the walker stands on outside a cave.
    // Left alone it would draw a meadow over the hole and a wall across it.
    // So the record takes the column's top, its cap wears the material that
    // is actually there, and every neighbour's wall height for this side
    // follows. Outside a mouth the column top is the height rounded up by
    // less than a layer and the record is left exactly as it was, so the
    // surface keeps its one source everywhere the carve did not touch it.
    for (slot, &index) in members.iter().enumerate() {
        let column = &columns[slot];
        let Some(top) = column.surface() else {
            continue;
        };
        let top_m = column::layer_altitude(top) + 1.0;
        let height = finest[index].direction_height[3];
        if top_m >= height {
            continue;
        }
        finest[index].direction_height[3] = top_m;
        let code = render_code(column.material(top));
        finest[index].metadata[1] = (finest[index].metadata[1] & !0xff) | code;
        let Some(table) = neighbors.get(index) else {
            continue;
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
    let mut tier = ColumnTier {
        columns,
        slots,
        records,
        light: Vec::new(),
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
            // Either the column top is the height rounded up by under a layer,
            // or the record was lowered to the column's top: never a cap
            // drawn over air.
            assert!(
                (top_m >= height && top_m - height < 1.0001) || (top_m - height).abs() < 1e-4,
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
}
