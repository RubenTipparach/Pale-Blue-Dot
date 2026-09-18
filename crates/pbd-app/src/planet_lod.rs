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
use super::lattice::{Lattice, LocalCell};
use super::terrain::{PLANET_RADIUS, surface_code, surface_height};
use super::topology::{DualCell, midpoint};
use bevy::{
    prelude::*,
    render::extract_resource::ExtractResource,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
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
        floors[side] = heights.at(midpoint(source.direction, neighbor));
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
}

impl FineSet {
    /// The angular radius the finest level is complete to, which is where the
    /// walker's contact stops trusting it.
    pub fn finest_radius(&self) -> f32 {
        self.finest_radius
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
pub fn generate_fine(anchor: Vec3) -> FineSet {
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
    FineSet {
        anchor,
        levels,
        finest_neighbors,
        finest_radius,
        complete,
    }
}

/// The current fine set, shared with the render world by `Arc`; the version
/// says when the fine regions of the storage buffer need rewriting.
#[derive(Resource, Clone, ExtractResource)]
pub struct PlanetFine {
    pub set: Arc<FineSet>,
    pub version: u64,
}

#[derive(Resource, Default)]
pub struct LodRefresh {
    task: Option<Task<FineSet>>,
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

/// Rebuild the fine set on the compute pool once the player has walked
/// `REGEN_DISTANCE_M` from its anchor, and swap it in, with the walker's
/// contact, when it lands. One rebuild in flight at a time.
pub fn refresh_lod(
    mut commands: Commands,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Res<super::PlanetRenderFrame>,
    fine: Res<PlanetFine>,
    mut refresh: ResMut<LodRefresh>,
    mut contact: ResMut<super::PlanetContact>,
) {
    if let Some(task) = refresh.task.as_mut() {
        if let Some(set) = block_on(poll_once(task)) {
            refresh.task = None;
            let set = Arc::new(set);
            contact.set_fine(&set);
            commands.insert_resource(PlanetFine {
                set,
                version: fine.version + 1,
            });
        }
        return;
    }
    let Some(direction) = player_direction(&cameras, frame.center.as_vec3()) else {
        return;
    };
    let moved = direction.dot(fine.set.anchor).clamp(-1.0, 1.0).acos() * PLANET_RADIUS;
    if moved > REGEN_DISTANCE_M {
        refresh.task =
            Some(AsyncComputeTaskPool::get().spawn(async move { generate_fine(direction) }));
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
        let set = generate_fine(Vec3::new(0.8776, 0.4794, 0.0));
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
        let set = generate_fine(anchor);
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
        let set = generate_fine(Vec3::new(0.3, 0.8, -0.5));
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
