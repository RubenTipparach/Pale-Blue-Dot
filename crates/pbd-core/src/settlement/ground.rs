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

/// A lamp a town stands in its ground (`cities-in-the-world` task 5.2): a
/// lamp block at a cell's centre, in the layer from `altitude_m`, metres
/// over the radius. It is the template's, on the town's stored chart, so it
/// is derived and never saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    pub direction: Vec3,
    pub altitude_m: f32,
    pub material: Material,
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
    /// The exact keys of the footprint's and the margin's cells: where a
    /// player's edit leaves a site unsettled (slice 4a).
    keys: Vec<u32>,
    /// Its street lamps and lanterns (task 5.2).
    lamps: Vec<Lamp>,
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
        let cells: Vec<(usize, Option<Material>, f32)> = footprint
            .iter()
            .map(|&(i, top)| (i, top, terrace))
            .collect();
        Self::terraced(patch, radius_m, &cells, natural)
    }

    /// A town's ground whose footprint stands on several levels (slice 4b):
    /// each footprint cell with its own terrace, metres over the radius, and
    /// each margin cell eased toward the terrace of the cell it was reached
    /// from.
    pub fn terraced(
        patch: &Patch,
        radius_m: f32,
        footprint: &[(usize, Option<Material>, f32)],
        natural: impl Fn(Vec3) -> f32,
    ) -> Self {
        let mut cells = Vec::new();
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        for &(index, top, terrace) in footprint {
            if seen.insert(index) {
                cells.push((index, 0u8, top, terrace));
            }
        }
        let mut ring: Vec<(usize, f32)> = footprint.iter().map(|f| (f.0, f.2)).collect();
        let mut k = 0u8;
        loop {
            k += 1;
            let mut next = Vec::new();
            for &(i, terrace) in &ring {
                for &n in &patch.cells[i].neighbors {
                    if n < patch.cells.len() && seen.insert(n) {
                        next.push((n, terrace));
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            let settled = next.iter().all(|&(i, terrace)| {
                (natural(patch.cells[i].direction) - terrace).abs() <= f32::from(k)
            });
            let last = settled || k > MAX_MARGIN_RINGS;
            let tag = if last { GroundAt::OUTSIDE } else { k };
            for &(i, terrace) in &next {
                cells.push((i, tag, None, terrace));
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
        let keys = cells
            .iter()
            .filter(|c| c.1 != GroundAt::OUTSIDE)
            .map(|c| patch.keys[c.0])
            .collect();
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
                .map(|&(i, ring, top, terrace)| GroundAt {
                    centre: patch.cells[i].direction,
                    ring,
                    terrace,
                    top,
                })
                .collect(),
            keys,
            lamps: Vec::new(),
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

    /// The direction at the middle of the footprint.
    pub fn anchor(&self) -> Vec3 {
        self.anchor
    }

    /// This ground with its town's lamps (task 5.2).
    pub fn with_lamps(mut self, lamps: Vec<Lamp>) -> Self {
        self.lamps = lamps;
        self
    }

    pub fn lamps(&self) -> &[Lamp] {
        &self.lamps
    }

    /// The lamp in the column at a direction, a cell's centre, if the town
    /// stands one there.
    pub fn lamp(&self, direction: Vec3) -> Option<&Lamp> {
        if self.lamps.is_empty() {
            return None;
        }
        let d = direction.normalize_or_zero();
        if d.dot(self.anchor) < self.cos_reach {
            return None;
        }
        // Half a metre off a cell's centre is still that cell's column. By
        // the chord, not the dot product: on a planet of 4800 m the cosine
        // of half a metre is 1.0 in f32, and two equal directions' dot
        // product lands either side of it.
        let near = 0.5 / self.radius_m;
        self.lamps
            .iter()
            .find(|l| (l.direction - d).length_squared() < near * near)
    }

    /// Whether a player's edit lies in the footprint or the margin: a town
    /// laid there would bury it or leave it hanging (task 4.4).
    pub fn touches(&self, edits: &crate::edits::Edits) -> bool {
        !edits.is_empty() && self.keys.iter().any(|&k| !edits.for_cell(k).is_empty())
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

/// Squares a side on each face of the cube the sphere is gridded by, for
/// finding a direction's towns (slice 4a): a square is about 1 km across on
/// the game's planet, many times a town's ground.
const FACE_SQUARES: usize = 8;

/// The square of the cube's grid a direction falls in.
fn square(d: Vec3) -> usize {
    let a = d.abs();
    let (face, u, v, m) = if a.x >= a.y && a.x >= a.z {
        (usize::from(d.x < 0.0), d.y, d.z, a.x)
    } else if a.y >= a.z {
        (2 + usize::from(d.y < 0.0), d.x, d.z, a.y)
    } else {
        (4 + usize::from(d.z < 0.0), d.x, d.y, a.z)
    };
    let q = |t: f32| {
        let f = ((t / m.max(1e-9)) * 0.5 + 0.5) * FACE_SQUARES as f32;
        (f.floor().max(0.0) as usize).min(FACE_SQUARES - 1)
    };
    (face * FACE_SQUARES + q(u)) * FACE_SQUARES + q(v)
}

/// Every town's ground in one world.
#[derive(Clone, Debug)]
pub struct Ground {
    config: TerrainConfig,
    towns: Vec<TownGround>,
    /// The towns whose ground reaches into each square of the cube's grid,
    /// in town order: a height in no town looks at the few in its square,
    /// never at every town.
    squares: Vec<Vec<u16>>,
    digest: u64,
}

impl Ground {
    pub fn new(config: TerrainConfig, towns: Vec<TownGround>) -> Self {
        let digest = towns
            .iter()
            .fold(towns.len() as u64, |h, t| h.rotate_left(7) ^ t.digest());
        let mut squares = vec![Vec::new(); 6 * FACE_SQUARES * FACE_SQUARES];
        for (i, t) in towns.iter().enumerate() {
            // Points over the town's cap, a little past its edge, closer
            // together than the smallest square is wide (a square shrinks
            // toward a cube's corner, to about a third), find every square
            // the cap touches. The game's towns are a few points; a cap as
            // wide as the test sphere's is a few hundred.
            let reach = t.cos_reach.clamp(-1.0, 1.0).acos() * 1.1;
            let step = std::f32::consts::FRAC_PI_2 / FACE_SQUARES as f32 / 6.0;
            let rings = (reach / step).ceil().max(1.0) as usize;
            let mut touched = BTreeSet::from([square(t.anchor)]);
            for r in 1..=rings {
                let rho = reach * r as f32 / rings as f32;
                let around = ((std::f32::consts::TAU * rho.sin()) / step).ceil().max(8.0) as usize;
                for k in 0..around {
                    let a = k as f32 * std::f32::consts::TAU / around as f32;
                    let side = t.east * a.cos() + t.north * a.sin();
                    touched.insert(square(t.anchor * rho.cos() + side * rho.sin()));
                }
            }
            for sq in touched {
                squares[sq].push(i as u16);
            }
        }
        Self {
            config,
            towns,
            squares,
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
        self.squares[square(direction)]
            .iter()
            .find_map(|&i| self.towns[usize::from(i)].at(direction))
    }

    /// The first town's lamp in the column at a direction (task 5.2).
    pub fn lamp(&self, direction: Vec3) -> Option<&Lamp> {
        self.squares[square(direction)]
            .iter()
            .find_map(|&i| self.towns[usize::from(i)].lamp(direction))
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

/// The lamp a town stands in the column at a direction (task 5.2): what
/// [`crate::column::generate_solid`] puts in that column's first layer over
/// the ground, where the layer is air.
pub fn lamp(config: &TerrainConfig, direction: Vec3) -> Option<Lamp> {
    if !ANY.load(Ordering::Acquire) {
        return None;
    }
    let guard = INSTALLED.read().unwrap_or_else(|e| e.into_inner());
    let ground = guard.as_ref()?;
    if ground.config != *config {
        return None;
    }
    ground.lamp(direction).copied()
}

/// Whether a direction is in a town's footprint, where no tree grows.
pub fn cleared(config: &TerrainConfig, direction: Vec3) -> bool {
    with(config, direction, |g| g.ring == 0).unwrap_or(false)
}
