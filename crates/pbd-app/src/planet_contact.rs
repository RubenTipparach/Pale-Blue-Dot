//! Radial contact with the same immutable column records used by the GPU.
//! A small cube-map index seeds a walk over explicit five/six-edge neighbors;
//! gameplay never scans every column or reads geometry back from the GPU.
//!
//! Two tiers: the base level covers the globe, and the finest level's band
//! around the player, swapped in with each regeneration, is what a walker
//! actually stands on. A query inside the fine band walks the fine tier and
//! falls back to the base only where the fine set has no answer.

use super::lod::FineSet;
use super::{GpuCell, PLANET_RADIUS, topology::DualCell};
use bevy::{math::DVec3, prelude::*};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

const INDEX_SIDE: usize = 32;
const INDEX_LEN: usize = 6 * INDEX_SIDE * INDEX_SIDE;

/// A radial intersection with one rendered cap, in the planet's local frame.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceContact {
    /// Stable column identifier, including the twelve pentagons.
    pub cell_id: u32,
    /// Distance from the body center to the exact flat cap triangle, in metres.
    pub radius: f32,
    /// Outward normal of that triangle, rather than the smooth sphere normal.
    pub normal: Vec3,
    /// Explicit preview material ID (zero is ocean).
    pub biome: u32,
    /// Positive for water caps. The preview does not render a seabed beneath it.
    pub water_depth: f32,
    /// The SOLID ground: the seabed under a water cap, and `radius` itself
    /// everywhere else. `radius` is the surface an aircraft must clear and the
    /// rain lands on, which over water is the sea; a walker stands on this
    /// one, which is what lets them wade in and swim rather than walk on the
    /// sheet as if it were a floor.
    pub floor_radius: f32,
}

/// One level's records and neighbour table. `u32::MAX` marks a neighbour the
/// set does not hold, which only the fine tier has.
struct Tier {
    columns: Arc<Vec<GpuCell>>,
    neighbors: Vec<[u32; 6]>,
}

impl Tier {
    /// Walk from `start` toward the cell containing `direction`, stepping
    /// across whichever edge the ray is outside of. `None` when the walk
    /// steps off the set or fails to converge.
    fn walk(&self, start: usize, direction: Vec3) -> Option<usize> {
        let ray = direction.as_dvec3();
        let mut id = start;
        for _ in 0..self.columns.len().min(4096) {
            let cell = &self.columns[id];
            let degree = cell.degree();
            let mut outside = None;
            let mut lowest = -1e-12;
            for side in 0..degree {
                let a = corner(cell, side);
                let b = corner(cell, (side + 1) % degree);
                let distance = a.cross(b).dot(ray);
                if distance < lowest {
                    lowest = distance;
                    outside = Some(side);
                }
            }
            match outside {
                Some(side) => {
                    let next = self.neighbors[id][side];
                    if next == u32::MAX {
                        return None;
                    }
                    id = next as usize;
                }
                None => return Some(id),
            }
        }
        None
    }

    /// The contact on cell `id` along `direction`. This matches the
    /// renderer's triangle fan, including terrace discontinuities and ocean
    /// caps. Callers decide whether water supports an actor.
    fn contact(&self, id: usize, direction: Vec3) -> SurfaceContact {
        let cell = &self.columns[id];
        let axis = Vec3::from_slice(&cell.direction_height[..3]);
        let height = cell.direction_height[3];
        let radius = PLANET_RADIUS + height.max(0.);
        let ray = direction.as_dvec3();
        let center = (axis * radius).as_dvec3();
        let axis = axis.as_dvec3();
        let degree = cell.degree();
        let mut best_side = 0;
        let mut best_score = f64::NEG_INFINITY;
        for side in 0..degree {
            let a = corner(cell, side);
            let b = corner(cell, (side + 1) % degree);
            // Once inside the polygon, these two radial half-planes identify
            // the corresponding wedge of its GPU triangle fan.
            let score = axis.cross(a).dot(ray).min(b.cross(axis).dot(ray));
            if score > best_score {
                best_score = score;
                best_side = side;
            }
        }
        // Multiply in f32 first, exactly as the vertex shader does, then use
        // f64 for the small edge differences at a four-kilometre world radius.
        let a = (corner(cell, best_side).as_vec3() * radius).as_dvec3();
        let b = (corner(cell, (best_side + 1) % degree).as_vec3() * radius).as_dvec3();
        let normal = (a - center).cross(b - center).normalize();
        SurfaceContact {
            cell_id: id as u32,
            radius: (normal.dot(center) / normal.dot(ray)) as f32,
            normal: normal.as_vec3(),
            biome: cell.metadata[1],
            water_depth: (-height).max(0.),
            floor_radius: (normal.dot(center) / normal.dot(ray)) as f32 - (-height).max(0.),
        }
    }
}

