//! Rivers that run down to the sea, reach by reach (generator version 8,
//! `ocean-environments` design 13).
//!
//! **The stage is a field.** A column is a pure function of its direction, so
//! a river's water level cannot come from a walk done when a tier is built:
//! two tiers would disagree at their seam. It comes instead from one
//! drainage map of the whole body, built once per world from the generator's
//! own ground and nothing else, so every tier, every thread and every run
//! reads the same river.
//!
//! **The map** is a latitude-longitude lattice (`cell_deg`, 0.5 degrees: 42 m
//! at the equator). Its drainage is the priority flood: from every sea cell
//! outward in rising order of the level water would spill at, each land cell
//! drains into the neighbour it was reached from. So every land cell drains
//! to the sea by construction, down a path whose spill level never rises,
//! and a hollow is crossed at its spill rather than trapping the water. A
//! cell is river where enough cells drain through it (`min_catchment`).
//!
//! **A reach** is the stretch of river from one lattice cell to the next
//! downstream. Its water stands at that cell's spill less a metre, so it
//! never rises downstream; where it falls between reaches the drop is a
//! riffle (one layer) or a fall (more), at the lattice cell where the reach
//! begins.

use crate::planet_gen::{self, TerrainConfig};
use crate::terrain::Material;
use glam::Vec3;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Mutex, OnceLock};

/// How rivers are laid out (generator version 8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverSettings {
    /// The drainage lattice's spacing, degrees.
    pub cell_deg: f32,
    /// How many lattice cells must drain through a cell for it to be river.
    pub min_catchment: u32,
    /// Rivers rise no higher than this, metres over the sea: springs are in
    /// the hills, not on the peaks.
    pub max_spring_m: f32,
    /// Half the river's width at its smallest, a stream at its spring,
    /// metres; it widens with its catchment up to `max_half_width_m`. And
    /// the banks beside it.
    pub half_width_m: f32,
    pub max_half_width_m: f32,
    pub bank_m: f32,
    /// How far the river wanders either side of the lattice's straight line,
    /// metres, along a smooth field.
    pub meander_m: f32,
    /// Marsh pools stand only where the ground round them stands as high
    /// (the rim), sampled this far out, metres.
    pub rim_m: f32,
}

impl RiverSettings {
    pub const V8: Self = Self {
        cell_deg: 0.5,
        min_catchment: 40,
        max_spring_m: 90.0,
        half_width_m: 2.2,
        max_half_width_m: 9.0,
        bank_m: 7.0,
        meander_m: 7.0,
        rim_m: 2.9,
    };
}

/// The spacing a sheet's levels fall by, metres: one cell.
const CELL_M: f32 = 2.833;
const MEANDER_SALT: u64 = 0x3EA_DE55;
/// How wide the levee round a river's valley is, metres.
const LEVEE_M: f32 = 3.0;
/// How far the levee's outer slope runs down to the ground, at most, metres.
const LEVEE_FOOT_M: f32 = 12.0;
/// How far past a river's banks its ground is sealed from caves, metres.
const SEAL_M: f32 = LEVEE_M + LEVEE_FOOT_M;
const NONE: u32 = u32::MAX;

/// One lattice cell of the drainage map.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Node {
    /// The cell it drains into, or `NONE` for the sea.
    down: u32,
    /// Whether it is river.
    river: bool,
    /// Its water's top, whole metres over sea level (river cells).
    level: i32,
    /// The most any river reach feeding it stands above it, metres.
    drop: i32,
    /// Half its width, metres: wider the more drains through it.
    half_width: f32,
}

/// One cell of a river's [`RiverMap::course`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CourseStep {
    /// The lattice cell's centre, a unit direction.
    pub centre: Vec3,
    /// The river's half width there, metres.
    pub half_width_m: f32,
    /// Its water's top, whole metres over sea level.
    pub level: i32,
    /// False for the last step: the sea cell the river runs into.
    pub river: bool,
}

/// The body's drainage, built once per world.
#[derive(Debug, PartialEq)]
pub struct RiverMap {
    width: usize,
    height: usize,
    cell_deg: f32,
    nodes: Vec<Node>,
}

