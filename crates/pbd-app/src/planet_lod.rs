//! The LOD resident set: the base level once, the fine bands per anchor.
//!
//! Tiles are drawn at a level chosen from great-circle distance to the
//! player. Level 7 covers the globe and is uploaded once; levels 8 to 11 are
//! generated as bands around an anchor on the compute pool whenever the player
//! has moved `REGEN_DISTANCE_M` from the last one, and the fine regions of
//! the storage buffer are rewritten when the set lands. The partition rule
//! the GPU applies, and the record it reads, are in
//! `openspec/changes/hexagon-lod/design.md`, "Implementation decisions".

use super::GpuCell;
use super::column::{self, ColumnTier};
use super::lattice::{Lattice, LocalCell};
use super::terrain::{PLANET_RADIUS, surface_code, surface_height};
use super::topology::{DualCell, midpoint};
use crate::config::ColumnSettings;
use bevy::{
    prelude::*,
    render::extract_resource::ExtractResource,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use pbd_core::edits::Edits;
use std::{
    collections::HashMap,
    ops::Range,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

/// The coarsest level, resident for the whole globe: 163,842 cells.
pub const BASE_LEVEL: u8 = 7;
/// The tier a player stands on. `tile_width_m(FINEST_LEVEL)` is the gold
/// standard on the shipped radius; `lattice` pins it.
pub const FINEST_LEVEL: u8 = 11;
/// The fine levels in ascending order; index `k` is level `8 + k`.
pub const FINE_LEVELS: [u8; 4] = [8, 9, 10, 11];
/// Outer radius of each fine level's band, great-circle metres from the
/// player, in `FINE_LEVELS` order. Each halves as the tile halves.
pub const BAND_M: [f32; 4] = [2_400.0, 1_200.0, 600.0, 300.0];
/// Records per fine level in the storage buffer. The level-11 band is the
/// largest at about 52,000 cells.
pub const FINE_CAPACITY: u32 = 65_536;
/// How far the player walks from the anchor before the fine set is rebuilt.
/// The bands extend past their radii by this much so the live bands, which
/// move with the player, never leave the resident set.
pub const REGEN_DISTANCE_M: f32 = 40.0;
/// Metres of extra margin per metre of height above the ground. Aloft the
/// bands are laid this much wider and rebuilt this much less often: at flight
/// speeds a 40 m margin is a rebuild every few frames, and from a kilometre up
/// a band a few hundred metres behind the ship is detail no one can see is
/// stale (`far-side-flight`).
pub const REGEN_PER_HEIGHT: f32 = 0.3;

/// The rebuild margin at a height above the ground: the walk's
/// `REGEN_DISTANCE_M` on the ground, growing with height.
pub fn regen_m(height: f32) -> f32 {
    REGEN_DISTANCE_M + REGEN_PER_HEIGHT * height.max(0.0)
}

/// Mean centre-to-centre tile width at a level, metres, on the shipped radius.
pub fn tile_width_m(level: u8) -> f32 {
    1.2087 * PLANET_RADIUS / (1u32 << level) as f32
}

/// How far each fine band reaches across the ground, great-circle metres from
/// the point under the player, for a player `player_radius` from the planet's
/// centre over ground at `ground_radius`. A band is a SLANT radius: the ground
/// within `BAND_M[k]` of the player's eye, not of the ground under it. From
/// height `h` it reaches the ground out to about `sqrt(B^2 - h^2)`, exactly
/// by the triangle through the centre, and not at all once `h >= B`. So from a
/// kilometre up there is no finest band to build or draw: its nearest tile
/// would be a kilometre away (`far-side-flight`). Standing on the ground this
/// is `BAND_M` itself.
pub fn live_bands_m(player_radius: f32, ground_radius: f32) -> [f32; 4] {
    let height = (player_radius - ground_radius).max(0.0) as f64;
    // The triangle through the centre, drawn on the sphere every band is
    // measured on (great-circle metres at PLANET_RADIUS, as `metres_from_anchor`
    // measures), so on the ground it is `BAND_M` exactly. In f64: the cosine is
    // within 1e-5 of one, where f32's acos loses a metre in three hundred.
    let ground = PLANET_RADIUS as f64;
    let eye = ground + height;
    std::array::from_fn(|k| {
        let band = BAND_M[k] as f64;
        if height >= band {
            return 0.0;
        }
        let cos =
            ((eye * eye + ground * ground - band * band) / (2.0 * eye * ground)).clamp(-1.0, 1.0);
        ((cos.acos() * ground) as f32).min(BAND_M[k])
    })
}

/// The cosine of a fine level's nominal band radius. `LodParams::of` is what
/// the GPU actually compares against, since a truncated band is complete to
/// less than its nominal radius; this is the nominal value the tests pin.
#[cfg(test)]
pub fn band_cos(level: u8) -> f32 {
    let k = (level - FINE_LEVELS[0]) as usize;
    (BAND_M[k] / PLANET_RADIUS).cos()
}

/// A cell's inputs to its GPU record, whichever generator produced it.
struct CellSource<'a> {
    direction: Vec3,
    corners: &'a [Vec3],
    neighbor_directions: &'a [Vec3],
    level: u8,
    owners: [Vec3; 2],
    id: u32,
}

/// One height per direction, memoised: neighbours and edge midpoints are
/// shared between records and the noise is the expensive part.
#[derive(Default)]
struct Heights(HashMap<[u32; 3], f32>);

impl Heights {
    fn at(&mut self, direction: Vec3) -> f32 {
        *self
            .0
            .entry(direction.to_array().map(f32::to_bits))
            .or_insert_with(|| surface_height(direction))
    }
}

/// The record the shaders read. Neighbour heights are real, below sea level
/// included; the fine floor per side is the height at that edge's midpoint,
/// which is the level below's midpoint cell; owners are the level below's
/// cells this one belongs to.
/// How far down a wall must reach to meet the ground a FINER band draws along
/// this edge: the lowest cap on it, not the height at its middle.
///
/// The wall from this cell's cap to its neighbour's is what closes the step
/// between them, and where the neighbour's region is drawn one level finer it
/// has to reach the finer caps instead. That used to be a single sample, the
/// height at the shared edge's midpoint, and a finer band draws SEVERAL cells
/// along that edge: any of them lower than the one sample is ground the wall's
/// foot hangs above, and what shows through the gap is sky. Measured over four
/// thousand land edges at the spawn, the midpoint stood 0.45 m above the
/// lowest cap on average and 2.0 m at worst, with one edge in eighty short by
/// more than a whole cell - which is the slivers of sky between the terrace
/// rows of a far hillside.
///
/// Nine samples across the edge, which is finer than any band that can draw
/// against it, and the midpoint is one of them, so this can only ever reach
/// further down than it did.
fn fine_floor(here: Vec3, neighbor: Vec3, heights: &mut Heights) -> f32 {
    let mut floor = heights.at(midpoint(here, neighbor));
    for step in 1..FLOOR_SAMPLES {
        let f = step as f32 / FLOOR_SAMPLES as f32;
        floor = floor.min(heights.at((here * (1.0 - f) + neighbor * f).normalize()));
    }
    floor
}

/// Samples across a shared edge when measuring its fine floor. A band is one
/// level finer than the band it meets, so at most a couple of cells stand on
/// an edge; seventeen is well past that, and measured against a 64-sample
/// ground truth it leaves the floor 0.05 m high on average against the nine
/// samples' 0.11 m. The cost is paid once per record, at build time.
const FLOOR_SAMPLES: usize = 17;

/// Which sides of a record carry a fine floor.
///
/// The shader reads a floor in ONE place, the wall branch of
/// `planet_surface.wgsl`, and only on a level coarser than the finest, where
/// the neighbour across that side is inside the next finer band. Seventeen
/// samples on every side of every cell was ninety-seven percent of a sixteen
/// second rebuild, measured on the owner's desktop, and almost none of it was
/// ever read (`openspec/changes/fine-set-in-a-second/design.md`). A side the
/// rule skips carries the neighbour's own height, which is what its wall
/// reaches down to anyway, so even a read the rule missed draws the wall a
/// finest-level side draws.
#[derive(Clone, Copy)]
enum FloorRule {
    /// Every side: the base, which is built once and serves every anchor.
    Every,
    /// Sides whose neighbour's direction dotted with `anchor` exceeds `cos`,
    /// or, for the partition this set replaces, dotted with `also.0` exceeds
    /// `also.1`: the landing's cross-fade draws that one's walls too
    /// (`detail-fade`).
    Within {
        anchor: Vec3,
        cos: f32,
        also: Option<(Vec3, f32)>,
    },
    /// No side: the finest level, which nothing is drawn finer than.
    None,
}

impl FloorRule {
    fn reads(self, neighbor: Vec3) -> bool {
        match self {
            FloorRule::Every => true,
            FloorRule::Within { anchor, cos, also } => {
                neighbor.dot(anchor) > cos || also.is_some_and(|(a, c)| neighbor.dot(a) > c)
            }
            FloorRule::None => false,
        }
    }
}

fn record(source: CellSource, heights: &mut Heights, rule: FloorRule) -> GpuCell {
    let height = heights.at(source.direction);
    let degree = source.corners.len();
    let mut corners = [[0.; 4]; 6];
    let mut floors = [0.; 6];
    let mut occlusion = 0.;
    for side in 0..degree {
        let corner = source.corners[side];
        let neighbor = source.neighbor_directions[side];
        let neighbor_height = heights.at(neighbor);
        corners[side] = [corner.x, corner.y, corner.z, neighbor_height];
        floors[side] = if rule.reads(neighbor) {
            fine_floor(source.direction, neighbor, heights)
        } else {
            neighbor_height
        };
        let separation = (source.direction.distance(neighbor) * PLANET_RADIUS).max(1.);
        occlusion += ((neighbor_height - height) / separation).clamp(0., 1.);
    }
    let skylight = 1. - occlusion / degree as f32 * 0.55;
    let [a, b] = source.owners;
    GpuCell {
        direction_height: [
            source.direction.x,
            source.direction.y,
            source.direction.z,
            height,
        ],
        corners,
        metadata: [
            degree as u32 | (source.level as u32) << 8,
            surface_code(source.direction, height),
            (skylight * 65535.) as u32,
            source.id,
        ],
        owner_a: [a.x, a.y, a.z, floors[0]],
        owner_b: [b.x, b.y, b.z, floors[1]],
        floors: [floors[2], floors[3], floors[4], floors[5]],
        spare: [0.; 4],
    }
}

/// The base level's records from the whole-sphere dual. Owners are the cell
/// itself: the base has no level below it to be partitioned by.
///
/// Every side carries its floor, because the finer band the base meets moves
/// with every anchor; so this is the most expensive thing the planet builds,
/// and it is built on every core, in chunks joined in order.
pub fn base_records(cells: &[DualCell]) -> Vec<GpuCell> {
    let build = |range: Range<usize>| -> Vec<GpuCell> {
        let mut heights = Heights::default();
        let mut neighbors = Vec::with_capacity(6);
        range
            .map(|index| {
                let cell = &cells[index];
                neighbors.clear();
                neighbors.extend(cell.neighbors.iter().map(|&n| cells[n].direction));
                record(
                    CellSource {
                        direction: cell.direction,
                        corners: &cell.corners,
                        neighbor_directions: &neighbors,
                        level: BASE_LEVEL,
                        owners: [cell.direction; 2],
                        id: index as u32,
                    },
                    &mut heights,
                    FloorRule::Every,
                )
            })
            .collect()
    };
    let threads = build_threads();
    let chunk = cells.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..cells.len())
            .step_by(chunk)
            .map(|start| scope.spawn(move || build(start..(start + chunk).min(cells.len()))))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a chunk of the base"))
            .collect()
    })
}