/// The finest level around its anchor, with a tangent-plane grid of walk
/// seeds so a query starts within a few cells of its answer.
struct FineTier {
    tier: Tier,
    anchor: Vec3,
    cos_trusted: f32,
    tangent: Vec3,
    bitangent: Vec3,
    grid_m: f32,
    grid: HashMap<(i32, i32), u32>,
}

impl FineTier {
    fn new(set: &FineSet) -> Self {
        let tier = Tier {
            columns: Arc::new(set.levels[3].clone()),
            neighbors: set.finest_neighbors.clone(),
        };
        let anchor = set.anchor;
        let tangent = anchor.any_orthonormal_vector();
        let bitangent = anchor.cross(tangent);
        let grid_m = 3.0 * super::lod::tile_width_m(super::lod::FINEST_LEVEL);
        let mut fine = Self {
            tier,
            anchor,
            // Trust the fine tier one grid cell inside where it is complete.
            cos_trusted: (set.finest_radius() - grid_m / PLANET_RADIUS).cos(),
            tangent,
            bitangent,
            grid_m,
            grid: HashMap::new(),
        };
        for (id, cell) in fine.tier.columns.iter().enumerate() {
            let key = fine.key(Vec3::from_slice(&cell.direction_height[..3]));
            fine.grid.entry(key).or_insert(id as u32);
        }
        fine
    }

    fn key(&self, direction: Vec3) -> (i32, i32) {
        let t = direction.dot(self.tangent) * PLANET_RADIUS / self.grid_m;
        let b = direction.dot(self.bitangent) * PLANET_RADIUS / self.grid_m;
        (t.floor() as i32, b.floor() as i32)
    }

    fn locate(&self, direction: Vec3) -> Option<usize> {
        if direction.dot(self.anchor) < self.cos_trusted {
            return None;
        }
        let (t, b) = self.key(direction);
        // The bucket, then its ring, so a bucket the cells' centres skipped
        // still finds a seed one cell over.
        let seed = (0..9).find_map(|n| self.grid.get(&(t + n % 3 - 1, b + n / 3 - 1)).copied())?;
        self.tier.walk(seed as usize, direction)
    }
}

/// CPU contact authority shares the immutable renderer upload without copying
/// it. The base tier adds six neighbour IDs per cell and a 24 KiB spatial seed
/// table; the fine tier is rebuilt with each fine set. None of it enters the
/// render world.
#[derive(Resource)]
pub struct PlanetContact {
    coarse: Tier,
    seeds: Box<[u32]>,
    fine: Option<FineTier>,
}

impl PlanetContact {
    pub(super) fn new(columns: Arc<Vec<GpuCell>>, cells: &[DualCell]) -> Self {
        let neighbors = cells
            .iter()
            .map(|cell| {
                let mut ids = [u32::MAX; 6];
                for (id, neighbor) in ids.iter_mut().zip(&cell.neighbors) {
                    *id = *neighbor as u32;
                }
                ids
            })
            .collect();
        let mut seeds = vec![u32::MAX; INDEX_LEN];
        let mut scores = vec![f32::NEG_INFINITY; INDEX_LEN];
        for (id, cell) in cells.iter().enumerate() {
            let bin = bin_index(cell.direction);
            let score = cell.direction.dot(bin_direction(bin));
            if score > scores[bin] {
                scores[bin] = score;
                seeds[bin] = id as u32;
            }
        }
        // Low-resolution test planets leave some bins empty. Original icosahedron
        // vertices keep even those seeds in the correct hemisphere without an
        // exhaustive scan. Production bins are all occupied.
        for (bin, seed) in seeds.iter_mut().enumerate() {
            if *seed == u32::MAX {
                let direction = bin_direction(bin);
                *seed = (0..12)
                    .max_by(|&a, &b| {
                        cells[a]
                            .direction
                            .dot(direction)
                            .total_cmp(&cells[b].direction.dot(direction))
                    })
                    .expect("a closed dual sphere contains twelve original vertices")
                    as u32;
            }
        }
        Self {
            coarse: Tier { columns, neighbors },
            seeds: seeds.into_boxed_slice(),
            fine: None,
        }
    }