/// A river at a point: how far from its line, how far down its reach, its
/// water's top and the drop into the reach.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverHere {
    /// Metres from the river's line.
    pub across_m: f32,
    /// Metres down the reach from where it begins.
    pub along_m: f32,
    /// The reach's water top, whole metres over sea level.
    pub level: i32,
    /// How far the water falls into this reach where it begins, metres.
    pub drop: i32,
    /// Up the river from here: a unit tangent at the point.
    pub upstream: Vec3,
    /// Half the river's width here, metres.
    pub half_width_m: f32,
}

impl RiverMap {
    fn centre(&self, index: usize) -> Vec3 {
        let (i, j) = (index % self.width, index / self.width);
        let lat = (-90.0 + (j as f32 + 0.5) * self.cell_deg).to_radians();
        let lon = (-180.0 + (i as f32 + 0.5) * self.cell_deg).to_radians();
        Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
    }

    fn cell_of(&self, d: Vec3) -> (i64, i64) {
        let lat = d.y.clamp(-1.0, 1.0).asin().to_degrees();
        let lon = d.z.atan2(d.x).to_degrees();
        (
            ((lon + 180.0) / self.cell_deg).floor() as i64,
            ((lat + 90.0) / self.cell_deg).floor() as i64,
        )
    }

    fn index(&self, i: i64, j: i64) -> Option<usize> {
        if j < 0 || j >= self.height as i64 {
            return None;
        }
        let i = i.rem_euclid(self.width as i64);
        Some(j as usize * self.width + i as usize)
    }

    /// Build the drainage of `cfg`'s ground (with no rivers in it), its
    /// ground sampled on every core.
    pub fn build(cfg: &TerrainConfig, settings: &RiverSettings) -> Self {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        Self::build_on(cfg, settings, threads)
    }

    /// [`Self::build`] on `threads` threads (`river-cache` design 2). The
    /// lattice's altitudes are pure and independent, so they are sampled in
    /// chunks on scoped threads and joined in order; the flood after them is
    /// one thread. The map does not depend on the count.
    pub fn build_on(cfg: &TerrainConfig, settings: &RiverSettings, threads: usize) -> Self {
        let width = (360.0 / settings.cell_deg).round() as usize;
        let height = (180.0 / settings.cell_deg).round() as usize;
        let mut map = Self {
            width,
            height,
            cell_deg: settings.cell_deg,
            nodes: vec![
                Node {
                    down: NONE,
                    river: false,
                    level: 0,
                    drop: 0,
                    half_width: 0.0,
                };
                width * height
            ],
        };
        let ground = TerrainConfig {
            rivers: None,
            ..*cfg
        };
        let sea = cfg.sea_level_m;
        let mut alt = vec![0.0f32; width * height];
        let chunk = (width * height).div_ceil(threads.max(1)).max(1);
        std::thread::scope(|scope| {
            for (n, out) in alt.chunks_mut(chunk).enumerate() {
                let (map, ground) = (&map, &ground);
                scope.spawn(move || {
                    for (i, a) in out.iter_mut().enumerate() {
                        *a = planet_gen::surface_altitude(ground, map.centre(n * chunk + i));
                    }
                });
            }
        });
        // The priority flood, in a fixed order: the spill in millimetres,
        // then the index.
        let key = |m: f32| (m * 1000.0).round() as i64;
        let mut spill = vec![f32::NAN; width * height];
        let mut heap = BinaryHeap::new();
        for (k, &a) in alt.iter().enumerate() {
            if a < sea {
                spill[k] = sea;
                heap.push(Reverse((key(sea), k as u32)));
            }
        }
        let mut order = Vec::with_capacity(width * height);
        let mut done = vec![false; width * height];
        while let Some(Reverse((_, k))) = heap.pop() {
            let k = k as usize;
            if done[k] {
                continue;
            }
            done[k] = true;
            order.push(k);
            let (i, j) = ((k % width) as i64, (k / width) as i64);
            for (di, dj) in [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                let Some(n) = map.index(i + di, j + dj) else {
                    continue;
                };
                if done[n] || !spill[n].is_nan() && spill[n] <= alt[n].max(spill[k]) {
                    continue;
                }
                let s = alt[n].max(spill[k]);
                spill[n] = s;
                map.nodes[n].down = k as u32;
                heap.push(Reverse((key(s), n as u32)));
            }
        }
        // Sea cells drain nowhere.
        for (k, &a) in alt.iter().enumerate() {
            if a < sea {
                map.nodes[k].down = NONE;
            }
        }
        // What drains through each cell: upstream first.
        let mut catchment = vec![1u32; width * height];
        for &k in order.iter().rev() {
            let down = map.nodes[k].down;
            if down != NONE {
                catchment[down as usize] += catchment[k];
            }
        }
        for &k in &order {
            let node = &mut map.nodes[k];
            node.level = ((spill[k] - sea).floor() as i32 - 1).max(0);
            node.river = alt[k] >= sea
                && catchment[k] >= settings.min_catchment
                && spill[k] < sea + settings.max_spring_m;
            // A stream at its spring, a metre and a half wider each time
            // its catchment doubles.
            let grown = (catchment[k] as f32 / settings.min_catchment as f32)
                .max(1.0)
                .log2();
            node.half_width = (settings.half_width_m + 1.5 * grown).min(settings.max_half_width_m);
        }
        // The sea cell a river runs into is part of its mouth.
        for k in 0..width * height {
            if map.nodes[k].river {
                let down = map.nodes[k].down;
                if down != NONE {
                    let (level, drop) = (map.nodes[k].level, map.nodes[down as usize].level);
                    let d = &mut map.nodes[down as usize];
                    d.drop = d.drop.max(level - drop);
                }
            }
        }
        map
    }

