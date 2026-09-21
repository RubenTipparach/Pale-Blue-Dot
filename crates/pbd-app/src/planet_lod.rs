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
use std::{collections::HashMap, sync::Arc};

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

/// Mean centre-to-centre tile width at a level, metres, on the shipped radius.
pub fn tile_width_m(level: u8) -> f32 {
    1.2087 * PLANET_RADIUS / (1u32 << level) as f32
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

fn record(source: CellSource, heights: &mut Heights) -> GpuCell {
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
        floors[side] = fine_floor(source.direction, neighbor, heights);
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
pub fn base_records(cells: &[DualCell]) -> Vec<GpuCell> {
    let mut heights = Heights::default();
    let mut records = Vec::with_capacity(cells.len());
    let mut neighbors = Vec::with_capacity(6);
    for (index, cell) in cells.iter().enumerate() {
        neighbors.clear();
        neighbors.extend(cell.neighbors.iter().map(|&n| cells[n].direction));
        records.push(record(
            CellSource {
                direction: cell.direction,
                corners: &cell.corners,
                neighbor_directions: &neighbors,
                level: BASE_LEVEL,
                owners: [cell.direction; 2],
                id: index as u32,
            },
            &mut heights,
        ));
    }
    records
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
    /// The voxel columns for the innermost part of the finest level: what makes
    /// a cave, an overhang and a block to remove expressible at all. Built on
    /// this same task, because the records carry their own slots.
    pub columns: ColumnTier,
}

impl FineSet {
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
            .find(|(_, radius)| metres <= **radius)
            .map(|(level, _)| *level)
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
            bands: Vec4::from_array(set.complete.map(|m| (m / PLANET_RADIUS).cos())),
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

/// Generate every fine level for an anchor. Each level is a band from just
/// inside the next finer band's radius to just outside its own, plus the
/// regeneration distance, so the live bands stay resident as the player
/// walks. Over capacity, the farthest cells are dropped, never the nearest.
pub fn generate_fine(anchor: Vec3, columns: &ColumnSettings, edits: &Edits) -> FineSet {
    let anchor = anchor.normalize_or(Vec3::Y);
    let mut lattice = Lattice::default();
    let mut heights = Heights::default();
    let mut levels: [Vec<GpuCell>; 4] = Default::default();
    let mut finest_neighbors = Vec::new();
    let mut finest_radius = (BAND_M[3] + REGEN_DISTANCE_M) / PLANET_RADIUS;
    let mut complete = BAND_M;
    for (k, &level) in FINE_LEVELS.iter().enumerate() {
        let margin = REGEN_DISTANCE_M + 3.0 * tile_width_m(level);
        let inner = if k + 1 < FINE_LEVELS.len() {
            (BAND_M[k + 1] - margin).max(0.0)
        } else {
            0.0
        };
        let outer = BAND_M[k] + margin;
        let mut cells =
            lattice.cells_in_band(level, anchor, inner / PLANET_RADIUS, outer / PLANET_RADIUS);
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
            }) * PLANET_RADIUS;
            complete[k] = complete[k].min((farthest - 2.0 * tile_width_m(level)).max(0.0));
            if level == FINEST_LEVEL {
                // Complete only inside the farthest kept cell less its ring.
                let farthest = cells.last().map_or(0.0, |c| {
                    c.cell.direction.dot(anchor).clamp(-1.0, 1.0).acos()
                });
                finest_radius = finest_radius
                    .min(farthest - 2.0 * tile_width_m(level) / PLANET_RADIUS)
                    .max(0.0);
            }
        }
        levels[k] = cells
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
                )
            })
            .collect();
        if level == FINEST_LEVEL {
            finest_neighbors = cells
                .iter()
                .map(|local| {
                    let mut ids = [u32::MAX; 6];
                    for (id, &n) in ids.iter_mut().zip(&local.cell.neighbors) {
                        *id = if n == usize::MAX { u32::MAX } else { n as u32 };
                    }
                    ids
                })
                .collect();
        }
    }
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
    task: Option<Task<FineSet>>,
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

/// The direction the bands are anchored on: the active camera, which is at
/// the player whether walking or flying.
fn player_direction(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    center: Vec3,
) -> Option<Vec3> {
    cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .and_then(|(transform, _)| (transform.translation() - center).try_normalize())
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
) {
    let direction = player_direction(&cameras, frame.center.as_vec3());
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
        if let Some(set) = block_on(poll_once(task)) {
            let took = refresh.in_flight_s().unwrap_or(0.0);
            refresh.task = None;
            refresh.started = None;
            let set = Arc::new(set);
            let behind = direction.map_or(0.0, |d| set.metres_from_anchor(d));
            info!(
                "fine set {} landed after {took:.1} s: {} columns, the player {behind:.0} m from its anchor",
                fine.version + 1,
                set.columns.columns.len()
            );
            contact.set_fine(&set);
            commands.insert_resource(PlanetFine {
                set,
                version: fine.version + 1,
            });
        }
        return;
    }
    let Some(direction) = direction else {
        return;
    };
    let moved = fine.set.metres_from_anchor(direction);
    if refresh.force || moved > REGEN_DISTANCE_M {
        refresh.force = false;
        let settings = settings.clone();
        // The edits travel WITH the task: the tier is rebuilt off the pool and
        // a set built without them would quietly undig every hole the moment
        // the player walked far enough.
        let made = edits.edits.clone();
        refresh.started = Some(std::time::Instant::now());
        refresh.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { generate_fine(direction, &settings, &made) }),
        );
    }
}

