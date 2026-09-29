//! The ground a town stands on (`cities-in-the-world` decision 3): a
//! terrace under its footprint, eased back to the natural ground across a
//! margin, and cleared of trees.
//!
//! It is GENERATED, never edited: every height the planet has comes from
//! [`crate::column::surface_m`], and that asks the ground installed here.
//! The ground is a pure function of the towns it was built from, installed
//! when a world's towns are known and keyed by the world's generator config,
//! so a ground never reaches a world it was not made for.

use super::chart::Patch;
use crate::planet_gen::TerrainConfig;
use crate::terrain::Material;
use glam::Vec3;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// The most rings a margin eases over: a terrace this many layers off the
/// natural ground is met by a cliff past it.
pub const MAX_MARGIN_RINGS: u8 = 12;

/// One cell of a town's ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundAt {
    pub centre: Vec3,
    /// 0 in the footprint, `k` in the margin's `k`th ring, and
    /// [`Self::OUTSIDE`] for the ring past the margin, which is kept only so
    /// a direction there finds its own cell nearest and not a margin cell.
    pub ring: u8,
    /// The terrace, metres over the sea radius, on a layer.
    pub terrace: f32,
    /// The top a lane or a building's floor gives the footprint's cell.
    pub top: Option<Material>,
}

impl GroundAt {
    pub const OUTSIDE: u8 = u8::MAX;

    /// The height here, from the natural one.
    pub fn height(&self, natural: f32) -> f32 {
        match self.ring {
            0 => self.terrace,
            Self::OUTSIDE => natural,
            k => natural.clamp(self.terrace - f32::from(k), self.terrace + f32::from(k)),
        }
    }
}

/// One town's ground.
#[derive(Clone, Debug)]
pub struct TownGround {
    anchor: Vec3,
    cos_reach: f32,
    east: Vec3,
    north: Vec3,
    radius_m: f32,
    bucket_m: f32,
    /// A keyed lookup, never iterated.
    buckets: HashMap<(i32, i32), Vec<u32>>,
    cells: Vec<GroundAt>,
}

