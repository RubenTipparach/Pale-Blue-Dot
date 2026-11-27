//! The river band (`river-cache` design 14): a river's final water geometry
//! loaded the way terrain is, out to the finest band.
//!
//! The owner: "the full final water geometry should be loading out to a
//! reasonable distance that once the player crosses a threshold another job
//! starts loading water again. exactly the same process as terrain", "this
//! data should be cached somewhere so it is easily retreivable", "since we
//! know water WILL NOT change on its own", and "water rebuild is only when a
//! player influences the flow of the river".
//!
//! - **Built in the fine-set job** ([`build`], called by
//!   `planet::lod::generate_fine_live_on` after the column tier), over every
//!   finest cell a river's water may reach, and landing with its ground.
//! - **The real water**: each cell's column, its river sources and the rule's
//!   settled flow (`ColumnTier::settle_rivers`, stamped from the river cache),
//!   meshed by the same mesher as the tier's water (`water_flow::liquid_mesh`).
//! - **Cached by cell**: a cell's water pieces are planet-centred and, away
//!   from the rim of the band's own tier, a pure function of the world and the
//!   player's edits. So they are kept by cell key ([`PIECES`]), in memory and
//!   in a file beside the worlds, built once and read by every later job and
//!   launch. A job builds only the cells nobody has built yet. An edit drops
//!   the cells it reaches ([`drop_round`]).
//! - **The column tier's own cells** (not its rim) are drawn by the tier,
//!   which runs the live rule; the band draws everything else.

use bevy::math::Vec3;
use bevy::prelude::{debug, info};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};

use crate::config::ColumnSettings;
use crate::planet::column::{self, ColumnTier, Members};
use crate::planet::{GpuCell, PLANET_RADIUS, terrain_config};
use crate::water_flow::{FlowVertex, LiquidMesh, liquid_mesh};
use pbd_core::edits::Edits;

/// How far a cell's water is kept from the rim of the tier it was meshed in,
/// in cells: the rule's settle near a rim depends on the rim, out to its
/// seven-cell reach and a fall's run below it, so a cell's water is its own
/// only as far in as the settle's own keep (`SETTLE_KEEP_CELLS`), and a
/// piece's faces read its neighbours one cell further.
const KEEP_FROM_RIM: u16 = column::SETTLE_KEEP_CELLS as u16 + 1;
/// How many rings round the cells to build a tier is grown by, so every cell
/// built is at least [`KEEP_FROM_RIM`] from its rim.
const GROW_RINGS: usize = KEEP_FROM_RIM as usize + 1;
/// The column tier draws its water only this far in from its rim; the band
/// draws the ring outside it (`water_flow::liquid_mesh_from`).
pub const TIER_KEEP: u16 = KEEP_FROM_RIM;
/// How far past a river's banks its band reaches, m: the levee and a cell.
const BANK_REACH_M: f32 = 6.0;
/// How far round an edit near a river its cached water is built again, m:
/// the rule's seven-cell reach and a fall's run below it.
pub const EDIT_REACH_M: f32 = 40.0;
/// The most cells a band's tier is built over at once.
const BAND_CAPACITY: usize = 40_000;

/// One cell's water: its pieces' vertices, still (never eased), in the order
/// the mesher made them. Empty for a cell the river leaves dry.
type CellWater = std::sync::Arc<[FlowVertex]>;

/// Every cell's water built so far, by cell key, for the world it was built
/// in (its seed and generator version).
struct Pieces {
    world: (u64, u32),
    cells: HashMap<u32, CellWater>,
    /// Cells added since the file was last written.
    grown: usize,
}

static PIECES: OnceLock<RwLock<Pieces>> = OnceLock::new();
static LOADED: AtomicBool = AtomicBool::new(false);
static SAVING: Mutex<()> = Mutex::new(());
pub static HITS: AtomicUsize = AtomicUsize::new(0);
pub static BUILT: AtomicUsize = AtomicUsize::new(0);

fn world() -> (u64, u32) {
    (terrain_config().seed, crate::planet::generator_version())
}