/// Per-view LOD inputs the compute and surface shaders read.
pub struct LodParams {
    pub player: Vec3,
    pub bands: Vec4,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn set_with_bands(anchor: Vec3, complete: [f32; 4]) -> FineSet {
        FineSet {
            anchor,
            levels: Default::default(),
            finest_neighbors: Vec::new(),
            finest_radius: 0.0,
            complete,
            columns: ColumnTier::empty(),
        }
    }

    fn metres_away(anchor: Vec3, metres: f32) -> Vec3 {
        let axis = anchor.any_orthonormal_vector();
        Quat::from_axis_angle(axis, metres / PLANET_RADIUS) * anchor
    }

    #[test]
    fn the_level_underfoot_is_the_finest_complete_band_that_reaches_it() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let set = set_with_bands(anchor, BAND_M);
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
    //! A measurement instrument for the near-field-streaming change: what a
    //! fine-set rebuild costs, level by level and then the tier, and what one
    //! column and the worm gather cost on their own. Ignored because it takes
    //! seconds in release and tens of seconds in debug; run it with
    //! `cargo test -p pbd-app --release --lib streaming_cost -- --ignored --nocapture`.
    use super::*;
    use crate::planet::terrain::{PLANET_RADIUS, TERRAIN};
    use std::time::Instant;

    #[test]
    #[ignore]
    fn what_the_near_field_costs_to_build() {
        let anchor = Vec3::new(0.8772014, 0.48012277, 0.0).normalize();
        let settings = ColumnSettings::default();
        let edits = Edits::default();
        let mut lattice = Lattice::default();
        let mut heights = Heights::default();
        let mut finest = Vec::new();
        let mut finest_neighbors = Vec::new();
        let mut total_ms = 0.0;
        for (k, &level) in FINE_LEVELS.iter().enumerate() {
            let margin = REGEN_DISTANCE_M + 3.0 * tile_width_m(level);
            let inner = if k + 1 < FINE_LEVELS.len() {
                (BAND_M[k + 1] - margin).max(0.0)
            } else {
                0.0
            };
            let outer = BAND_M[k] + margin;
            let started = Instant::now();
            let cells =
                lattice.cells_in_band(level, anchor, inner / PLANET_RADIUS, outer / PLANET_RADIUS);
            let laid = started.elapsed().as_secs_f64() * 1000.;
            let records: Vec<GpuCell> = cells
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
                    )
                })
                .collect();
            let recorded = started.elapsed().as_secs_f64() * 1000.;
            total_ms += recorded;
            eprintln!(
                "level {level}: {} cells, lattice {laid:.0} ms, records {:.0} ms, {:.3} ms per cell",
                cells.len(),
                recorded - laid,
                (recorded - laid) / cells.len().max(1) as f64
            );
            if level == FINEST_LEVEL {
                finest_neighbors = cells
                    .iter()
                    .map(|local| {
                        let mut ids = [u32::MAX; 6];
                        for (id, &n) in ids.iter_mut().zip(&local.cell.neighbors) {
                            *id = if n == usize::MAX { u32::MAX } else { n as u32 };
                        }
                        ids
                    })
                    .collect();
                finest = records;
            }
        }
        let started = Instant::now();
        let field = settings.worms();
        let region = pbd_core::worms::gather(&field, &TERRAIN, anchor, settings.reach_m);
        let gathered = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let tier = column::build(anchor, &mut finest, &finest_neighbors, &settings, &edits);
        let built = started.elapsed().as_secs_f64() * 1000.;
        total_ms += built;
        eprintln!(
            "tier: {} columns, worm gather {gathered:.1} ms, build (gather + columns + reconcile + \
             relight) {built:.1} ms, {:.3} ms per column",
            tier.columns.len(),
            built / tier.columns.len().max(1) as f64
        );
        // One column on its own, as an on-demand edit would generate it: the
        // mean over the tier's own directions, gather amortised away.
        let started = Instant::now();
        let mut generated = 0usize;
        for cell in finest.iter().take(500) {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            let column = pbd_core::column::generate_edited(
                &region,
                &field,
                &TERRAIN,
                direction,
                edits.for_cell(cell.metadata[3]),
            );
            generated += usize::from(column.solid(0));
        }
        let single = started.elapsed().as_secs_f64() * 1000. / 500.;
        eprintln!(
            "one column, generated alone: {single:.3} ms (over 500; {generated} solid at the base)"
        );
        let started = Instant::now();
        let whole = generate_fine(anchor, &settings, &edits);
        let all = started.elapsed().as_secs_f64() * 1000.;
        eprintln!(
            "generate_fine whole: {all:.0} ms ({} records, tier {}), parts summed {total_ms:.0} ms",
            whole.levels.iter().map(Vec::len).sum::<usize>(),
            whole.columns.columns.len()
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