/// The fine levels around one anchor, with the finest level's neighbour
/// table for the walker's contact.
#[derive(Clone)]
pub struct FineSet {
    pub anchor: Vec3,
    /// Records per level in `FINE_LEVELS` order, each at most `FINE_CAPACITY`.
    pub(crate) levels: [Vec<GpuCell>; 4],
    /// Neighbour indices into `levels[3]`, `u32::MAX` off the set.
    pub finest_neighbors: Vec<[u32; 6]>,
    /// The angular radius the finest level is complete to: the band plus the
    /// walk, or less when capacity truncated the band.
    finest_radius: f32,
    /// Per level, the radius in metres out to which this level is resident AND
    /// complete, clamped to its nominal band. This is the radius the partition
    /// hides the coarser level inside, so what is hidden always has a
    /// replacement: a truncated band stops hiding where it stops existing.
    complete: [f32; 4],
    /// The live band radii this set was laid for (`live_bands_m` at the height
    /// it was requested from); zero for a band it did not lay.
    live: [f32; 4],
    /// The rebuild margin this set was laid with (`regen_m`).
    regen: f32,
    /// Per level, the ring its records span round the anchor, metres: inner
    /// and outer (`lay_band`). What a landing's cross-fade checks the old
    /// partition against (`fade_covered`).
    rings: [[f32; 2]; 4],
    /// The voxel columns for the innermost part of the finest level: what makes
    /// a cave, an overhang and a block to remove expressible at all. Built on
    /// this same task, because the records carry their own slots.
    pub columns: ColumnTier,
}

impl FineSet {
    /// Per fine level, the ring its records span round the anchor, metres.
    pub fn rings(&self) -> [[f32; 2]; 4] {
        self.rings
    }

    /// This set as the partition its successor replaces (`detail-fade`).
    pub fn replaced(&self) -> Replaced {
        Replaced {
            anchor: self.anchor,
            complete: self.complete,
        }
    }

    /// The angular radius the finest level is complete to, which is where the
    /// walker's contact stops trusting it.
    pub fn finest_radius(&self) -> f32 {
        self.finest_radius
    }

    /// The finest level's records, which are the ones a column belongs to.
    pub fn finest_records(&self) -> &[GpuCell] {
        &self.levels[3]
    }

    /// The level this set draws a direction at: the finest band whose
    /// COMPLETE radius reaches it, or `None` where only the base draws. The
    /// same `complete` array the partition uploads, so what this answers is
    /// what the GPU is drawing, not what the nominal bands promise.
    pub fn level_at(&self, direction: Vec3) -> Option<u8> {
        let metres = direction
            .normalize_or(Vec3::Y)
            .dot(self.anchor)
            .clamp(-1.0, 1.0)
            .acos()
            * PLANET_RADIUS;
        FINE_LEVELS
            .iter()
            .zip(&self.complete)
            .rev()
            .find(|(_, radius)| **radius > 0.0 && metres <= **radius)
            .map(|(level, _)| *level)
    }

    /// Whether this set no longer serves a player `moved` metres from its
    /// anchor whose live bands are `live`: a band now live that the set did
    /// not lay, or a live band about to outrun what was laid. On the ground
    /// that is the walk rule, `moved > REGEN_DISTANCE_M`; higher up the set
    /// was laid with a wider margin (`regen_m`), so fast flight rebuilds far
    /// less often. A set with fine bands, seen from high
    /// enough that none is live, is replaced once by an empty one, with a
    /// margin so hovering at the edge does not flip it back and forth.
    pub fn outrun(&self, live: &[f32; 4], moved: f32, clear_of_all: bool) -> bool {
        let outran = live
            .iter()
            .zip(&self.live)
            .filter(|(now, _)| **now > 0.0)
            .any(|(now, laid)| *laid <= 0.0 || moved > laid - now + self.regen);
        outran || clear_of_all && self.live.iter().any(|m| *m > 0.0)
    }

    /// The live radii this set was laid for.
    pub fn live(&self) -> [f32; 4] {
        self.live
    }

    /// Great-circle metres from this set's anchor to a direction.
    pub fn metres_from_anchor(&self, direction: Vec3) -> f32 {
        direction
            .normalize_or(Vec3::Y)
            .dot(self.anchor)
            .clamp(-1.0, 1.0)
            .acos()
            * PLANET_RADIUS
    }

    /// Make one cell's record agree with the column under it, after an edit.
    ///
    /// The GPU rebuilds the geometry from what it is sent, so what is sent has
    /// to be the whole truth: the runs say what is underground and the RECORD
    /// says where the surface is. Repacking one and leaving the other is a
    /// world where the hole is real and the lid over it is too.
    pub fn reconcile(&mut self, record: usize) {
        let Some(&slot) = self.columns.slots.get(record) else {
            return;
        };
        let Some(column) = self.columns.columns.get(slot).cloned() else {
            return;
        };
        column::reconcile_surface(&mut self.levels[3], &self.finest_neighbors, record, &column);
    }