fn pieces() -> &'static RwLock<Pieces> {
    let lock = PIECES.get_or_init(|| {
        RwLock::new(Pieces {
            world: world(),
            cells: HashMap::new(),
            grown: 0,
        })
    });
    // Another world opened in this process: its rivers are not these.
    if lock.read().map(|p| p.world != world()).unwrap_or(false)
        && let Ok(mut p) = lock.write()
    {
        p.world = world();
        p.cells.clear();
        p.grown = 0;
        LOADED.store(false, Ordering::Relaxed);
    }
    if !LOADED.swap(true, Ordering::Relaxed)
        && let Some(path) = file()
        && let Some(cells) = read(&path)
        && let Ok(mut p) = lock.write()
    {
        let n = cells.len();
        p.cells.extend(cells);
        info!("RIVER_BAND read {n} cells' water from {}", path.display());
    }
    lock
}

/// The band's water for a fine set, and what building it cost.
#[derive(Default)]
pub struct RiverBand {
    /// The water the band draws, still, planet-centred.
    pub mesh: LiquidMesh,
    /// Each drawn cell's water top, m over the radius, by cell key: what the
    /// eye and the walker read off the tier, where no column says where the
    /// water is.
    pub tops: HashMap<u32, f32>,
    /// Cells drawn, read from the cache, and built by this job.
    pub cells: usize,
    pub read: usize,
    pub built: usize,
    pub ms: f64,
}

/// Build the band for a fine set: every finest cell a river's water may
/// reach, less the column tier's own cells (which the tier draws), read from
/// the cache or built. `finest` and `neighbors` are the set's finest records
/// and their neighbour table, after the column tier stamped its slots.
pub fn build(
    anchor: Vec3,
    finest: &[GpuCell],
    neighbors: &[[u32; 6]],
    tier: &ColumnTier,
    settings: &ColumnSettings,
    edits: &Edits,
) -> RiverBand {
    let started = std::time::Instant::now();
    let cfg = terrain_config();
    if cfg.rivers.is_none()
        || finest.is_empty()
        || std::env::var("PBD_RIVER_BAND").is_ok_and(|v| v == "off")
    {
        return RiverBand::default();
    }
    let dir = |i: usize| Vec3::from_slice(&finest[i].direction_height[..3]);
    // The river's cells: within its banks' reach.
    let river: Vec<bool> = (0..finest.len())
        .map(|i| pbd_core::rivers::near(cfg, dir(i), BANK_REACH_M))
        .collect();
    // The tier's deep cells: the tier draws those, with its live rule.
    let tier_depth = tier.rim_depth(TIER_KEEP);
    let tier_draws = |i: usize| {
        column::slot_of(&finest[i])
            .is_some_and(|s| tier_depth.get(s as usize).is_some_and(|&d| d >= TIER_KEEP))
    };
    let wanted: Vec<usize> = (0..finest.len())
        .filter(|&i| river[i] && !tier_draws(i))
        .collect();
    // The cache holds the generated river only, never an edited one: it is
    // keyed by the world's seed and version, which every save of that seed
    // shares. So a cell within an edit's reach is never read from it nor
    // kept in it; it is built with this save's edits every time.
    let edited: Vec<Vec3> = (0..finest.len())
        .filter(|&i| !edits.for_cell(finest[i].key()).is_empty())
        .map(dir)
        .collect();
    let reach = (EDIT_REACH_M / PLANET_RADIUS).cos();
    let touched = |i: usize| {
        let d = dir(i);
        edited.iter().any(|e| e.dot(d) >= reach)
    };
    let mut mesh = LiquidMesh::default();
    let mut tops = HashMap::new();
    let mut read = 0;
    let mut missing = Vec::new();
    {
        let cache = pieces().read().expect("band pieces");
        for &i in &wanted {
            let cached = if touched(i) || bypass() {
                None
            } else {
                cache.cells.get(&finest[i].key())
            };
            match cached {
                Some(water) => {
                    push(&mut mesh, &mut tops, finest[i].key(), water);
                    read += 1;
                }
                None => missing.push(i),
            }
        }
    }
    let mut built = 0;
    if !missing.is_empty() {
        // The cells to build, grown by a few rings so each is far enough from
        // the tier's rim to be meshed against its real neighbours.
        let mut member = vec![false; finest.len()];
        let mut front: VecDeque<(usize, usize)> = missing.iter().map(|&i| (i, 0)).collect();
        for &i in &missing {
            member[i] = true;
        }
        while let Some((i, ring)) = front.pop_front() {
            if ring >= GROW_RINGS {
                continue;
            }
            for &n in &neighbors[i] {
                let n = n as usize;
                if n < finest.len() && !member[n] {
                    member[n] = true;
                    front.push_back((n, ring + 1));
                }
            }
        }
        let mut scratch = finest.to_vec();
        let band = column::build_with(
            anchor,
            &mut scratch,
            neighbors,
            settings,
            edits,
            &Members {
                worms_m: 0.0,
                select: &|i, _| member[i],
                capacity: BAND_CAPACITY,
                light: true,
            },
        );
        let made = liquid_mesh(&band, &scratch, PLANET_RADIUS);
        // Each built cell's distance from the band tier's rim, in cells.
        let depth = band.rim_depth(KEEP_FROM_RIM);
        let mut by_key: HashMap<u32, Vec<FlowVertex>> = HashMap::new();
        for piece in &made.pieces {
            let key = (piece.ident >> 32) as u32;
            let at = piece.first as usize;
            by_key
                .entry(key)
                .or_default()
                .extend_from_slice(&made.vertices[at..at + piece.n as usize]);
        }
        let wanted_now: HashSet<usize> = missing.iter().copied().collect();
        let mut keep = Vec::new();
        for (slot, &index) in band.water.records.iter().enumerate() {
            if !wanted_now.contains(&index) {
                continue;
            }
            let key = finest[index].key();
            let water: CellWater = by_key.remove(&key).unwrap_or_default().into();
            push(&mut mesh, &mut tops, key, &water);
            built += 1;
            // Kept only where its neighbours were all real: deep enough in.
            if depth.get(slot).is_some_and(|&d| d >= KEEP_FROM_RIM)
                && key != crate::planet::NO_KEY
                && !touched(index)
            {
                keep.push((key, water));
            }
        }
        if !bypass()
            && let Ok(mut cache) = pieces().write()
        {
            cache.grown += keep.len();
            cache.cells.extend(keep);
        }
    }
    HITS.fetch_add(read, Ordering::Relaxed);
    BUILT.fetch_add(built, Ordering::Relaxed);
    let ms = started.elapsed().as_secs_f64() * 1e3;
    debug!(
        "river band: {} cells, {read} read, {built} built, {} vertices, {ms:.1} ms",
        wanted.len(),
        mesh.vertices.len()
    );
    RiverBand {
        mesh,
        tops,
        cells: wanted.len(),
        read,
        built,
        ms,
    }
}