    /// The river at a direction, if `d` is within its banks' reach.
    pub fn at(&self, cfg: &TerrainConfig, settings: &RiverSettings, d: Vec3) -> Option<RiverHere> {
        self.at_widened(cfg, settings, d, 0.0)
    }

    /// [`Self::at`] with the banks' reach widened by `extra` metres: what a
    /// far cell, wider than the river, asks.
    pub fn at_widened(
        &self,
        cfg: &TerrainConfig,
        settings: &RiverSettings,
        d: Vec3,
        extra: f32,
    ) -> Option<RiverHere> {
        let d = d.normalize_or(Vec3::Y);
        let radius = cfg.radius_m;
        let (ci, cj) = self.cell_of(d);
        let mut best: Option<RiverHere> = None;
        let mut best_q = f32::INFINITY;
        for dj in -2..=2 {
            for di in -2..=2 {
                let Some(k) = self.index(ci + di, cj + dj) else {
                    continue;
                };
                let node = self.nodes[k];
                if !node.river {
                    continue;
                }
                let a = self.centre(k);
                let b = if node.down == NONE {
                    a
                } else {
                    self.centre(node.down as usize)
                };
                // The tangent plane at `d`, in metres.
                let flat = |v: Vec3| (v - d * v.dot(d)) * radius;
                let (pa, pb) = (flat(a), flat(b));
                let seg = pb - pa;
                let len2 = seg.length_squared();
                let t = if len2 > 1e-6 {
                    (-pa.dot(seg) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let closest = pa + seg * t;
                // The line wanders along a smooth field read at its own point,
                // so the river curves rather than running ruler-straight.
                let side = if len2 > 1e-6 {
                    d.cross(seg / len2.sqrt())
                } else {
                    Vec3::ZERO
                };
                let on_line = (d + closest / radius).normalize_or(d);
                let wander =
                    (planet_gen::noise01(cfg.seed ^ MEANDER_SALT, on_line, cfg.scale_of(60.0), 2)
                        - 0.5)
                        * 2.0
                        * settings.meander_m;
                let q = (-closest - side * wander).length();
                let reach = node.half_width + settings.bank_m + extra;
                if q < reach && q - node.half_width < best_q {
                    best_q = q - node.half_width;
                    best = Some(RiverHere {
                        half_width_m: node.half_width,
                        across_m: q,
                        along_m: t * len2.sqrt(),
                        level: node.level,
                        drop: node.drop,
                        upstream: if len2 > 1e-6 {
                            -seg / len2.sqrt()
                        } else {
                            Vec3::ZERO
                        },
                    });
                }
            }
        }
        best
    }

    /// Every river cell's way to the sea, for a test: the levels down its
    /// path, which must never rise, and whether it reaches the sea.
    pub fn path_to_sea(&self, start: usize) -> (Vec<i32>, bool) {
        let mut levels = Vec::new();
        let mut k = start;
        for _ in 0..self.nodes.len() {
            let node = self.nodes[k];
            if !node.river {
                // A river's downstream is a river or the sea.
                return (levels, node.down == NONE);
            }
            levels.push(node.level);
            if node.down == NONE {
                return (levels, true);
            }
            k = node.down as usize;
        }
        (levels, false)
    }

    /// The river cells, for a test.
    pub fn rivers(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.nodes.len()).filter(|&k| self.nodes[k].river)
    }

    /// A lattice cell's centre and water level, for a capture's search.
    pub fn describe(&self, k: usize) -> (Vec3, i32, i32, bool) {
        let n = self.nodes[k];
        (self.centre(k), n.level, n.drop, n.down == NONE)
    }

    /// Half a river cell's width, metres.
    pub fn half_width(&self, k: usize) -> f32 {
        self.nodes[k].half_width
    }

    /// The cell a river cell drains into.
    pub fn down(&self, k: usize) -> Option<usize> {
        let d = self.nodes[k].down;
        (d != NONE).then_some(d as usize)
    }

    /// The whole river through the river cell nearest `d` within `within`
    /// cells of its lattice cell, from its source to the cell it meets the
    /// sea in (`river-cache` design 8, the river walk): up from that cell by
    /// the widest feeder (the one most water drains through) until no river
    /// feeds it, then down to the sea. Each step is a cell's centre, its
    /// river's half width, m, and its level. Empty with no river near.
    pub fn course(&self, d: Vec3, within: i64) -> Vec<CourseStep> {
        let d = d.normalize_or(Vec3::Y);
        let (ci, cj) = self.cell_of(d);
        let mut nearest: Option<(f32, usize)> = None;
        for dj in -within..=within {
            for di in -within..=within {
                let Some(k) = self.index(ci + di, cj + dj) else {
                    continue;
                };
                if !self.nodes[k].river {
                    continue;
                }
                let gap = self.centre(k).angle_between(d);
                if nearest.is_none_or(|(g, _)| gap < g) {
                    nearest = Some((gap, k));
                }
            }
        }
        let Some((_, start)) = nearest else {
            return Vec::new();
        };
        // Up: a feeder drains into its neighbour, so the eight cells round a
        // cell hold every one; the widest, ties to the lowest index.
        let mut source = start;
        for _ in 0..self.nodes.len() {
            let (i, j) = (source % self.width, source / self.width);
            let mut best: Option<usize> = None;
            for dj in -1i64..=1 {
                for di in -1i64..=1 {
                    let Some(k) = self.index(i as i64 + di, j as i64 + dj) else {
                        continue;
                    };
                    let n = self.nodes[k];
                    if k == source || !n.river || n.down != source as u32 {
                        continue;
                    }
                    if best.is_none_or(|b| n.half_width > self.nodes[b].half_width) {
                        best = Some(k);
                    }
                }
            }
            match best {
                Some(up) => source = up,
                None => break,
            }
        }
        let mut out = Vec::new();
        let mut k = source;
        for _ in 0..self.nodes.len() {
            let n = self.nodes[k];
            out.push(CourseStep {
                centre: self.centre(k),
                half_width_m: n.half_width,
                level: n.level,
                river: n.river,
            });
            if !n.river || n.down == NONE {
                break;
            }
            k = n.down as usize;
        }
        out
    }
}

type Maps = Mutex<HashMap<(u64, u32, u32), &'static OnceLock<RiverMap>>>;

/// The drainage map of a world, built the first time it is asked for. One
/// cell is leaked per world, which lives as long as the program.
pub fn map(cfg: &TerrainConfig, settings: &RiverSettings) -> &'static RiverMap {
    static MAPS: OnceLock<Maps> = OnceLock::new();
    type Last = std::cell::Cell<Option<((u64, u32, u32), &'static OnceLock<RiverMap>)>>;
    thread_local! {
        // Every altitude asks: the last world's map without the lock.
        static LAST: Last = const { std::cell::Cell::new(None) };
    }
    let key = (
        cfg.seed,
        settings.cell_deg.to_bits(),
        settings.min_catchment,
    );
    if let Some((k, cell)) = LAST.with(|l| l.get())
        && k == key
        && let Some(map) = cell.get()
    {
        return map;
    }
    let cell = *MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("river maps")
        .entry(key)
        .or_insert_with(|| Box::leak(Box::new(OnceLock::new())));
    let map = cell.get_or_init(|| RiverMap::build(cfg, settings));
    LAST.with(|l| l.set(Some((key, cell))));
    map
}

/// The river at a direction under a config with rivers.
pub fn at(cfg: &TerrainConfig, d: Vec3) -> Option<RiverHere> {
    let settings = cfg.rivers.as_ref()?;
    map(cfg, settings).at(cfg, settings, d)
}

/// Whether a river's banks reach within `extra_m` of `d`: the cheap read a
/// caller that tests an area with [`floods`] (the map's pixel, `map-rivers`)
/// makes first, since no water stands anywhere this says no.
pub fn near(cfg: &TerrainConfig, d: Vec3, extra_m: f32) -> bool {
    let Some(settings) = cfg.rivers.as_ref() else {
        return false;
    };
    map(cfg, settings)
        .at_widened(cfg, settings, d, extra_m.max(0.0))
        .is_some()
}

/// Where a river's valley and a margin past its banks are sealed from
/// caves (`water-flow` E16): the altitude, metres over the radius, above
/// which no cave opens in a column here. Four metres under the sea's level,
/// under every reach's bed and plunge pool: a reach's own level is not
/// enough, because at a fall the reach below runs through the same ground
/// and a cave under its bed would drain it.
pub fn sealed_from(cfg: &TerrainConfig, d: Vec3) -> Option<f32> {
    let settings = cfg.rivers.as_ref()?;
    map(cfg, settings).at_widened(cfg, settings, d, SEAL_M)?;
    Some(cfg.sea_level_m - 4.0)
}

/// The water's top at a point of a river, whole metres over sea level: the
/// reach's level, or a metre under the ground the river cuts through where
/// that is lower, so a river never stands over its banks (no levee is ever
/// raised; a dip in the ground holds the river lower there).
fn top_here(
    cfg: &TerrainConfig,
    settings: &RiverSettings,
    here: &RiverHere,
    d: Vec3,
    base: f32,
) -> i32 {
    // The ground round the valley's rim as well as under the line, so the
    // water is held by its banks on every side and never spills onto the
    // land beside a river that runs a little high.
    let d = d.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
    let north = d.cross(east);
    let rim = (here.half_width_m + settings.bank_m) / cfg.radius_m;
    let low = (0..6)
        .map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 6.0;
            let n = (d + (east * a.cos() + north * a.sin()) * rim).normalize();
            planet_gen::surface_altitude_unrivered(cfg, n)
        })
        .fold(base, f32::min);
    // A layer under the lowest of it; the levee round the valley
    // (`water-flow` E16) holds what runs out over a low rim.
    here.level
        .min((low - cfg.sea_level_m).floor() as i32 - 1)
        .max(0)
}