    /// The column a finest record would have if the tier adopted it now:
    /// solid, as the rim is, with the save's edits for its cell. What an edit
    /// reads the material it takes from, before anything is committed.
    pub fn adoptable(&self, record: usize, edits: &Edits) -> Option<pbd_core::column::Column> {
        let cell = self.levels[3].get(record)?;
        Some(pbd_core::column::generate_edited_solid(
            &super::terrain::TERRAIN,
            Vec3::from_slice(&cell.direction_height[..3]),
            edits.for_cell(cell.metadata[3]),
        ))
    }

    /// Adopt a column for a finest record outside the tier; see
    /// `ColumnTier::adopt`.
    pub fn adopt(&mut self, record: usize, column: pbd_core::column::Column) -> bool {
        self.columns
            .adopt(&mut self.levels[3], &self.finest_neighbors, record, column)
    }

    /// Take the column tier out, leaving an empty one. For tests that want the
    /// tier and the records it was stamped into side by side.
    #[cfg(test)]
    pub(crate) fn take_columns(&mut self) -> ColumnTier {
        std::mem::replace(&mut self.columns, ColumnTier::empty())
    }
}

impl LodParams {
    /// The partition this set can actually serve. The anchor is the set's own,
    /// never the live camera: the records exist around the anchor, so hiding a
    /// coarse tile anywhere else is hiding it where nothing replaces it. A set
    /// that is one regeneration behind the player therefore draws a slightly
    /// stale level of detail, which nobody can see, rather than a hole, which
    /// everybody can.
    pub fn of(set: &FineSet) -> Self {
        Self {
            player: set.anchor,
            // A band the set did not lay is 2.0, which no dot product exceeds:
            // cos(0) would still admit a tile exactly at the anchor.
            bands: Vec4::from_array(set.complete.map(|m| {
                if m > 0.0 {
                    (m / PLANET_RADIUS).cos()
                } else {
                    2.0
                }
            })),
        }
    }

    /// The partition before any fine set is resident: nothing is hidden and no
    /// fine tile draws, so the base covers the globe on its own.
    pub fn base_only() -> Self {
        Self {
            player: Vec3::Y,
            bands: Vec4::splat(2.0),
        }
    }
}

/// A stable identity for a fine cell across regenerations, so its tree and
/// its texture variation do not reshuffle when the set is rebuilt.
fn stable_id(cell: &LocalCell) -> u32 {
    let p = cell.point;
    let mut h = (p.face as u32).wrapping_mul(0x9e37_79b9)
        ^ p.i.wrapping_mul(0x85eb_ca6b)
        ^ p.j.wrapping_mul(0xc2b2_ae35)
        ^ (p.level as u32) << 27;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^ (h >> 12)
}

/// One fine level laid around an anchor: its cells, truncated to capacity
/// nearest first, and the radius it is complete to.
pub(crate) struct Band {
    pub(crate) cells: Vec<LocalCell>,
    /// Metres from the anchor this level is resident AND complete to.
    complete_m: f32,
    /// The angular radius the level is complete to, band plus walk, or less
    /// where capacity truncated it. Only the finest level's is kept.
    radius: f32,
    /// The ring the records span round the anchor, metres: inner and outer.
    ring: [f32; 2],
}

/// Lay fine level `FINE_LEVELS[k]` as a band from just inside the next finer
/// band's radius to just outside its own, plus the regeneration distance, so
/// the live bands stay resident as the player walks. The radii are the live
/// ones (`live_bands_m`); a band that is not live is not laid at all. Over
/// capacity, the farthest cells are dropped, never the nearest.
///
/// `cover` is a ring, metres round `anchor`, the records must also span: the
/// part of this level the partition being replaced draws (`cover_ring`), so a
/// landing can cross-fade from it (`detail-fade`).
pub(crate) fn lay_band(
    k: usize,
    anchor: Vec3,
    live: &[f32; 4],
    regen: f32,
    cover: Option<[f32; 2]>,
) -> Band {
    if live[k] <= 0.0 {
        return Band {
            cells: Vec::new(),
            complete_m: 0.0,
            radius: 0.0,
            ring: [0.0; 2],
        };
    }
    let level = FINE_LEVELS[k];
    let mut lattice = Lattice::default();
    let margin = regen + 3.0 * tile_width_m(level);
    let mut inner = if k + 1 < FINE_LEVELS.len() {
        (live[k + 1] - margin).max(0.0)
    } else {
        0.0
    };
    let mut outer = live[k] + margin;
    if let Some([cover_inner, cover_outer]) = cover {
        inner = inner.min(cover_inner);
        outer = outer.max(cover_outer);
    }
    let mut cells =
        lattice.cells_in_band(level, anchor, inner / PLANET_RADIUS, outer / PLANET_RADIUS);
    let mut complete_m = live[k];
    let mut radius = (live[k] + regen) / PLANET_RADIUS;
    let mut ring = [inner, outer];
    if cells.len() > FINE_CAPACITY as usize {
        warn!(
            "level {level} band holds {} cells over a capacity of {FINE_CAPACITY}; dropping the farthest",
            cells.len()
        );
        let mut order: Vec<usize> = (0..cells.len()).collect();
        order.sort_by(|&a, &b| {
            cells[b]
                .cell
                .direction
                .dot(anchor)
                .total_cmp(&cells[a].cell.direction.dot(anchor))
        });
        let mut remap = vec![usize::MAX; cells.len()];
        for (new, &old) in order.iter().take(FINE_CAPACITY as usize).enumerate() {
            remap[old] = new;
        }
        let mut kept: Vec<LocalCell> = Vec::with_capacity(FINE_CAPACITY as usize);
        let mut taken: Vec<Option<LocalCell>> = cells.into_iter().map(Some).collect();
        for &old in order.iter().take(FINE_CAPACITY as usize) {
            let mut cell = taken[old].take().expect("each cell is taken once");
            for n in &mut cell.cell.neighbors {
                *n = if *n == usize::MAX {
                    usize::MAX
                } else {
                    remap[*n]
                };
            }
            kept.push(cell);
        }
        cells = kept;
        // The band was cut short, so the level is complete only inside the
        // farthest cell it kept, less its own ring. Report that rather than
        // the nominal band, or the level above would hide tiles out to a
        // radius this one does not reach.
        let farthest = cells.last().map_or(0.0, |c| {
            c.cell.direction.dot(anchor).clamp(-1.0, 1.0).acos()
        });
        let reach = (farthest * PLANET_RADIUS - 2.0 * tile_width_m(level)).max(0.0);
        complete_m = complete_m.min(reach);
        // The records reach this far whatever the band: a ring widened to
        // cover a replaced partition may lose only its widening.
        ring[1] = ring[1].min(reach);
        radius = radius
            .min(farthest - 2.0 * tile_width_m(level) / PLANET_RADIUS)
            .max(0.0);
    }
    Band {
        cells,
        complete_m,
        radius,
        ring,
    }
}

/// The floor rule for fine level `k`, once every finer level's complete
/// radius is settled: a side reads a floor where its neighbour is inside the
/// next finer level's complete radius, and two of this level's tiles past it
/// cover the one approximation, which is that the shader finds the neighbour
/// by reflecting the centre through the edge midpoint rather than knowing it.
fn floor_rule(
    k: usize,
    anchor: Vec3,
    complete: &[f32; 4],
    replacing: Option<&Replaced>,
) -> FloorRule {
    if k + 1 >= FINE_LEVELS.len() {
        return FloorRule::None;
    }
    let guard = 2.0 * tile_width_m(FINE_LEVELS[k]);
    FloorRule::Within {
        anchor,
        cos: ((complete[k + 1] + guard) / PLANET_RADIUS).cos(),
        also: replacing
            .filter(|old| old.complete[k + 1] > 0.0)
            .map(|old| {
                (
                    old.anchor,
                    ((old.complete[k + 1] + guard) / PLANET_RADIUS).cos(),
                )
            }),
    }
}

/// The partition a new set replaces, which its landing cross-fades from
/// (`detail-fade`): the resident set's anchor and complete radii, metres.
#[derive(Clone, Copy, Debug)]
pub struct Replaced {
    pub anchor: Vec3,
    pub complete: [f32; 4],
}