impl TownGround {
    /// A town's ground from its footprint cells (patch index and the top
    /// they take), its terrace, and the natural height at a direction.
    /// The margin rings run out through the patch's neighbour tables until a
    /// whole ring already lies within its easing of the terrace, or
    /// [`MAX_MARGIN_RINGS`].
    pub fn new(
        patch: &Patch,
        radius_m: f32,
        footprint: &[(usize, Option<Material>)],
        terrace: f32,
        natural: impl Fn(Vec3) -> f32,
    ) -> Self {
        let mut cells = Vec::new();
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        for &(index, top) in footprint {
            if seen.insert(index) {
                cells.push((index, 0u8, top));
            }
        }
        let mut ring: Vec<usize> = footprint.iter().map(|f| f.0).collect();
        let mut k = 0u8;
        loop {
            k += 1;
            let mut next = Vec::new();
            for &i in &ring {
                for &n in &patch.cells[i].neighbors {
                    if n < patch.cells.len() && seen.insert(n) {
                        next.push(n);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            let settled = next
                .iter()
                .all(|&i| (natural(patch.cells[i].direction) - terrace).abs() <= f32::from(k));
            let last = settled || k > MAX_MARGIN_RINGS;
            let tag = if last { GroundAt::OUTSIDE } else { k };
            for &i in &next {
                cells.push((i, tag, None));
            }
            if last {
                break;
            }
            ring = next;
        }
        let anchor = footprint
            .iter()
            .fold(Vec3::ZERO, |sum, f| sum + patch.cells[f.0].direction)
            .normalize_or(Vec3::Y);
        let (north, east) = crate::geo::north_east(anchor);
        let cos_reach = cells
            .iter()
            .map(|c| patch.cells[c.0].direction.dot(anchor))
            .fold(1.0f32, f32::min)
            - 1e-6;
        let bucket_m = 3.0;
        let mut ground = Self {
            anchor,
            cos_reach,
            east,
            north,
            radius_m,
            bucket_m,
            buckets: HashMap::new(),
            cells: cells
                .iter()
                .map(|&(i, ring, top)| GroundAt {
                    centre: patch.cells[i].direction,
                    ring,
                    terrace,
                    top,
                })
                .collect(),
        };
        for (i, cell) in ground.cells.iter().enumerate() {
            let key = ground.bucket(cell.centre);
            ground.buckets.entry(key).or_default().push(i as u32);
        }
        ground
    }

    fn bucket(&self, d: Vec3) -> (i32, i32) {
        let x = d.dot(self.east) * self.radius_m / self.bucket_m;
        let y = d.dot(self.north) * self.radius_m / self.bucket_m;
        (x.floor() as i32, y.floor() as i32)
    }

    /// The town's cell at a direction, or `None` off its ground.
    pub fn at(&self, direction: Vec3) -> Option<&GroundAt> {
        let d = direction.normalize_or_zero();
        if d.dot(self.anchor) < self.cos_reach {
            return None;
        }
        let (bx, by) = self.bucket(d);
        let mut best: Option<(f32, u32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for &i in self.buckets.get(&(bx + dx, by + dy)).into_iter().flatten() {
                    let dot = self.cells[i as usize].centre.dot(d);
                    if best.is_none_or(|(b, j)| dot > b || (dot == b && i < j)) {
                        best = Some((dot, i));
                    }
                }
            }
        }
        let cell = &self.cells[best?.1 as usize];
        (cell.ring != GroundAt::OUTSIDE).then_some(cell)
    }

    /// How many cells are the footprint, and how many the margin.
    pub fn counts(&self) -> (usize, usize) {
        let footprint = self.cells.iter().filter(|c| c.ring == 0).count();
        let margin = self
            .cells
            .iter()
            .filter(|c| c.ring != 0 && c.ring != GroundAt::OUTSIDE)
            .count();
        (footprint, margin)
    }

    /// A number that changes when the ground does: the map's raster cache
    /// is named by it.
    fn digest(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut mix = |v: u64| {
            h ^= v;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for c in &self.cells {
            mix(u64::from(c.centre.x.to_bits()));
            mix(u64::from(c.centre.y.to_bits()));
            mix(u64::from(c.ring));
            mix(u64::from(c.terrace.to_bits()));
            mix(c.top.map_or(0, |m| m as u64 + 1));
        }
        h
    }
}

/// Every town's ground in one world.
#[derive(Clone, Debug)]
pub struct Ground {
    config: TerrainConfig,
    towns: Vec<TownGround>,
    digest: u64,
}

impl Ground {
    pub fn new(config: TerrainConfig, towns: Vec<TownGround>) -> Self {
        let digest = towns
            .iter()
            .fold(towns.len() as u64, |h, t| h.rotate_left(7) ^ t.digest());
        Self {
            config,
            towns,
            digest,
        }
    }

    pub fn digest(&self) -> u64 {
        self.digest
    }

    pub fn towns(&self) -> &[TownGround] {
        &self.towns
    }

    /// The first town's cell at a direction.
    pub fn at(&self, direction: Vec3) -> Option<&GroundAt> {
        self.towns.iter().find_map(|t| t.at(direction))
    }
}

static INSTALLED: RwLock<Option<Arc<Ground>>> = RwLock::new(None);
/// Whether any ground is installed: the one load a direction with no towns
/// pays.
static ANY: AtomicBool = AtomicBool::new(false);

/// Install a world's ground, or none. The planet built before this call is
/// on the old ground: the caller rebuilds it.
pub fn install(ground: Option<Ground>) {
    let any = ground.is_some();
    *INSTALLED.write().unwrap_or_else(|e| e.into_inner()) = ground.map(Arc::new);
    ANY.store(any, Ordering::Release);
}

/// The installed ground.
pub fn installed() -> Option<Arc<Ground>> {
    if !ANY.load(Ordering::Acquire) {
        return None;
    }
    INSTALLED.read().unwrap_or_else(|e| e.into_inner()).clone()
}

fn with<T>(config: &TerrainConfig, direction: Vec3, f: impl FnOnce(&GroundAt) -> T) -> Option<T> {
    if !ANY.load(Ordering::Acquire) {
        return None;
    }
    let guard = INSTALLED.read().unwrap_or_else(|e| e.into_inner());
    let ground = guard.as_ref()?;
    if ground.config != *config {
        return None;
    }
    ground.at(direction).map(f)
}

/// The surface at a direction, from its natural height: what
/// [`crate::column::surface_m`] answers.
pub fn surface(config: &TerrainConfig, direction: Vec3, natural: f32) -> f32 {
    with(config, direction, |g| g.height(natural)).unwrap_or(natural)
}

/// The top a town gives the ground at a direction, where it gives one.
pub fn top(config: &TerrainConfig, direction: Vec3) -> Option<Material> {
    with(config, direction, |g| g.top).flatten()
}

/// Whether a direction is in a town's footprint, where no tree grows.
pub fn cleared(config: &TerrainConfig, direction: Vec3) -> bool {
    with(config, direction, |g| g.ring == 0).unwrap_or(false)
}