    /// Swap in the finest level of a fine set as the tier a walker stands on.
    pub fn set_fine(&mut self, set: &FineSet) {
        self.fine = Some(FineTier::new(set));
    }

    /// The base level's records, which tests and the spawn walk read.
    #[cfg(test)]
    fn columns(&self) -> &[GpuCell] {
        &self.coarse.columns
    }

    /// Query a unit ray or any finite body-local position. Inside the fine
    /// band this is the tile a player stands on; elsewhere the base level.
    pub fn sample(&self, direction: Vec3) -> SurfaceContact {
        let direction = direction.try_normalize().unwrap_or(Vec3::Y);
        if let Some(fine) = &self.fine
            && let Some(id) = fine.locate(direction)
        {
            return fine.tier.contact(id, direction);
        }
        let id = self.locate(direction);
        self.coarse.contact(id, direction)
    }

    /// Deterministic breadth-first search for a dry base cap near the
    /// requested ray. Intended for spawning/resetting, not the per-tick path.
    pub fn find_land_near(&self, direction: Vec3) -> Vec3 {
        let direction = direction.try_normalize().unwrap_or(Vec3::Y);
        let start = self.locate(direction);
        let columns = &self.coarse.columns;
        let mut seen = vec![false; columns.len()];
        let mut pending = VecDeque::from([start]);
        seen[start] = true;
        while let Some(id) = pending.pop_front() {
            let cell = &columns[id];
            if cell.direction_height[3] >= 0. {
                return Vec3::from_slice(&cell.direction_height[..3]);
            }
            for &neighbor in &self.coarse.neighbors[id][..cell.degree()] {
                let neighbor = neighbor as usize;
                if !seen[neighbor] {
                    seen[neighbor] = true;
                    pending.push_back(neighbor);
                }
            }
        }
        // The temperate preview contains land. A future wholly oceanic planet
        // will explicitly choose a swimming or vessel spawn instead.
        direction
    }

    fn locate(&self, direction: Vec3) -> usize {
        self.coarse
            .walk(self.seeds[bin_index(direction)] as usize, direction)
            .expect("closed planet contact traversal did not converge")
    }

    /// Builds the same topology and GPU records as the live planet at a smaller
    /// resolution for physical movement and contact regression tests.
    #[cfg(test)]
    pub(crate) fn test_planet(subdivisions: u32) -> Self {
        let cells = super::topology::dual_sphere(subdivisions);
        Self::new(Arc::new(super::lod::base_records(&cells)), &cells)
    }

    /// A real generated terrace for motor regressions: the start lies 0.6 m
    /// inside its lower dry cap, and the unit tangent points through the shared
    /// edge toward a dry cap between 6 and 24 m higher. No test-only geometry or
    /// resampled continuous heights substitute for the renderer's columns.
    #[cfg(test)]
    pub(crate) fn test_terrace() -> (Self, Vec3, Vec3) {
        let cells = super::topology::dual_sphere(5);
        let columns = Arc::new(super::lod::base_records(&cells));
        let contact = Self::new(columns.clone(), &cells);
        for (id, cell) in cells.iter().enumerate() {
            let lower_height = columns[id].direction_height[3];
            if lower_height < 0. {
                continue;
            }
            for (side, &neighbor) in cell.neighbors.iter().enumerate() {
                let rise = columns[neighbor].direction_height[3] - lower_height;
                if !(6. ..=24.).contains(&rise) {
                    continue;
                }
                let a = cell.corners[side];
                let b = cell.corners[(side + 1) % cell.corners.len()];
                let edge = (a + b).normalize();
                let inward = a.cross(b).normalize();
                let angle = 0.6 / (PLANET_RADIUS + lower_height);
                let start = (edge * angle.cos() + inward * angle.sin()).normalize();
                let higher_ray = (edge * angle.cos() - inward * angle.sin()).normalize();
                let heading = (-inward + start * inward.dot(start)).normalize();
                let lower = contact.sample(start);
                let higher = contact.sample(higher_ray);
                assert_eq!(lower.cell_id, id as u32);
                assert_eq!(higher.cell_id, neighbor as u32);
                assert_eq!(lower.water_depth, 0.);
                assert_eq!(higher.water_depth, 0.);
                assert!(higher.radius - lower.radius > 5.5);
                assert!(heading.dot(start).abs() < 1e-5);
                return (contact, start, heading);
            }
        }
        panic!("seeded preview should contain a dry 6–24 m terrace");
    }
}