/// The ground a river makes of `base` at `d`: its bed two layers under its
/// water in the channel. The banks are the ground as it was.
pub fn carve(cfg: &TerrainConfig, d: Vec3, base: f32) -> f32 {
    let Some(settings) = cfg.rivers.as_ref() else {
        return base;
    };
    if base < cfg.sea_level_m - 1.0 {
        return base;
    }
    let Some(here) = map(cfg, settings).at_widened(cfg, settings, d, LEVEE_M + LEVEE_FOOT_M) else {
        return base;
    };
    if here.across_m >= here.half_width_m + settings.bank_m {
        // A levee round the valley, as high as any water in the valley near
        // it (its reach's level, and the reach above where it falls in),
        // and a layer over: a river that runs along a slope cannot spill out
        // over its low side.
        // Flat for its width, then down at one in one to the ground: never
        // a wall a coarse tier's seam cannot close.
        let past = here.across_m - here.half_width_m - settings.bank_m - LEVEE_M;
        return base.max(levee_m(cfg, &here) - past.max(0.0));
    }
    let top = top_here(cfg, settings, &here, d, base);
    let water = cfg.sea_level_m + top as f32;
    // Under a fall of three layers or more, a plunge pool: two layers deeper
    // for two cells downstream of the step (M25's river tab).
    let plunge = if top > 0 {
        let map = map(cfg, settings);
        (1..=2).any(|n| {
            let at = (d.normalize_or(Vec3::Y) + here.upstream * (n as f32 * CELL_M / cfg.radius_m))
                .normalize();
            top_at(cfg, settings, map, at).is_some_and(|up| up >= top + 3)
        })
    } else {
        false
    };
    let core: f32 = 1.5;
    let core = core.max(here.half_width_m - 1.5);
    let outer = here.half_width_m + settings.bank_m;
    let t = ((here.across_m - core) / (outer - core)).clamp(0.0, 1.0);
    // The pool's extra depth eases out up the bank, so its edge is a slope
    // and not a two-metre step.
    let bed = (water - if plunge { 4.0 - 2.0 * t } else { 2.0 } + 0.01).min(base);
    // A valley, not a slot: the bed is flat for a metre and a half either
    // side of the line, then rises evenly to the ground at the banks' outer
    // edge, so no step across the river is taller than the ground a coarse
    // tier's walls can reach (the seam report's slits).
    // The banks rise to the levee where it stands over the ground, so the
    // valley meets it without a step.
    let rim = base.max(levee_m(cfg, &here));
    (bed + (rim - bed) * t).min(rim)
}