fn push(mesh: &mut LiquidMesh, tops: &mut HashMap<u32, f32>, key: u32, water: &[FlowVertex]) {
    mesh.vertices.extend_from_slice(water);
    if !water.is_empty() {
        mesh.cells += 1;
        // The highest point of its faces is its water's top.
        let top = water
            .iter()
            .map(|v| v.position.length() - PLANET_RADIUS)
            .fold(f32::NEG_INFINITY, f32::max);
        tops.insert(key, top);
    }
}

/// Drop the cached water of every cell within `metres` of `at` (a unit
/// direction): what an edit to a river's flow does, so the next job builds
/// those cells again with the edit in them (owner: "water rebuild is only
/// when a player influences the flow of the river").
pub fn drop_round(at: Vec3, metres: f32, keys: impl IntoIterator<Item = (u32, Vec3)>) {
    let cos = (metres / PLANET_RADIUS).cos();
    let at = at.normalize_or(Vec3::Y);
    if let Ok(mut cache) = pieces().write() {
        let mut dropped = 0;
        for (key, d) in keys {
            if d.normalize_or(Vec3::Y).dot(at) >= cos && cache.cells.remove(&key).is_some() {
                dropped += 1;
            }
        }
        if dropped > 0 {
            cache.grown += 1;
        }
    }
}

// --- The file ----------------------------------------------------------------

/// The file's mark, and its version: 2 since a full river's top is drawn
/// under its bank (`townsfolk` design 42p), so a file meshed before is read
/// as none and meshed again.
const MAGIC: &[u8; 8] = b"PBDBAND2";

fn file() -> Option<PathBuf> {
    if !pbd_core::river_cache::enabled() {
        return None;
    }
    let (seed, version) = world();
    Some(
        crate::paths::saves_dir()
            .join("cache")
            .join(format!("river-band-{seed:016x}-g{version}.bin")),
    )
}