/// The ring, metres round `anchor`, that level `k`'s records must span for
/// a landing to cross-fade from `old`: where `old` draws level `k` (between
/// its complete radii of `k + 1` and `k`), moved by the distance between the
/// two anchors. A metre over `fade_covered`'s rounding slack, so a set laid
/// to it passes that check by construction. `None` where `old` does not draw
/// level `k`.
fn cover_ring(k: usize, anchor: Vec3, old: &Replaced) -> Option<[f32; 2]> {
    const OVER_SLACK_M: f32 = 1.0;
    let outer = old.complete[k];
    if outer <= 0.0 {
        return None;
    }
    let d = old.anchor.dot(anchor).clamp(-1.0, 1.0).acos() * PLANET_RADIUS;
    let inner = if k + 1 < FINE_LEVELS.len() {
        old.complete[k + 1]
    } else {
        0.0
    };
    Some([
        (inner - d - OVER_SLACK_M).max(0.0),
        outer + d + OVER_SLACK_M,
    ])
}

/// The records' ring per fine level as the visibility pass tests it during a
/// fade (`detail-fade` design section 4): cosines round the new anchor, inner
/// and outer, each shrunk by one of that level's tiles, so a cell centre that
/// passes is a cell the records hold. A level with no ring holds nothing
/// (an outer cosine of 2, which no dot product reaches).
pub fn records_cosines(rings: &[[f32; 2]; 4]) -> (Vec4, Vec4) {
    let mut inner = [1.0f32; 4];
    let mut outer = [2.0f32; 4];
    for (k, &level) in FINE_LEVELS.iter().enumerate() {
        let tile = tile_width_m(level);
        let [from, to] = rings[k];
        let from = if from > 0.0 { from + tile } else { 0.0 };
        let to = to - tile;
        if to > from {
            inner[k] = (from / PLANET_RADIUS).cos();
            outer[k] = (to / PLANET_RADIUS).cos();
        }
    }
    (Vec4::from_array(inner), Vec4::from_array(outer))
}

/// Whether the set `new` holds every cell the partition `old` (the one the
/// render world draws now) draws. Where it does not, the landing still
/// cross-fades, and the visibility pass draws the new partition whole in the
/// ring the records lack (`records_cosines`), so that ring switches at once
/// and says so in the log. Level `k` of `old` is
/// drawn between the complete radii of `k + 1` and `k` round its anchor; the
/// new records of level `k` are a ring round the new anchor (`FineSet::rings`).
/// With the anchors `d` apart, the old ring must lie inside the new. Returns
/// the first level that does not, and by how far.
const FADE_SLACK_M: f32 = 0.5;

pub fn fade_covered(old: &LodParams, new: &FineSet) -> Result<(), String> {
    let radius = |cos: f32| {
        if cos <= 1.0 {
            cos.clamp(-1.0, 1.0).acos() * PLANET_RADIUS
        } else {
            0.0
        }
    };
    let d = old.player.dot(new.anchor).clamp(-1.0, 1.0).acos() * PLANET_RADIUS;
    for (k, &level) in FINE_LEVELS.iter().enumerate() {
        let old_outer = radius(old.bands[k]);
        if old_outer <= 0.0 {
            continue;
        }
        let old_inner = if k + 1 < FINE_LEVELS.len() {
            radius(old.bands[k + 1])
        } else {
            0.0
        };
        let [new_inner, new_outer] = new.rings[k];
        let need_outer = old_outer + d;
        let need_inner = (old_inner - d).max(0.0);
        // Half a metre of rounding: a band edge and the ring laid from it are
        // the same number computed twice.
        if need_outer > new_outer + FADE_SLACK_M {
            return Err(format!(
                "level {level}: the old band reaches {need_outer:.0} m from the new anchor, the new records {new_outer:.0} m"
            ));
        }
        if need_inner + FADE_SLACK_M < new_inner {
            return Err(format!(
                "level {level}: the old band starts {need_inner:.0} m from the new anchor, the new records {new_inner:.0} m"
            ));
        }
    }
    Ok(())
}

/// Threads a rebuild may use: every core. The rebuild is what the player is
/// waiting on, and a core it leaves idle is a longer wait on ground that
/// cannot be dug.
fn build_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// Threads a rebuild may use while the player is PLAYING, not waiting: a
/// quarter of the cores. On every core the rebuild starved the main thread:
/// measured on the far-side tour (8 cores, 1440x900), frames over 16.7 ms fell
/// from 195 to 3 and p99.9 from 47.5 ms to 5.6 ms at two threads
/// (`far-side-flight`). A forced rebuild, which the player is waiting on,
/// still takes every core.
fn background_threads() -> usize {
    (build_threads() / 4).max(1)
}

/// Generate every fine level for an anchor on the ground, on every core.
pub fn generate_fine(anchor: Vec3, columns: &ColumnSettings, edits: &Edits) -> FineSet {
    generate_fine_live(
        anchor,
        BAND_M,
        REGEN_DISTANCE_M,
        columns,
        edits,
        build_threads(),
        None,
    )
}

/// Generate the fine levels live at `live` (`live_bands_m` for the player's
/// height) around an anchor, on `threads` threads. A band that is not live is
/// not built.
pub fn generate_fine_live(
    anchor: Vec3,
    live: [f32; 4],
    regen: f32,
    columns: &ColumnSettings,
    edits: &Edits,
    threads: usize,
    replacing: Option<Replaced>,
) -> FineSet {
    generate_fine_live_on(anchor, live, regen, columns, edits, threads, replacing)
}

/// Generate every fine level for an anchor on the ground on `threads` threads.
#[cfg(test)]
pub(crate) fn generate_fine_on(
    anchor: Vec3,
    columns: &ColumnSettings,
    edits: &Edits,
    threads: usize,
) -> FineSet {
    generate_fine_live_on(
        anchor,
        BAND_M,
        REGEN_DISTANCE_M,
        columns,
        edits,
        threads,
        None,
    )
}

/// Generate the live fine levels on `threads` threads. The output does not
/// depend on the count: a band is laid by its own lattice, and a record is a
/// pure function of its cell whichever memo it was built with, so the parallel
/// set is the serial one byte for byte, and a test holds it to that.
pub(crate) fn generate_fine_live_on(
    anchor: Vec3,
    live: [f32; 4],
    regen: f32,
    columns: &ColumnSettings,
    edits: &Edits,
    threads: usize,
    replacing: Option<Replaced>,
) -> FineSet {
    let anchor = anchor.normalize_or(Vec3::Y);
    let threads = threads.max(1);
    // The bands first, all four, because a coarse level's floors depend on
    // how far the next finer level is complete to, and truncation decides
    // that.
    // Each ring also spans what the replaced partition draws, so the landing
    // can cross-fade from it (`detail-fade` design section 3).
    let cover = |k: usize| {
        replacing
            .as_ref()
            .and_then(|old| cover_ring(k, anchor, old))
    };
    let bands: Vec<Band> = if threads > 1 {
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..FINE_LEVELS.len())
                .map(|k| {
                    let cover = cover(k);
                    scope.spawn(move || lay_band(k, anchor, &live, regen, cover))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("a band laid"))
                .collect()
        })
    } else {
        (0..FINE_LEVELS.len())
            .map(|k| lay_band(k, anchor, &live, regen, cover(k)))
            .collect()
    };
    let complete: [f32; 4] = std::array::from_fn(|k| bands[k].complete_m);
    let rings: [[f32; 2]; 4] = std::array::from_fn(|k| bands[k].ring);
    let finest_radius = bands[3].radius;
    // Then the records, cut into chunks the threads take in turn. Each chunk
    // has its own height memo, which costs some sharing across chunk edges
    // and buys no locks.
    let total: usize = bands.iter().map(|band| band.cells.len()).sum();
    let chunk = (total / (threads * 4)).max(256);
    let jobs: Vec<(usize, Range<usize>)> = bands
        .iter()
        .enumerate()
        .flat_map(|(k, band)| {
            let len = band.cells.len();
            (0..len)
                .step_by(chunk)
                .map(move |start| (k, start..(start + chunk).min(len)))
        })
        .collect();
    let build = |(k, range): &(usize, Range<usize>)| -> Vec<GpuCell> {
        let level = FINE_LEVELS[*k];
        let rule = floor_rule(*k, anchor, &complete, replacing.as_ref());
        let mut heights = Heights::default();
        bands[*k].cells[range.clone()]
            .iter()
            .map(|local| {
                record(
                    CellSource {
                        direction: local.cell.direction,
                        corners: &local.cell.corners,
                        neighbor_directions: &local.neighbor_directions,
                        level,
                        owners: local.owners,
                        id: stable_id(local),
                    },
                    &mut heights,
                    rule,
                )
            })
            .collect()
    };
    let built: Vec<Vec<GpuCell>> = if threads > 1 {
        let next = AtomicUsize::new(0);
        let done: Vec<Mutex<Vec<GpuCell>>> = jobs.iter().map(|_| Mutex::default()).collect();
        std::thread::scope(|scope| {
            for _ in 0..threads.min(jobs.len()) {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(work) = jobs.get(index) else {
                            break;
                        };
                        *done[index].lock().expect("one writer per job") = build(work);
                    }
                });
            }
        });
        done.into_iter()
            .map(|slot| slot.into_inner().expect("one writer per job"))
            .collect()
    } else {
        jobs.iter().map(build).collect()
    };
    let mut levels: [Vec<GpuCell>; 4] = Default::default();
    for ((k, _), records) in jobs.iter().zip(built) {
        levels[*k].extend(records);
    }
    let finest_neighbors: Vec<[u32; 6]> = bands[3]
        .cells
        .iter()
        .map(|local| {
            let mut ids = [u32::MAX; 6];
            for (id, &n) in ids.iter_mut().zip(&local.cell.neighbors) {
                *id = if n == usize::MAX { u32::MAX } else { n as u32 };
            }
            ids
        })
        .collect();
    // The column tier, last, because it stamps each finest record with its own
    // slot: the tier and the records it is read through are one artifact and
    // are built on one task.
    let columns = column::build(anchor, &mut levels[3], &finest_neighbors, columns, edits);
    FineSet {
        anchor,
        levels,
        finest_neighbors,
        finest_radius,
        complete,
        live,
        regen,
        rings,
        columns,
    }
}