/// The levee's top round a reach, metres over the radius: a layer over its
/// water and the fall into it.
fn levee_m(cfg: &TerrainConfig, here: &RiverHere) -> f32 {
    cfg.sea_level_m + (here.level + here.drop + 1) as f32 + 0.01
}

/// The water a river puts in a column over its bed: `(altitude of the
/// layer's top, material)`, bottom up, sources to the water's top here.
pub fn water(cfg: &TerrainConfig, d: Vec3) -> Vec<(i32, Material)> {
    let Some(settings) = cfg.rivers.as_ref() else {
        return Vec::new();
    };
    let map = map(cfg, settings);
    let d = d.normalize_or(Vec3::Y);
    let Some(top) = top_at(cfg, settings, map, d) else {
        return Vec::new();
    };
    let sea = cfg.sea_level_m.round() as i32;
    // Sources from the ground the valley leaves here up to the reach's top:
    // still water on its own bed, never over open air. What flows between
    // reaches is the rule's, settled when a tier is built
    // (`planet::column::ColumnTier::settle_rivers`), so it is exactly what
    // the rule keeps.
    let floor = planet_gen::surface_altitude(cfg, d).floor() as i32 - sea;
    if floor >= top {
        return Vec::new();
    }
    let out: Vec<(i32, Material)> = (floor + 1..=top)
        .map(|t| (sea + t, Material::Water))
        .collect();
    out
}