fn f(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Write the cached cells to the file if any were added since it was last
/// written: atomic (a temporary file, then a rename), off the frame. Returns
/// the cells written.
pub fn save() -> usize {
    let Some(path) = file() else {
        return 0;
    };
    let _one = SAVING.lock();
    let cells: Vec<(u32, CellWater)> = {
        let Ok(mut cache) = pieces().write() else {
            return 0;
        };
        if cache.grown == 0 {
            return 0;
        }
        cache.grown = 0;
        cache.cells.iter().map(|(k, v)| (*k, v.clone())).collect()
    };
    let mut out = Vec::with_capacity(cells.len() * 64);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(cells.len() as u64).to_le_bytes());
    for (key, water) in &cells {
        out.extend_from_slice(&key.to_le_bytes());
        out.extend_from_slice(&(water.len() as u32).to_le_bytes());
        for v in water.iter() {
            for x in [v.position, v.normal, v.flow] {
                f(&mut out, x.x);
                f(&mut out, x.y);
                f(&mut out, x.z);
            }
            f(&mut out, v.kind);
            f(&mut out, v.sky);
        }
    }
    let sum = pbd_core::river_cache::fnv(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    let write = || -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension(format!("tmp{}", std::process::id()));
        std::fs::write(&temp, &out)?;
        std::fs::rename(&temp, &path)?;
        pbd_core::river_cache::clear_stale_temps(&path);
        Ok(())
    };
    match write() {
        Ok(()) => cells.len(),
        Err(e) => {
            info!("RIVER_BAND could not be written: {e}");
            0
        }
    }
}

fn read(path: &PathBuf) -> Option<Vec<(u32, CellWater)>> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 24 {
        return None;
    }
    let (body, sum) = bytes.split_at(bytes.len() - 8);
    if pbd_core::river_cache::fnv(body) != u64::from_le_bytes(sum.try_into().ok()?) {
        return None;
    }
    if &body[..8] != MAGIC {
        return None;
    }
    let mut at = 8;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = body.get(at..at + n)?;
        at += n;
        Some(s)
    };
    let u32_ = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap());
    let f32_ = |b: &[u8]| f32::from_le_bytes(b.try_into().unwrap());
    let count = u64::from_le_bytes(take(8)?.try_into().ok()?) as usize;
    let mut cells = Vec::with_capacity(count.min(1 << 22));
    for _ in 0..count {
        let key = u32_(take(4)?);
        let n = u32_(take(4)?) as usize;
        let mut water = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            let b = take(44)?;
            let v = |k: usize| {
                Vec3::new(
                    f32_(&b[k * 4..k * 4 + 4]),
                    f32_(&b[k * 4 + 4..k * 4 + 8]),
                    f32_(&b[k * 4 + 8..k * 4 + 12]),
                )
            };
            let (position, normal, flow) = (v(0), v(3), v(6));
            let (kind, sky) = (f32_(&b[36..40]), f32_(&b[40..44]));
            water.push(FlowVertex::still(position, normal, flow, kind, sky));
        }
        cells.push((key, water.into()));
    }
    Some(cells)
}