/// The current fine set, shared with the render world by `Arc`; the version
/// says when the fine regions of the storage buffer need rewriting.
#[derive(Resource, Clone, ExtractResource)]
pub struct PlanetFine {
    pub set: Arc<FineSet>,
    pub version: u64,
}

/// What the streaming is doing under the player, for the HUD and for the
/// error an edit logs when it finds nothing to edit. A player who has outrun
/// the fine set reads it off the screen instead of inferring it from a wall
/// that vanishes.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct NearField {
    /// The level drawing the ground under the camera; `None` is the base.
    pub level: Option<u8>,
    /// Whether the cell under the camera has a column, which is the only
    /// state in which it can be dug or built on.
    pub column: bool,
    /// Great-circle metres from the camera to the resident set's anchor.
    pub from_anchor_m: f32,
    /// Seconds the in-flight rebuild has been running, if one is.
    pub rebuild_s: Option<f32>,
    /// Columns resident in the tier.
    pub columns: usize,
}

impl NearField {
    /// One line for the HUD.
    pub fn line(&self) -> String {
        let level = self
            .level
            .map_or("L7 base".to_string(), |level| format!("L{level}"));
        let column = if self.column { "column" } else { "NO COLUMN" };
        let rebuild = self
            .rebuild_s
            .map_or(String::new(), |s| format!(" | rebuilding {s:.1} s"));
        format!(
            "underfoot {level}, {column} | {:.0} m from anchor | {} columns{rebuild}",
            self.from_anchor_m, self.columns
        )
    }
}

#[derive(Resource, Default)]
pub struct LodRefresh {
    /// The set and its contact tier, both built on the pool.
    task: Option<Task<(Arc<FineSet>, super::contact::PreparedFine)>>,
    /// When the in-flight task was spawned.
    started: Option<std::time::Instant>,
    /// Rebuild the tier whatever the player has or has not walked.
    ///
    /// The distance rule answers "has the player left the tier", which is the
    /// only reason to rebuild while ONE world is open. Loading another changes
    /// the edits under a tier that is still standing where it was, and a load
    /// that lands a few metres from where the last one ended would otherwise
    /// keep the previous world's holes until the player walked far enough to
    /// notice.
    force: bool,
}

/// Where the bands are measured from: the active camera, which is at the
/// player whether walking or flying, body-local (from the planet's centre).
fn player_position(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    center: Vec3,
) -> Option<Vec3> {
    cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform.translation() - center)
}

impl LodRefresh {
    /// Ask for a rebuild on the next frame, whatever the player has walked.
    pub fn force(&mut self) {
        self.force = true;
    }

    /// How long the in-flight rebuild has been running, if one is.
    pub fn in_flight_s(&self) -> Option<f32> {
        self.task
            .as_ref()
            .and(self.started)
            .map(|started| started.elapsed().as_secs_f32())
    }
}

/// Rebuild the fine set on the compute pool once the player has walked
/// `REGEN_DISTANCE_M` from its anchor, and swap it in, with the walker's
/// contact, when it lands. One rebuild in flight at a time.
// Eight parameters: the seven the rebuild already needed, and the save, whose
// edits a set built without would quietly undig. A struct of them would be a
// struct with one caller.
#[allow(clippy::too_many_arguments)]
pub fn refresh_lod(
    mut commands: Commands,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Res<super::PlanetRenderFrame>,
    fine: Res<PlanetFine>,
    settings: Res<ColumnSettings>,
    mut refresh: ResMut<LodRefresh>,
    mut contact: ResMut<super::PlanetContact>,
    edits: Res<crate::saves::WorldSave>,
    mut near: ResMut<NearField>,
    air: Option<Res<crate::atmosphere::Air>>,
) {
    let player = player_position(&cameras, frame.center.as_vec3());
    let direction = player.and_then(|p| p.try_normalize());
    if let Some(direction) = direction {
        let column = contact
            .finest_cell(direction)
            .is_some_and(|record| fine.set.columns.column(record).is_some());
        let readout = NearField {
            level: fine.set.level_at(direction),
            column,
            from_anchor_m: fine.set.metres_from_anchor(direction),
            rebuild_s: refresh.in_flight_s(),
            columns: fine.set.columns.columns.len(),
        };
        if *near != readout {
            *near = readout;
        }
    }
    if let Some(task) = refresh.task.as_mut() {
        if let Some((set, prepared)) = block_on(poll_once(task)) {
            let took = refresh.in_flight_s().unwrap_or(0.0);
            refresh.task = None;
            refresh.started = None;
            let behind = direction.map_or(0.0, |d| set.metres_from_anchor(d));
            info!(
                "fine set {} landed after {took:.1} s: {} columns, the player {behind:.0} m from its anchor",
                fine.version + 1,
                set.columns.columns.len()
            );
            let timer = std::time::Instant::now();
            contact.set_prepared(prepared);
            commands.insert_resource(PlanetFine {
                set,
                version: fine.version + 1,
            });
            spent("landing: contact tier", timer);
        }
        return;
    }
    let (Some(player), Some(direction)) = (player, direction) else {
        return;
    };
    // The bands as the player can use them from where they are: from high up
    // the finest are not live at all, and nothing is built for them.
    let ground = super::terrain::terrain_radius(direction);
    let live = live_bands_m(player.length(), ground);
    let regen = regen_m(player.length() - ground);
    // Past the coarsest band by a margin, so hovering at the edge does not
    // swap an empty set in and out.
    let clear_of_all = player.length() - ground > BAND_M[0] * 1.05;
    let moved = fine.set.metres_from_anchor(direction);
    // A rebuild the player is not waiting on waits for the atmosphere's step
    // to finish; see `advance_air`.
    let air_busy = air.is_some_and(|air| air.stepping());
    if refresh.force || (!air_busy && fine.set.outrun(&live, moved, clear_of_all)) {
        let threads = if refresh.force {
            build_threads()
        } else {
            background_threads()
        };
        refresh.force = false;
        let settings = settings.clone();
        // The edits travel WITH the task: the tier is rebuilt off the pool and
        // a set built without them would quietly undig every hole the moment
        // the player walked far enough.
        let timer = std::time::Instant::now();
        let made = edits.edits.clone();
        spent("request: cloning the edits", timer);
        let replacing = fine.set.replaced();
        info!(
            "fine set requested: {:.0} m above the ground, live bands {:.0?} m, {moved:.0} m from the anchor, {threads} threads",
            player.length() - ground,
            live
        );
        refresh.started = Some(std::time::Instant::now());
        refresh.task = Some(AsyncComputeTaskPool::get().spawn(async move {
            let set = Arc::new(generate_fine_live(
                direction,
                live,
                regen,
                &settings,
                &made,
                threads,
                Some(replacing),
            ));
            let prepared = super::PlanetContact::prepare_fine(&set);
            (set, prepared)
        }));
    }
}