/// Whether a river's water stands over the ground at `d` (`flora-v2`
/// 12.10): the same answer as [`water`] being non-empty, without building
/// the list. What the far records ask to draw a river's bed as a bed.
pub fn floods(cfg: &TerrainConfig, d: Vec3) -> bool {
    let Some(settings) = cfg.rivers.as_ref() else {
        return false;
    };
    let d = d.normalize_or(Vec3::Y);
    let Some(top) = top_at(cfg, settings, map(cfg, settings), d) else {
        return false;
    };
    let sea = cfg.sea_level_m.round() as i32;
    (planet_gen::surface_altitude(cfg, d).floor() as i32 - sea) < top
}

/// The water's top at a point of a river's channel, whole metres over sea
/// level, or `None` off the channel or at the sea.
fn top_at(cfg: &TerrainConfig, settings: &RiverSettings, map: &RiverMap, d: Vec3) -> Option<i32> {
    // Anywhere the valley's ground is under the reach's water, banks too.
    let here = map.at(cfg, settings, d)?;
    if here.level <= 0 {
        return None;
    }
    let base = planet_gen::surface_altitude_unrivered(cfg, d);
    if base < cfg.sea_level_m - 1.0 {
        return None;
    }
    let level = top_here(cfg, settings, &here, d, base);
    (level > 0).then_some(level)
}