#[cfg(test)]
thread_local! {
    /// A test builds on this thread as if the cache were empty, and keeps
    /// nothing in it: other tests share the cache and fill it meanwhile.
    static BYPASS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn bypass() -> bool {
    #[cfg(test)]
    {
        BYPASS.with(|b| b.get())
    }
    #[cfg(not(test))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::lod;

    /// A river with a fall, as the drainage map has it.
    fn a_fall() -> Vec3 {
        let cfg = terrain_config();
        let settings = cfg.rivers.expect("rivers");
        let map = pbd_core::rivers::map(cfg, &settings);
        map.rivers()
            .map(|k| map.describe(k))
            .find(|&(c, level, drop, _)| drop >= 3 && level >= 3 && c.y.abs() < 0.7)
            .map(|(c, ..)| c)
            .expect("a river with a fall")
    }

    fn sorted(mesh: &LiquidMesh) -> Vec<[u32; 11]> {
        let mut out: Vec<[u32; 11]> = mesh
            .vertices
            .iter()
            .map(|v| {
                let p = v.position.to_array().map(f32::to_bits);
                let n = v.normal.to_array().map(f32::to_bits);
                let f = v.flow.to_array().map(f32::to_bits);
                [
                    p[0],
                    p[1],
                    p[2],
                    n[0],
                    n[1],
                    n[2],
                    f[0],
                    f[1],
                    f[2],
                    v.kind.to_bits(),
                    v.sky.to_bits(),
                ]
            })
            .collect();
        out.sort_unstable();
        out
    }

    /// The river band's water read from the cache is the water built fresh
    /// (`river-cache` design 14): a set 40 m on from one already built reads
    /// most of its cells, and its water is the same as the same set's built
    /// with the cache empty, vertex for vertex.
    #[test]
    fn a_band_read_from_the_cache_is_the_band_built_fresh() {
        if terrain_config().rivers.is_none() {
            return;
        }
        let anchor = a_fall();
        let east = Vec3::Y.cross(anchor).normalize_or(Vec3::X);
        let on = (anchor + east * (40.0 / PLANET_RADIUS)).normalize();
        let settings = ColumnSettings::default();
        let first = lod::generate_fine(anchor, &settings, &Edits::new());
        assert!(
            first.river.cells > 100,
            "a river band: {} cells",
            first.river.cells
        );
        let warm = lod::generate_fine(on, &settings, &Edits::new());
        assert!(
            warm.river.read > warm.river.cells / 2,
            "read {} of {}",
            warm.river.read,
            warm.river.cells
        );
        BYPASS.with(|b| b.set(true));
        let fresh = lod::generate_fine(on, &settings, &Edits::new());
        BYPASS.with(|b| b.set(false));
        assert_eq!(fresh.river.read, 0);
        assert_eq!(warm.river.cells, fresh.river.cells);
        let (a, b) = (sorted(&warm.river.mesh), sorted(&fresh.river.mesh));
        assert!(!a.is_empty(), "the band draws water");
        assert_eq!(
            a.len(),
            b.len(),
            "vertices: read {} against fresh {}",
            a.len(),
            b.len()
        );
        let differ = a.iter().zip(&b).filter(|(x, y)| x != y).count();
        assert_eq!(differ, 0, "{differ} vertices differ");
    }
}

#[cfg(test)]
mod probe_reach {
    use super::*;
    use crate::planet::lod;

    /// INSTRUMENT (ignored): from one anchor at the owner's river's source,
    /// the band's water along the river: for points every 25 m down its
    /// line, whether the cell floods, is wanted, is in the tier, and has band
    /// vertices over it and at what height against its water's top.
    #[test]
    #[ignore]
    fn probe_band_reach_down_the_river() {
        let cfg = terrain_config();
        let src = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 19.25f32.to_radians(),
            lon: (-6.75f32).to_radians(),
        });
        let path = crate::river_walk::RiverPath::through(src, 1.0).expect("river");
        let anchor = path.line_at(60.0);
        let set = lod::generate_fine(anchor, &ColumnSettings::default(), &Edits::new());
        let verts: Vec<Vec3> = set.river.mesh.vertices.iter().map(|v| v.position).collect();
        eprintln!("band {} cells, {} vertices", set.river.cells, verts.len());
        let mut m = 60.0;
        while m < 420.0 {
            let d = path.line_at(m);
            let from = d.angle_between(anchor) * PLANET_RADIUS;
            let near: Vec<&Vec3> = verts
                .iter()
                .filter(|v| v.normalize().angle_between(d) * PLANET_RADIUS < 3.0)
                .collect();
            let top = near
                .iter()
                .map(|v| v.length() - PLANET_RADIUS)
                .fold(f32::NEG_INFINITY, f32::max);
            let rec = set.finest_records().iter().min_by(|a, b| {
                let da = Vec3::from_slice(&a.direction_height[..3]).angle_between(d);
                let db = Vec3::from_slice(&b.direction_height[..3]).angle_between(d);
                da.total_cmp(&db)
            });
            let slot = rec.and_then(column::slot_of);
            eprintln!(
                "{m:4.0} m down ({from:3.0} m from the anchor): floods {} water {:?} tier slot {:?} | {} band vertices within 3 m, top {:.2} m | record height {:.2}",
                pbd_core::rivers::floods(cfg, d),
                pbd_core::rivers::water(cfg, d)
                    .last()
                    .map(|w| w.0 as f32 - cfg.sea_level_m),
                slot,
                near.len(),
                top,
                rec.map_or(f32::NAN, |r| r.direction_height[3])
            );
            m += 25.0;
        }
    }
}