/// MEASUREMENT (`far-side-flight`, the walk's spikes): log a main-thread step
/// that took over 2 ms, with what it was.
pub(crate) fn spent(what: &str, since: std::time::Instant) {
    let ms = since.elapsed().as_secs_f64() * 1000.0;
    if ms > 2.0 {
        info!("SPENT {ms:.1} ms {what}");
    }
}

/// Per-view LOD inputs the compute and surface shaders read.
#[derive(Clone, Copy, Debug)]
pub struct LodParams {
    pub player: Vec3,
    pub bands: Vec4,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On the ground the live bands are the nominal ones; from height they
    /// shrink by the slant distance, level 11 is gone from 300 m up and every
    /// fine band from 2,400 m (`far-side-flight`).
    #[test]
    fn bands_are_measured_from_the_players_height() {
        let ground = PLANET_RADIUS + 12.0;
        let on_ground = live_bands_m(ground, ground);
        for k in 0..4 {
            assert!((on_ground[k] - BAND_M[k]).abs() < 0.01, "{on_ground:?}");
        }
        let at = |h: f32| live_bands_m(ground + h, ground);
        let low = at(250.0);
        assert!(
            low[3] > 0.0 && low[3] < 170.0,
            "level 11 at 250 m: {}",
            low[3]
        );
        assert_eq!(at(300.0)[3], 0.0, "no level 11 from 300 m up");
        // From a kilometre up the coarsest band reaches 1,999 m on this curved
        // 4.8 km world (the triangle through the centre), not a flat 2,182 m.
        let km = at(1_000.0);
        assert!((km[0] - 1_999.0).abs() < 2.0 && km[2] == 0.0, "{km:?}");
        assert_eq!(at(2_400.0), [0.0; 4], "no fine band from 2,400 m up");
        let mut previous = on_ground;
        for h in [10.0, 100.0, 400.0, 900.0, 1_500.0, 2_300.0] {
            let now = at(h);
            for k in 0..4 {
                assert!(
                    now[k] <= previous[k],
                    "bands only shrink as the player climbs"
                );
            }
            previous = now;
        }
    }

    /// The rebuild rule: the walk's 40 m on the ground; a band that turns live
    /// and was not laid forces a rebuild; a set seen from above every band is
    /// swapped for an empty one.
    #[test]
    fn a_set_is_outrun_by_distance_or_by_a_band_turning_live() {
        let anchor = Vec3::Y;
        let set = super::near_field_tests::set_with_bands(anchor, BAND_M);
        assert!(!set.outrun(&BAND_M, REGEN_DISTANCE_M - 1.0, false));
        assert!(set.outrun(&BAND_M, REGEN_DISTANCE_M + 1.0, false));
        let mut aloft = super::near_field_tests::set_with_bands(anchor, BAND_M);
        aloft.live = [2_000.0, 600.0, 0.0, 0.0];
        assert!(
            aloft.outrun(&[2_000.0, 600.0, 500.0, 0.0], 0.0, false),
            "level 10 turned live"
        );
        assert!(!aloft.outrun(&[0.0; 4], 0.0, false));
        assert!(
            aloft.outrun(&[0.0; 4], 0.0, true),
            "high above: replaced by an empty set"
        );
        let mut empty = super::near_field_tests::set_with_bands(anchor, [0.0; 4]);
        empty.live = [0.0; 4];
        assert!(
            !empty.outrun(&[0.0; 4], 10_000.0, true),
            "an empty set is never rebuilt aloft"
        );
    }

    /// A set requested from above the finest band lays no finest level and no
    /// columns, and whatever it lays still serves as a set.
    #[test]
    fn a_set_built_from_altitude_has_no_finest_level() {
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let ground = super::super::terrain::terrain_radius(anchor);
        let live = live_bands_m(ground + 800.0, ground);
        assert_eq!(live[3], 0.0);
        let set = generate_fine_live_on(
            anchor,
            live,
            regen_m(800.0),
            &ColumnSettings::default(),
            &Edits::default(),
            2,
            None,
        );
        assert!(set.levels[3].is_empty() && set.levels[2].is_empty());
        assert!(!set.levels[0].is_empty(), "level 8 is live from 800 m");
        assert!(set.columns.columns.is_empty());
        assert_eq!(set.level_at(anchor), Some(FINE_LEVELS[1]));
    }

    #[test]
    fn bands_halve_with_the_tile_and_stay_inside_capacity() {
        for k in 1..FINE_LEVELS.len() {
            assert_eq!(BAND_M[k - 1], BAND_M[k] * 2.0);
            assert!(
                (tile_width_m(FINE_LEVELS[k - 1]) / tile_width_m(FINE_LEVELS[k]) - 2.0).abs()
                    < 1e-5
            );
        }
        assert!(band_cos(11) > band_cos(8));
        let set = generate_fine(
            Vec3::new(0.8776, 0.4794, 0.0),
            &ColumnSettings::default(),
            &Edits::new(),
        );
        for (k, level) in set.levels.iter().enumerate() {
            assert!(!level.is_empty());
            assert!(
                level.len() <= FINE_CAPACITY as usize,
                "level {} holds {}",
                FINE_LEVELS[k],
                level.len()
            );
            for cell in level {
                assert_eq!(cell.metadata[0] >> 8, FINE_LEVELS[k] as u32);
                assert!([5, 6].contains(&(cell.metadata[0] & 0xff)));
                assert!(cell.direction_height[3].is_finite());
            }
        }
        assert_eq!(set.finest_neighbors.len(), set.levels[3].len());
        // The finest band is complete out past its radius plus the walk.
        let complete = set.finest_radius().cos();
        for (cell, neighbors) in set.levels[3].iter().zip(&set.finest_neighbors) {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            if direction.dot(set.anchor) > complete {
                assert!(
                    neighbors
                        .iter()
                        .take((cell.metadata[0] & 0xff) as usize)
                        .all(|&n| n != u32::MAX)
                );
            }
        }
    }

    /// What the partition HIDES, the set must REPLACE. The shader hides a
    /// coarse tile inside `complete[k]` of the anchor and draws level k there
    /// instead, so level k has to be resident out to that radius with no gap.
    /// This is the invariant the first cut of the partition broke, by hiding
    /// around the live camera while the records sat around the anchor: a
    /// camera a few tens of metres off the anchor opened a ring of holes, and
    /// a camera flying opened a wide one, because a regeneration takes about
    /// two seconds and the player keeps moving through it.
    #[test]
    fn every_level_is_resident_out_to_the_radius_it_hides_the_coarser_one_inside() {
        let anchor = Vec3::new(0.3, 0.8, -0.5).normalize();
        let set = generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        assert_eq!(
            LodParams::of(&set).player,
            set.anchor,
            "the anchor is the set's"
        );
        let tangent = anchor.cross(Vec3::X).normalize();
        let bitangent = anchor.cross(tangent);
        for (k, level) in set.levels.iter().enumerate() {
            let tile = tile_width_m(FINE_LEVELS[k]);
            // Just inside the radius the coarser level stops drawing at, which
            // is the last place this level has to answer for.
            let radius = (set.complete[k] * 0.995 / PLANET_RADIUS).max(0.0);
            for step in 0..24 {
                let theta = std::f32::consts::TAU * step as f32 / 24.0;
                let out = tangent * theta.cos() + bitangent * theta.sin();
                let direction = anchor * radius.cos() + out * radius.sin();
                let nearest = level
                    .iter()
                    .map(|c| Vec3::from_slice(&c.direction_height[..3]).distance(direction))
                    .fold(f32::MAX, f32::min)
                    * PLANET_RADIUS;
                assert!(
                    nearest < 1.5 * tile,
                    "level {} has no record within {:.1} m of its own edge at bearing {theta:.2}: \
                     nearest {nearest:.1} m, complete to {:.0} m",
                    FINE_LEVELS[k],
                    1.5 * tile,
                    set.complete[k]
                );
            }
        }
    }