/// The river's water surface over a far cell, metres over the sea, for the
/// far tiers that have no columns (design 13: the far river is drawn as the
/// sea is, a sheet over the cell): a river within half the cell's width of
/// its centre, `None` otherwise or at the sea.
pub fn far_surface(cfg: &TerrainConfig, d: Vec3, cell_m: f32) -> Option<f32> {
    let settings = cfg.rivers.as_ref()?;
    let map = map(cfg, settings);
    let here = map.at_widened(cfg, settings, d, (cell_m * 0.5 - settings.bank_m).max(0.0))?;
    if here.level <= 0 || here.across_m > (cell_m * 0.5).max(here.half_width_m) {
        return None;
    }
    let base = planet_gen::surface_altitude_unrivered(cfg, d);
    let top = top_here(cfg, settings, &here, d, base);
    (top > 0).then_some(cfg.sea_level_m + top as f32)
}

/// Whether a marsh pool may stand at `d` (version 8): the ground round it,
/// `rim_m` out each way, is not lower, so the pool is held.
pub fn rimmed(cfg: &TerrainConfig, d: Vec3, surface_m: f32) -> bool {
    let Some(settings) = cfg.rivers.as_ref() else {
        return true;
    };
    let d = d.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
    let north = d.cross(east);
    (0..6).all(|k| {
        let a = k as f32 * std::f32::consts::TAU / 6.0;
        let n =
            (d + (east * a.cos() + north * a.sin()) * (settings.rim_m / cfg.radius_m)).normalize();
        planet_gen::surface_altitude(cfg, n).floor() >= surface_m.floor()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v8() -> TerrainConfig {
        TerrainConfig::for_version(8).expect("version 8")
    }

    /// A river's course runs from a source no river feeds, down cells each
    /// draining into the next, to the sea; and it passes the cell it was
    /// asked about (`river-cache` design 8). The owner's river at 20.885
    /// -6.393 is 1.6 km long.
    #[test]
    fn a_course_runs_from_its_source_to_the_sea() {
        let cfg = TerrainConfig::for_version(crate::terrain::GENERATOR_VERSION).unwrap();
        let settings = cfg.rivers.expect("rivers");
        let map = map(&cfg, &settings);
        let d = crate::geo::direction(crate::geo::LatLon {
            lat: 20.885f32.to_radians(),
            lon: (-6.393f32).to_radians(),
        });
        let course = map.course(d, 2);
        assert!(course.len() > 10, "{} steps", course.len());
        let last = course.last().unwrap();
        assert!(!last.river, "the course ends in the sea");
        assert!(course[..course.len() - 1].iter().all(|s| s.river));
        assert!(
            course.windows(2).all(|w| w[1].level <= w[0].level),
            "its water never rises"
        );
        let metres: f32 = course
            .windows(2)
            .map(|w| w[0].centre.angle_between(w[1].centre) * cfg.radius_m)
            .sum();
        assert!((1_400.0..1_800.0).contains(&metres), "{metres} m");
        let nearest = course
            .iter()
            .map(|s| s.centre.angle_between(d) * cfg.radius_m)
            .fold(f32::INFINITY, f32::min);
        assert!(nearest < 60.0, "passes the spot: {nearest} m");
    }

    /// The drainage map built on many threads is the one-thread map, node for
    /// node (`river-cache` design 2): a core count never changes a river.
    #[test]
    fn the_map_built_on_every_core_is_the_one_thread_map() {
        let cfg = v8();
        let settings = cfg.rivers.expect("version 8 has rivers");
        let one = RiverMap::build_on(&cfg, &settings, 1);
        for threads in [3, 7] {
            assert!(
                RiverMap::build_on(&cfg, &settings, threads) == one,
                "the map on {threads} threads differs from one thread's"
            );
        }
        assert!(one.rivers().count() > 0);
    }

    /// Every river cell drains to the sea, down a path whose water never
    /// rises (design 13); and there are rivers to drain.
    #[test]
    fn every_river_cell_drains_to_the_sea() {
        let cfg = v8();
        let map = map(&cfg, cfg.rivers.as_ref().unwrap());
        let rivers: Vec<usize> = map.rivers().collect();
        assert!(rivers.len() > 500, "{} river cells", rivers.len());
        for &k in &rivers {
            let (levels, sea) = map.path_to_sea(k);
            assert!(sea, "river cell {k} does not reach the sea");
            assert!(
                levels.windows(2).all(|w| w[1] <= w[0]),
                "river cell {k}'s water rises on its way: {levels:?}"
            );
        }
        let falls = rivers.iter().filter(|&&k| map.nodes[k].drop >= 2).count();
        eprintln!(
            "{} river cells, {} below a fall of two layers or more",
            rivers.len(),
            falls
        );
    }

    /// At the finest scale: the column's water sits on its own carved bed,
    /// all along a river.
    #[test]
    fn version_10_s_river_beds_are_drawn_as_beds() {
        // `flora-v2` 12.10: wherever a river's water stands over the ground,
        // the column has water over its top and that top is drawn as a bed:
        // never a grass or snow, so no tuft, flower, mushroom or tree. And
        // `floods`, which the far records ask, is the column's own answer.
        let cfg = TerrainConfig::for_version(10).unwrap();
        let settings = cfg.rivers.unwrap();
        let map = map(&cfg, &settings);
        let (mut beds, mut grassy) = (0, 0);
        for k in map.rivers().step_by(53).take(300) {
            let Some(down) = map.down(k) else {
                continue;
            };
            let (a, b) = (map.centre(k), map.centre(down));
            for step in 0..=12 {
                let p = a.lerp(b, step as f32 / 12.0).normalize();
                if crate::settlement::ground::top(&cfg, p).is_some() {
                    continue;
                }
                let column = crate::column::generate_solid(&cfg, p);
                let top = column.surface().expect("bedrock");
                let flooded = floods(&cfg, p);
                if planet_gen::surface_altitude(&cfg, p) >= cfg.sea_level_m {
                    assert_eq!(flooded, !water(&cfg, p).is_empty(), "floods is water()");
                }
                if !flooded {
                    continue;
                }
                assert!(
                    column.water_over(top),
                    "a flooded column has water on its top"
                );
                let drawn = column.drawn_material(top);
                assert!(
                    !matches!(
                        drawn,
                        Material::Grass
                            | Material::DryGrass
                            | Material::JungleGrass
                            | Material::Snow
                    ),
                    "a bed drawn as {drawn:?}"
                );
                if matches!(
                    column.material(top),
                    Material::Grass | Material::DryGrass | Material::JungleGrass | Material::Snow
                ) {
                    grassy += 1;
                }
                beds += 1;
            }
        }
        // The case the owner saw is the common one: a meadow's top under the
        // river, drawn as dirt.
        assert!(
            beds > 300 && grassy > beds / 4,
            "{beds} beds, {grassy} grassy"
        );
    }

    #[test]
    fn a_rivers_water_sits_on_its_own_bed() {
        let cfg = v8();
        let settings = cfg.rivers.unwrap();
        let map = map(&cfg, &settings);
        let mut checked = 0;
        for k in map.rivers().step_by(97).take(200) {
            let Some(down) = map.down(k) else {
                continue;
            };
            let (a, b) = (map.centre(k), map.centre(down));
            for step in 0..=20 {
                let p = a.lerp(b, step as f32 / 20.0).normalize();
                if let Some(here) = map.at(&cfg, &settings, p)
                    && here.across_m < here.half_width_m
                    && here.level > 0
                {
                    let bed = planet_gen::surface_altitude(&cfg, p).floor() as i32;
                    if bed < 0 {
                        // At the mouth the sea's floor is the bed.
                        continue;
                    }
                    let water = water(&cfg, p);
                    if water.is_empty() {
                        // Where version 7's channel is under the sea there.
                        continue;
                    }
                    assert_eq!(water[0].0, bed + 1, "the first water layer sits on the bed");
                    checked += 1;
                }
            }
        }
        assert!(checked > 200, "{checked}");
    }
}