fn corner(cell: &GpuCell, side: usize) -> DVec3 {
    Vec3::from_slice(&cell.corners[side][..3]).as_dvec3()
}

fn bin_index(direction: Vec3) -> usize {
    let a = direction.abs();
    let (face, u, v) = if a.x >= a.y && a.x >= a.z {
        (
            usize::from(direction.x < 0.),
            direction.y / a.x,
            direction.z / a.x,
        )
    } else if a.y >= a.z {
        (
            2 + usize::from(direction.y < 0.),
            direction.x / a.y,
            direction.z / a.y,
        )
    } else {
        (
            4 + usize::from(direction.z < 0.),
            direction.x / a.z,
            direction.y / a.z,
        )
    };
    let index =
        |value: f32| (((value + 1.) * 0.5 * INDEX_SIDE as f32) as usize).min(INDEX_SIDE - 1);
    face * INDEX_SIDE * INDEX_SIDE + index(v) * INDEX_SIDE + index(u)
}

fn bin_direction(bin: usize) -> Vec3 {
    let face = bin / (INDEX_SIDE * INDEX_SIDE);
    let u = ((bin % INDEX_SIDE) as f32 + 0.5) / INDEX_SIDE as f32 * 2. - 1.;
    let v = ((bin / INDEX_SIDE % INDEX_SIDE) as f32 + 0.5) / INDEX_SIDE as f32 * 2. - 1.;
    let sign = if face.is_multiple_of(2) { 1. } else { -1. };
    match face / 2 {
        0 => Vec3::new(sign, u, v),
        1 => Vec3::new(u, sign, v),
        _ => Vec3::new(u, v, sign),
    }
    .normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_matches_actual_uploaded_fan_triangles_including_pentagons() {
        let contact = PlanetContact::test_planet(3);
        let mut pentagons = 0;
        for (id, cell) in contact.columns().iter().enumerate() {
            let axis = Vec3::from_slice(&cell.direction_height[..3]);
            let radius = PLANET_RADIUS + cell.direction_height[3].max(0.);
            let degree = cell.degree();
            pentagons += usize::from(degree == 5);
            for side in 0..degree {
                let a = corner(cell, side).as_vec3() * radius;
                let b = corner(cell, (side + 1) % degree).as_vec3() * radius;
                // A barycentric point lies on the actual cap, not on the
                // normalized smooth sphere or an independently sampled height.
                let point = axis * radius * 0.2 + a * 0.3 + b * 0.5;
                let sample = contact.sample(point);
                assert_eq!(sample.cell_id, id as u32);
                assert!((sample.radius - point.length()).abs() < 0.002);
                assert!(sample.normal.dot(axis) > 0.99);
                assert_eq!(sample.biome, cell.metadata[1]);
            }
        }
        assert_eq!(pentagons, 12);
    }

    #[test]
    fn raised_cell_boundaries_report_real_step_height_without_interpolation() {
        let cells = super::super::topology::dual_sphere(3);
        let mut columns = super::super::lod::base_records(&cells);
        for cell in &mut columns {
            cell.direction_height[3] = 0.;
            cell.metadata[1] = 2;
        }
        // An original icosahedron vertex is a pentagon; use it as the high cap
        // and test both sides of every wall surrounding it.
        columns[0].direction_height[3] = 6.;
        let contact = PlanetContact::new(Arc::new(columns), &cells);
        let raised = &cells[0];
        for side in 0..raised.corners.len() {
            let edge = (raised.corners[side] + raised.corners[(side + 1) % raised.corners.len()])
                .normalize();
            let inside = edge.lerp(raised.direction, 0.001).normalize();
            let outside = edge
                .lerp(cells[raised.neighbors[side]].direction, 0.001)
                .normalize();
            let high = contact.sample(inside);
            let low = contact.sample(outside);
            assert_eq!(high.cell_id, 0);
            assert_eq!(low.cell_id, raised.neighbors[side] as u32);
            assert!((high.radius - low.radius - 6.).abs() < 0.01);
        }
    }

    #[test]
    fn cube_index_seams_poles_and_water_spawn_remain_valid() {
        let contact = PlanetContact::test_planet(3);
        for bin in 0..INDEX_LEN {
            let ray = bin_direction(bin);
            assert_eq!(bin_index(ray), bin);
            let sample = contact.sample(ray);
            assert!(sample.radius.is_finite() && sample.normal.is_finite());
        }
        for ray in [
            Vec3::Y,
            -Vec3::Y,
            Vec3::X,
            -Vec3::X,
            Vec3::Z,
            -Vec3::Z,
            Vec3::ONE,
        ] {
            assert!(contact.sample(ray).radius.is_finite());
        }
        let ocean = contact
            .columns()
            .iter()
            .find(|c| c.direction_height[3] < 0.)
            .unwrap();
        let ray = Vec3::from_slice(&ocean.direction_height[..3]);
        assert!(contact.sample(ray).water_depth > 0.);
        let dry_ray = contact.find_land_near(ray);
        assert_eq!(contact.sample(dry_ray).water_depth, 0.);
        assert_ne!(contact.sample(dry_ray).biome, 0);
    }

    #[test]
    fn production_resolution_keeps_millimetre_contact_accuracy() {
        let contact = PlanetContact::test_planet(8);
        for id in (0..12).chain((12..contact.columns().len()).step_by(9973)) {
            let cell = &contact.columns()[id];
            let radius = PLANET_RADIUS + cell.direction_height[3].max(0.);
            let center = Vec3::from_slice(&cell.direction_height[..3]) * radius;
            for side in 0..cell.degree() {
                let a = corner(cell, side).as_vec3() * radius;
                let b = corner(cell, (side + 1) % cell.degree()).as_vec3() * radius;
                let point = center * 0.2 + a * 0.3 + b * 0.5;
                let sample = contact.sample(point);
                assert_eq!(sample.cell_id, id as u32);
                assert!((sample.radius - point.length()).abs() < 0.002);
            }
        }
        // Hit every index bin, including the cube face boundaries and poles.
        for bin in 0..INDEX_LEN {
            assert!(contact.sample(bin_direction(bin)).radius.is_finite());
        }
    }

    /// Inside the fine band a query lands on a finest-level cell, which is a
    /// different, smaller tile than the base gives for the same ray, and the
    /// two tiers agree on which side of the sea the ray is.
    #[test]
    fn the_fine_tier_answers_inside_its_band_and_the_base_outside() {
        let mut contact = PlanetContact::test_planet(5);
        let anchor = contact.find_land_near(Vec3::new(0.8776, 0.4794, 0.0));
        let set = super::super::lod::generate_fine(anchor);
        let base = contact.sample(anchor);
        contact.set_fine(&set);
        let fine = contact.sample(anchor);
        assert_ne!(fine.cell_id, base.cell_id);
        assert_eq!(fine.water_depth > 0., base.water_depth > 0.);
        let fine_cell = &set.levels[3][fine.cell_id as usize];
        assert!(Vec3::from_slice(&fine_cell.direction_height[..3]).dot(anchor) > 0.9999);
        // Every ray inside the trusted radius resolves on the fine tier.
        let tangent = anchor.any_orthonormal_vector();
        for k in 0..200 {
            let angle = k as f32 * 0.0314;
            let reach = (k as f32 / 200.0) * 320.0 / PLANET_RADIUS;
            let ray = (anchor * reach.cos()
                + (tangent * angle.cos() + anchor.cross(tangent) * angle.sin()) * reach.sin())
            .normalize();
            let sample = contact.sample(ray);
            assert!(sample.radius.is_finite());
            assert!((sample.cell_id as usize) < set.levels[3].len());
        }
        // Far from the anchor the base answers, on a base cell id.
        let far = contact.sample(-anchor);
        assert!((far.cell_id as usize) < contact.columns().len());
    }
}