    /// A vertex-centred fine cell has the same direction and height as the
    /// coarse cell it is centred on, which is what makes a coarse cap's
    /// height the fine neighbour's height across a band boundary.
    #[test]
    fn a_vertex_centred_fine_cell_shares_its_owners_height() {
        let set = generate_fine(
            Vec3::new(0.3, 0.8, -0.5),
            &ColumnSettings::default(),
            &Edits::new(),
        );
        let coarse: HashMap<[u32; 3], f32> = set.levels[2]
            .iter()
            .map(|c| {
                (
                    Vec3::from_slice(&c.direction_height[..3])
                        .to_array()
                        .map(f32::to_bits),
                    c.direction_height[3],
                )
            })
            .collect();
        let mut checked = 0;
        for cell in &set.levels[3] {
            let a = Vec3::from_slice(&cell.owner_a[..3]);
            let b = Vec3::from_slice(&cell.owner_b[..3]);
            if a == b
                && let Some(height) = coarse.get(&a.to_array().map(f32::to_bits))
            {
                assert_eq!(*height, cell.direction_height[3]);
                checked += 1;
            }
        }
        assert!(checked > 100, "{checked} shared cells checked");
    }
}

#[cfg(test)]
mod near_field_tests {
    use super::*;

    pub(super) fn set_with_bands(anchor: Vec3, complete: [f32; 4]) -> FineSet {
        FineSet {
            anchor,
            levels: Default::default(),
            finest_neighbors: Vec::new(),
            finest_radius: 0.0,
            complete,
            live: complete,
            regen: REGEN_DISTANCE_M,
            rings: std::array::from_fn(|k| [0.0, complete[k]]),
            columns: ColumnTier::empty(),
        }
    }

    fn metres_away(anchor: Vec3, metres: f32) -> Vec3 {
        let axis = anchor.any_orthonormal_vector();
        Quat::from_axis_angle(axis, metres / PLANET_RADIUS) * anchor
    }

    /// A landing cross-fades only when every cell the old partition draws is
    /// among the new records (`detail-fade`): a step inside the new rings'
    /// margin fades, one past it does not, and neither does a new set that
    /// dropped a level the old one drew.
    #[test]
    fn a_landing_fades_only_when_the_new_records_hold_the_old_partition() {
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let old = LodParams::of(&set_with_bands(anchor, [2400.0, 1200.0, 600.0, 300.0]));
        // New rings a 60 m margin wider than the bands, inner edges included.
        let mut new = set_with_bands(metres_away(anchor, 40.0), [2400.0, 1200.0, 600.0, 300.0]);
        new.rings = [
            [540.0 * 2.0 - 60.0, 2460.0],
            [540.0, 1260.0],
            [240.0, 660.0],
            [0.0, 360.0],
        ];
        assert!(fade_covered(&old, &new).is_ok());
        new.anchor = metres_away(anchor, 100.0);
        assert!(
            fade_covered(&old, &new).is_err(),
            "100 m is past a 60 m margin"
        );
        new.anchor = metres_away(anchor, 40.0);
        new.rings[3] = [0.0, 0.0];
        assert!(
            fade_covered(&old, &new).is_err(),
            "the new set lost the finest level"
        );
    }

    /// A set laid to replace another holds every cell the other draws, so its
    /// landing cross-fades (`detail-fade` design section 3): in flight, where
    /// the anchor has moved well past the rebuild margin by the time the set
    /// is requested, and where the bands have resized with height. Before
    /// this, a third to a half of a flight's landings missed by 1 to 14 m and
    /// popped. The moves stay within what the finest level's capacity can lay
    /// (about 380 m round the anchor); past it the landing pops and logs why.
    #[test]
    fn a_set_laid_to_replace_another_covers_it_for_the_fade() {
        let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let old_complete = [2400.0, 1200.0, 600.0, 300.0];
        let old_set = set_with_bands(anchor, old_complete);
        let old = LodParams::of(&old_set);
        let replacing = old_set.replaced();
        for (moved, live) in [
            (60.0, BAND_M),
            (40.0, [2000.0, 1000.0, 500.0, 250.0]),
            (60.0, [2600.0, 1300.0, 650.0, 320.0]),
        ] {
            let new_anchor = metres_away(anchor, moved);
            let rings = std::array::from_fn(|k| {
                lay_band(
                    k,
                    new_anchor,
                    &live,
                    REGEN_DISTANCE_M,
                    cover_ring(k, new_anchor, &replacing),
                )
                .ring
            });
            let mut new = set_with_bands(new_anchor, live);
            new.rings = rings;
            assert!(
                fade_covered(&old, &new).is_ok(),
                "moved {moved} m with bands {live:?}: {:?}",
                fade_covered(&old, &new)
            );
            // Without the cover, the same landing is refused: this is the
            // pop the owner saw.
            let bare: [[f32; 2]; 4] = std::array::from_fn(|k| {
                lay_band(k, new_anchor, &live, REGEN_DISTANCE_M, None).ring
            });
            new.rings = bare;
            assert!(
                fade_covered(&old, &new).is_err(),
                "moved {moved} m with bands {live:?} fits the bare margin"
            );
        }
    }

    /// The records' rings as the visibility pass tests them: a tile inside at
    /// both edges, the finest level from the anchor out, and a level with no
    /// ring holding nothing (`detail-fade` design section 4).
    #[test]
    fn the_records_rings_are_tested_a_tile_inside() {
        let cos = |m: f32| (m / PLANET_RADIUS).cos();
        let (inner, outer) =
            records_cosines(&[[1100.0, 2500.0], [500.0, 1300.0], [0.0, 0.0], [0.0, 350.0]]);
        let t = |k: usize| tile_width_m(FINE_LEVELS[k]);
        assert!((inner.x - cos(1100.0 + t(0))).abs() < 1e-6);
        assert!((outer.x - cos(2500.0 - t(0))).abs() < 1e-6);
        assert!((inner.y - cos(500.0 + t(1))).abs() < 1e-6);
        assert_eq!(inner.w, 1.0, "the finest level starts at the anchor");
        assert!((outer.w - cos(350.0 - t(3))).abs() < 1e-6);
        assert_eq!(outer.z, 2.0, "a level not laid holds nothing");
    }

    #[test]
    fn the_level_underfoot_is_the_finest_complete_band_that_reaches_it() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = super::near_field_tests::set_with_bands(anchor, BAND_M);
        assert_eq!(set.level_at(anchor), Some(11));
        assert_eq!(set.level_at(metres_away(anchor, 100.0)), Some(11));
        assert_eq!(set.level_at(metres_away(anchor, 400.0)), Some(10));
        assert_eq!(set.level_at(metres_away(anchor, 1_000.0)), Some(9));
        assert_eq!(set.level_at(metres_away(anchor, 2_000.0)), Some(8));
        assert_eq!(set.level_at(metres_away(anchor, 3_000.0)), None);
        let away = set.metres_from_anchor(metres_away(anchor, 250.0));
        assert!((away - 250.0).abs() < 0.5, "measured {away} m for 250 m");
    }

    #[test]
    fn a_truncated_band_stops_answering_where_it_stops_being_complete() {
        let anchor = Vec3::Y;
        let set = set_with_bands(anchor, [2_400.0, 1_200.0, 600.0, 120.0]);
        assert_eq!(set.level_at(metres_away(anchor, 200.0)), Some(10));
    }

    #[test]
    fn the_readout_line_names_a_missing_column_loudly() {
        let near = NearField {
            level: Some(11),
            column: false,
            from_anchor_m: 97.0,
            rebuild_s: Some(2.5),
            columns: 3_105,
        };
        let line = near.line();
        assert!(line.contains("L11") && line.contains("NO COLUMN"), "{line}");
        assert!(
            line.contains("97 m") && line.contains("rebuilding 2.5 s"),
            "{line}"
        );
        assert_eq!(
            NearField::default().line(),
            "underfoot L7 base, NO COLUMN | 0 m from anchor | 0 columns"
        );
    }
}

#[cfg(test)]
mod streaming_cost {
    //! A measurement instrument for the near-field-streaming and
    //! fine-set-in-a-second changes: what a fine-set rebuild costs, split into
    //! the bands laid, the records and the tier, on one thread and on every
    //! core, and what one column and the worm gather cost on their own.
    //! Ignored because it takes seconds; run it with
    //! `cargo test -p pbd-app --release --lib streaming_cost -- --ignored --nocapture`.
    use super::*;
    use crate::planet::terrain::TERRAIN;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn what_the_near_field_costs_to_build() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let settings = ColumnSettings::default();
        let edits = Edits::default();
        for (k, level) in FINE_LEVELS.iter().enumerate() {
            let started = Instant::now();
            let band = lay_band(k, anchor, &BAND_M, REGEN_DISTANCE_M, None);
            eprintln!(
                "level {level}: {} cells laid in {:.0} ms",
                band.cells.len(),
                started.elapsed().as_secs_f64() * 1000.
            );
        }
        let field = settings.worms();
        let started = Instant::now();
        let region = pbd_core::worms::gather(&field, &TERRAIN, anchor, settings.reach_m);
        let gathered = started.elapsed().as_secs_f64() * 1000.;
        let whole = |threads: usize| {
            let started = Instant::now();
            let set = generate_fine_on(anchor, &settings, &edits, threads);
            (set, started.elapsed().as_secs_f64() * 1000.)
        };
        let (serial, one) = whole(1);
        let threads = build_threads();
        let (_, all) = whole(threads);
        let mut finest = serial.levels[3].clone();
        let started = Instant::now();
        let tier = column::build(
            anchor,
            &mut finest,
            &serial.finest_neighbors,
            &settings,
            &edits,
        );
        let built = started.elapsed().as_secs_f64() * 1000.;
        eprintln!(
            "tier: {} columns, worm gather {gathered:.1} ms, build (gather + columns + reconcile + \
             relight) {built:.1} ms",
            tier.columns.len(),
        );
        let started = Instant::now();
        for cell in finest.iter().take(500) {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            std::hint::black_box(pbd_core::column::generate_edited(
                &region,
                &field,
                &TERRAIN,
                direction,
                edits.for_cell(cell.metadata[3]),
            ));
        }
        eprintln!(
            "one column, generated alone: {:.3} ms (over 500)",
            started.elapsed().as_secs_f64() * 1000. / 500.
        );
        eprintln!(
            "generate_fine whole ({} records, tier {}): one thread {one:.0} ms, {threads} threads \
             {all:.0} ms",
            serial.levels.iter().map(Vec::len).sum::<usize>(),
            serial.columns.columns.len()
        );
    }
}

#[cfg(test)]
mod fast_build_tests {
    use super::*;

    /// The parallel build is the serial build, record for record and byte
    /// for byte: a thread count is never allowed to change the world.
    #[test]
    fn the_parallel_build_is_the_serial_build() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let settings = ColumnSettings::default();
        let serial = generate_fine_on(anchor, &settings, &Edits::new(), 1);
        let parallel = generate_fine_on(anchor, &settings, &Edits::new(), 6);
        assert_eq!(serial.complete, parallel.complete);
        assert_eq!(serial.finest_radius, parallel.finest_radius);
        assert_eq!(serial.finest_neighbors, parallel.finest_neighbors);
        for ((one, many), level) in serial.levels.iter().zip(&parallel.levels).zip(FINE_LEVELS) {
            assert_eq!(one.len(), many.len());
            assert!(
                bytemuck::cast_slice::<GpuCell, u8>(one)
                    == bytemuck::cast_slice::<GpuCell, u8>(many),
                "level {level} differs between one thread and six"
            );
        }
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(serial.columns.gpu_records()),
            bytemuck::cast_slice::<_, u8>(parallel.columns.gpu_records())
        );
    }

    fn floor_of(cell: &GpuCell, side: usize) -> f32 {
        match side {
            0 => cell.owner_a[3],
            1 => cell.owner_b[3],
            _ => cell.floors[side - 2],
        }
    }

    /// Every side the SHADER reads a floor on carries the full floor.
    ///
    /// This is `planet_surface.wgsl`'s wall branch in Rust: a level coarser
    /// than the finest, the neighbour found by reflecting the centre through
    /// the edge's midpoint, and `covered_by_finer` against the set's own
    /// partition. Wherever that holds, the record's floor must be what
    /// `fine_floor` gives for the edge, computed afresh. It also reports how
    /// many sides that is, which is the size of the ring the build still pays
    /// seventeen samples for.
    #[test]
    fn a_floor_is_computed_wherever_the_shader_reads_one() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let params = LodParams::of(&set);
        let bands = params.bands.to_array();
        let mut read = 0usize;
        let mut sides = 0usize;
        for k in 0..FINE_LEVELS.len() - 1 {
            let laid = lay_band(k, anchor, &BAND_M, REGEN_DISTANCE_M, None);
            assert_eq!(laid.cells.len(), set.levels[k].len());
            for (local, cell) in laid.cells.iter().zip(&set.levels[k]) {
                let axis = Vec3::from_slice(&cell.direction_height[..3]);
                let degree = cell.degree();
                for side in 0..degree {
                    sides += 1;
                    let a = Vec3::from_slice(&cell.corners[side][..3]);
                    let b = Vec3::from_slice(&cell.corners[(side + 1) % degree][..3]);
                    let mid = (a + b).normalize();
                    let neighbor = (2.0 * mid - axis).normalize();
                    if neighbor.dot(params.player) <= bands[k + 1] {
                        continue;
                    }
                    read += 1;
                    let expected = fine_floor(
                        local.cell.direction,
                        local.neighbor_directions[side],
                        &mut Heights::default(),
                    );
                    assert_eq!(
                        floor_of(cell, side),
                        expected,
                        "level {} side {side}: the shader reads a floor the build skipped",
                        FINE_LEVELS[k]
                    );
                }
            }
        }
        assert!(read > 1_000, "only {read} floored sides were checked");
        println!(
            "{read} of {sides} coarse sides are read as floors ({:.1}%)",
            100.0 * read as f32 / sides as f32
        );
    }
}

#[cfg(test)]
mod seam_report {
    use super::*;
    use crate::planet::terrain::{PLANET_RADIUS, TERRAIN};

    /// How far a coarse cell's wall can stop ABOVE the finer caps it meets.
    ///
    /// A wall runs from this cell's cap down to the neighbour's, and where the
    /// neighbour's region is drawn by a finer band it goes to the fine FLOOR
    /// instead: one sample, the height at the shared edge's midpoint. The cells
    /// actually drawn along that edge are many, and any of them lower than that
    /// one sample is a cell whose cap the wall does not reach - which is a slit
    /// of sky between the wall's foot and the ground.
    ///
    /// This measures the shortfall off the real height field rather than a
    /// picture: for a coarse edge, the midpoint sample against the lowest of
    /// the samples along the edge itself.
    /// Not only a report: no edge may be short by a whole cell, which is the
    /// state this found before `fine_floor` took the lowest cap rather than
    /// the midpoint (0.45 m on average, 2.0 m at worst, one edge in eighty
    /// short by more than a cell). A slit a cell tall is sky through the
    /// ground, and it is the thing this measurement exists to keep shut.
    #[test]
    fn a_wall_reaches_within_a_cell_of_the_lowest_ground_it_meets() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let mut heights = Heights::default();
        let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());
        let mut worst: f32 = 0.0;
        let mut over_a_metre = 0usize;
        let mut edges = 0usize;
        let mut total: f64 = 0.0;
        // Edges of about a coarse cell's width, spread over the region the
        // player can see: a level-10 cell is twice a level-11 cell across.
        let span = 2.0 * 1.2087 / (1u32 << 10) as f32;
        for i in 0..4_000 {
            let t = golden * i as f32;
            let r = 0.02 * (i as f32 / 4_000.0).sqrt();
            let (a, b) = anchor.any_orthonormal_pair();
            let here = (anchor + a * (r * t.cos()) + b * (r * t.sin())).normalize();
            let there = (here + a * span).normalize();
            if heights.at(here) < TERRAIN.sea_level_m || heights.at(there) < TERRAIN.sea_level_m {
                continue;
            }
            edges += 1;
            // What the RECORD carries as this edge's floor, against the lowest
            // cap actually on it. Sampled finer than `fine_floor` does, so the
            // check cannot pass by measuring itself.
            let carried = fine_floor(here, there, &mut heights);
            let mut lowest = carried;
            for k in 1..64 {
                let f = k as f32 / 64.0;
                lowest = lowest.min(heights.at((here * (1.0 - f) + there * f).normalize()));
            }
            let short = carried - lowest;
            total += short as f64;
            worst = worst.max(short);
            over_a_metre += (short > 1.0) as usize;
        }
        assert!(
            over_a_metre == 0,
            "{over_a_metre} of {edges} edges leave a slit a whole cell tall"
        );
        println!(
            "\ncoarse walls, {edges} land edges of {:.1} m: the recorded floor stands above the \
             lowest cap it meets by {:.2} m on average, {:.1} m at worst; {} edges ({:.1}%) \
             are short by more than a metre, which is a slit a cell tall",
            span * PLANET_RADIUS,
            total / edges.max(1) as f64,
            worst,
            over_a_metre,
            100.0 * over_a_metre as f32 / edges.max(1) as f32
        );
    }
}
