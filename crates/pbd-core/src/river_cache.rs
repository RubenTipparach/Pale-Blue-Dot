//! The whole river cached (`river-cache` design 11).
//!
//! Every river value the generator derives is a pure function of the world's
//! config and a point: the drainage map, the river's ground (`carve`), the
//! water's top in a channel (`top_at`), the far sheet (`far_surface`), and
//! the flowing water the rule keeps (the tier's settle). This module holds
//! them once worked out, so a rebuild, a second rebuild over the same ground
//! and a second launch read them instead of working them out again.
//!
//! **Exact by construction.** A memo's key is its call's input, bit for bit,
//! and its value is what the call returned, so a hit is the number the call
//! would have made. Cells keep fixed directions whatever the anchor, so a
//! cell one rebuild asked about is the same key in the next.
//!
//! **The file** (`<saves>/cache/rivers-*.bin`) holds the map's nodes and the
//! memos. It is derived data, never a player record: a missing, short,
//! mismatched or corrupt file is rebuilt, and deleting it costs one rebuild.
//! `PBD_RIVER_CACHE=off` turns the memos and the file off, a pricing
//! instrument.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};

/// A small multiplicative hasher: the keys are float bits, already well mixed.
#[derive(Default)]
pub struct Fx(u64);

const K: u64 = 0x517c_c1b7_2722_0a95;

impl Hasher for Fx {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(b)).wrapping_mul(K);
        }
    }
    fn write_u32(&mut self, i: u32) {
        self.0 = (self.0.rotate_left(5) ^ u64::from(i)).wrapping_mul(K);
    }
    fn write_usize(&mut self, i: usize) {
        self.0 = (self.0.rotate_left(5) ^ i as u64).wrapping_mul(K);
    }
}

type FxMap<Key, V> = HashMap<Key, V, BuildHasherDefault<Fx>>;

const SHARDS: usize = 64;

thread_local! {
    static LOOKUPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Memo lookups made on the calling thread so far: an instrument, to find
/// which thread asks the cache how often.
pub fn thread_lookups() -> u64 {
    LOOKUPS.with(|n| n.get())
}

/// A memo shared by the threads that build a fine set: sharded by key, so
/// two threads rarely wait on one lock.
/// Read-write locks: once warm nearly every access is a read, and reads on
/// one shard from the job's threads and the main thread go side by side.
///
/// Each shard keeps two generations (`play-slowdown`): a value goes into
/// the young one, and when that reaches the shard's half of the cap the old
/// one is dropped and the young one becomes old. A read looks in both. Past
/// the cap the memo forgets its oldest values instead of keeping nothing
/// new: before, a memo filled by the first rivers played near missed on
/// every value elsewhere for the rest of the save's life (12 % of lookups
/// hit on the owner's World 19, 2026-11-28).
pub struct Memo<Key, V> {
    shards: Vec<RwLock<Gens<Key, V>>>,
    len: AtomicUsize,
    /// The most values held, about: each shard rolls its generations over
    /// at its share of half of it (`river-cache` design 15: the point memo
    /// reached 16 M values and 342 MB).
    cap: usize,
}

struct Gens<Key, V> {
    young: FxMap<Key, V>,
    old: FxMap<Key, V>,
}

impl<Key, V> Default for Gens<Key, V> {
    fn default() -> Self {
        Self {
            young: FxMap::default(),
            old: FxMap::default(),
        }
    }
}

impl<Key: Eq + Hash + Copy, V: Clone> Memo<Key, V> {
    fn new(cap: usize) -> Self {
        Self {
            shards: (0..SHARDS).map(|_| RwLock::new(Gens::default())).collect(),
            len: AtomicUsize::new(0),
            cap,
        }
    }

    fn shard(&self, key: &Key) -> &RwLock<Gens<Key, V>> {
        let mut h = Fx::default();
        key.hash(&mut h);
        &self.shards[(h.finish() >> 58) as usize % SHARDS]
    }

    /// Values a shard's young generation holds before it rolls over.
    fn young_cap(&self) -> usize {
        (self.cap / SHARDS / 2).max(1)
    }

    pub fn get(&self, key: &Key) -> Option<V> {
        LOOKUPS.with(|n| n.set(n.get() + 1));
        let shard = self.shard(key).read().ok()?;
        shard.young.get(key).or_else(|| shard.old.get(key)).cloned()
    }

    pub fn put(&self, key: Key, value: V) {
        // Only if the shard is free this instant: a memo is a cache, and a
        // value not kept is worked out again on the next ask. A blocking
        // write waited behind the file's save reading the shard, and readers
        // queued behind the waiting write (2026-11-23: the main thread held
        // 1.7 s, a save took 55 s).
        let Ok(mut shard) = self.shard(&key).try_write() else {
            return;
        };
        if shard.young.len() >= self.young_cap() {
            let dropped = std::mem::take(&mut shard.young);
            let gone = std::mem::replace(&mut shard.old, dropped);
            self.len.fetch_sub(gone.len(), Ordering::Relaxed);
        }
        let had_old = shard.old.contains_key(&key);
        if shard.young.insert(key, value).is_none() && !had_old {
            self.len.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn remove(&self, key: &Key) {
        if let Ok(mut shard) = self.shard(key).write() {
            let young = shard.young.remove(key).is_some();
            let old = shard.old.remove(key).is_some();
            let held = usize::from(young) + usize::from(old);
            if held > 0 {
                self.len.fetch_sub(held, Ordering::Relaxed);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every entry, in key order within each shard's turn: for the file. A
    /// key in both generations is written once, with its young value.
    fn entries(&self) -> Vec<(Key, V)> {
        let mut out = Vec::with_capacity(self.len());
        for shard in &self.shards {
            if let Ok(shard) = shard.read() {
                out.extend(shard.young.iter().map(|(k, v)| (*k, v.clone())));
                out.extend(
                    shard
                        .old
                        .iter()
                        .filter(|(k, _)| !shard.young.contains_key(*k))
                        .map(|(k, v)| (*k, v.clone())),
                );
            }
        }
        out
    }
}

/// The memos of one world's rivers.
pub struct RiverField {
    /// `carve(d, base)`: by `d`'s and `base`'s bits.
    pub(crate) carve: Memo<[u32; 4], f32>,
    /// `top_at(d)`: by `d`'s bits; `i32::MIN` for none.
    pub(crate) top: Memo<[u32; 3], i32>,
    /// `far_surface(d, cell_m)`: by `d`'s and `cell_m`'s bits; the surface's
    /// bits, `u32::MAX` for none.
    pub(crate) far: Memo<[u32; 4], u32>,
    /// The flowing water the rule keeps, by finest cell key: each layer the
    /// tier's settle changed and the liquid it holds after
    /// ([`crate::liquid::Cell`] as [`liquid_code`]). Kept only where the
    /// tier's rim could not reach (the app's `SETTLE_KEEP_CELLS`).
    pub settled: Memo<u32, Box<[(u16, u8)]>>,
    /// `planet_gen::swamp_pool_held(d, surface)`: by `d`'s and the
    /// surface's bits (`swamps`, "Swamp water is not cached the way rivers
    /// are"). Held for the session; not in the file yet.
    pub(crate) pool: Memo<[u32; 4], bool>,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    /// Entries in the file as last written or read.
    filed: AtomicUsize,
}

impl RiverField {
    pub(crate) fn new() -> Self {
        Self {
            carve: Memo::new(2_000_000),
            top: Memo::new(2_000_000),
            far: Memo::new(500_000),
            settled: Memo::new(500_000),
            pool: Memo::new(1_000_000),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            filed: AtomicUsize::new(0),
        }
    }

    /// How many values are held, of each kind: carve, top, far, settled.
    pub fn counts(&self) -> [usize; 4] {
        [
            self.carve.len(),
            self.top.len(),
            self.far.len(),
            self.settled.len(),
        ]
    }

    /// How many swamp pool answers are held.
    pub fn pools(&self) -> usize {
        self.pool.len()
    }

    fn total(&self) -> usize {
        self.counts().iter().sum::<usize>() + self.pool.len()
    }

    pub(crate) fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
}

/// Whether the memos and the file are on: `PBD_RIVER_CACHE=off` prices the
/// game without them.
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        !std::env::var("PBD_RIVER_CACHE").is_ok_and(|v| v.eq_ignore_ascii_case("off"))
    })
}

/// A liquid cell as one byte: 0 air, 1 a source, 2 + level * 2 + falling a
/// flowing cell.
pub fn liquid_code(cell: crate::liquid::Cell) -> Option<u8> {
    use crate::liquid::Cell;
    match cell {
        Cell::Neutral => None,
        Cell::Air => Some(0),
        Cell::Source => Some(1),
        Cell::Flowing { level, falling } => Some(2 + level * 2 + u8::from(falling)),
    }
}

/// [`liquid_code`] back.
pub fn liquid_of(code: u8) -> crate::liquid::Cell {
    use crate::liquid::Cell;
    match code {
        0 => Cell::Air,
        1 => Cell::Source,
        c => Cell::Flowing {
            level: (c - 2) / 2,
            falling: (c - 2) % 2 == 1,
        },
    }
}

// --- The file ---------------------------------------------------------------

static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Where the cache files are written (`<saves>/cache`), or `None` for none:
/// the app sets it at start, before any altitude is asked.
pub fn set_dir(dir: Option<PathBuf>) {
    if let Ok(mut d) = DIR.lock() {
        *d = dir;
    }
}

pub(crate) fn dir() -> Option<PathBuf> {
    if !enabled() {
        return None;
    }
    DIR.lock().ok()?.clone()
}

const MAGIC: &[u8; 8] = b"PBDRIVR1";
/// 2: the swamp pools' answers after the settled water (`swamps`, "Swamp
/// water is not cached the way rivers are"). A format-1 file is not read; it
/// is derived, and is made again.
const FORMAT: u32 = 2;

/// FNV-1a, for the config's digest and the file's.
pub fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The file of a world: by its seed and its whole config's digest, so a
/// world of another version, seed or setting never reads it.
pub(crate) fn path_for(dir: &Path, seed: u64, digest: u64) -> PathBuf {
    dir.join(format!("rivers-{seed:016x}-{digest:016x}.bin"))
}

/// The map's nodes as the file holds them.
pub(crate) struct FiledNode {
    pub down: u32,
    pub river: bool,
    pub level: i32,
    pub drop: i32,
    pub half_width: f32,
}

struct Out(Vec<u8>);

impl Out {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
}

struct In<'a>(&'a [u8]);

impl In<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        if self.0.len() < n {
            return None;
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Some(a)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
}

/// Write the map and the memos to `path`, atomically (a temporary file,
/// then a rename). Returns the entries written.
pub(crate) fn write(
    path: &Path,
    digest: u64,
    size: (u32, u32),
    nodes: &[FiledNode],
    field: &RiverField,
) -> std::io::Result<usize> {
    let mut o = Out(Vec::with_capacity(
        nodes.len() * 17 + field.total() * 20 + 64,
    ));
    o.0.extend_from_slice(MAGIC);
    o.u32(FORMAT);
    o.u64(digest);
    o.u32(size.0);
    o.u32(size.1);
    for n in nodes {
        o.u32(n.down);
        o.0.push(u8::from(n.river));
        o.u32(n.level as u32);
        o.u32(n.drop as u32);
        o.u32(n.half_width.to_bits());
    }
    // Only what a river changes is written: a point it leaves as it was is
    // cheap to work out again, and was most of the file.
    let carve: Vec<_> = field
        .carve
        .entries()
        .into_iter()
        .filter(|(k, v)| v.to_bits() != k[3])
        .collect();
    o.u64(carve.len() as u64);
    for (k, v) in &carve {
        k.iter().for_each(|&x| o.u32(x));
        o.u32(v.to_bits());
    }
    let top: Vec<_> = field
        .top
        .entries()
        .into_iter()
        .filter(|(_, v)| *v != i32::MIN)
        .collect();
    o.u64(top.len() as u64);
    for (k, v) in &top {
        k.iter().for_each(|&x| o.u32(x));
        o.u32(*v as u32);
    }
    let far: Vec<_> = field
        .far
        .entries()
        .into_iter()
        .filter(|(_, v)| *v != u32::MAX)
        .collect();
    o.u64(far.len() as u64);
    for (k, v) in &far {
        k.iter().for_each(|&x| o.u32(x));
        o.u32(*v);
    }
    let settled = field.settled.entries();
    o.u64(settled.len() as u64);
    for (k, v) in &settled {
        o.u32(*k);
        o.u32(v.len() as u32);
        for &(layer, code) in v.iter() {
            o.0.extend_from_slice(&layer.to_le_bytes());
            o.0.push(code);
        }
    }
    let pool = field.pool.entries();
    o.u64(pool.len() as u64);
    for (k, v) in &pool {
        k.iter().for_each(|&x| o.u32(x));
        o.0.push(u8::from(*v));
    }
    let sum = fnv(&o.0);
    o.u64(sum);
    let entries = carve.len() + top.len() + far.len() + settled.len() + pool.len();
    let held = field.total();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension(format!("tmp{}", std::process::id()));
    {
        let mut f = std::fs::File::create(&temp)?;
        f.write_all(&o.0)?;
        f.sync_all()?;
    }
    std::fs::rename(&temp, path)?;
    clear_stale_temps(path);
    field.filed.store(held, Ordering::Relaxed);
    Ok(entries)
}

/// Remove temporary files beside `path` that a process stopped mid-write
/// left behind (`<name>.tmp<pid>`): a cache file can be hundreds of MB.
pub fn clear_stale_temps(path: &Path) {
    let (Some(dir), Some(stem)) = (path.parent(), path.file_stem().and_then(|s| s.to_str())) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(&format!("{stem}.tmp")) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Read `path` into `field`, returning the map's size and nodes; `None` for
/// a missing, short, mismatched or corrupt file, which leaves `field` as it
/// was.
pub(crate) fn read(
    path: &Path,
    digest: u64,
    field: &RiverField,
) -> Option<((u32, u32), Vec<FiledNode>)> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() < 8 + 4 + 8 + 8 + 8 {
        return None;
    }
    let (body, sum) = bytes.split_at(bytes.len() - 8);
    if fnv(body) != u64::from_le_bytes(sum.try_into().ok()?) {
        return None;
    }
    let mut i = In(body);
    if i.take(8)? != MAGIC || i.u32()? != FORMAT || i.u64()? != digest {
        return None;
    }
    let size = (i.u32()?, i.u32()?);
    let count = size.0 as usize * size.1 as usize;
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        nodes.push(FiledNode {
            down: i.u32()?,
            river: i.u8()? != 0,
            level: i.u32()? as i32,
            drop: i.u32()? as i32,
            half_width: f32::from_bits(i.u32()?),
        });
    }
    let mut carve = Vec::new();
    for _ in 0..i.u64()? {
        carve.push((
            [i.u32()?, i.u32()?, i.u32()?, i.u32()?],
            f32::from_bits(i.u32()?),
        ));
    }
    let mut top = Vec::new();
    for _ in 0..i.u64()? {
        top.push(([i.u32()?, i.u32()?, i.u32()?], i.u32()? as i32));
    }
    let mut far = Vec::new();
    for _ in 0..i.u64()? {
        far.push(([i.u32()?, i.u32()?, i.u32()?, i.u32()?], i.u32()?));
    }
    let mut settled = Vec::new();
    for _ in 0..i.u64()? {
        let key = i.u32()?;
        let n = i.u32()? as usize;
        let mut cells = Vec::with_capacity(n.min(64));
        for _ in 0..n {
            cells.push((i.u16()?, i.u8()?));
        }
        settled.push((key, cells.into_boxed_slice()));
    }
    let mut pool = Vec::new();
    for _ in 0..i.u64()? {
        pool.push(([i.u32()?, i.u32()?, i.u32()?, i.u32()?], i.u8()? != 0));
    }
    if !i.0.is_empty() {
        return None;
    }
    let entries = carve.len() + top.len() + far.len() + settled.len() + pool.len();
    pool.into_iter().for_each(|(k, v)| field.pool.put(k, v));
    carve.into_iter().for_each(|(k, v)| field.carve.put(k, v));
    top.into_iter().for_each(|(k, v)| field.top.put(k, v));
    far.into_iter().for_each(|(k, v)| field.far.put(k, v));
    settled
        .into_iter()
        .for_each(|(k, v)| field.settled.put(k, v));
    field.filed.store(entries, Ordering::Relaxed);
    Some((size, nodes))
}

/// Whether the field has grown since it was last written or read.
/// Whether the memos hold `share` more than the file does (0.05: five in a
/// hundred more), or the file holds nothing yet: what a save while playing
/// waits for, so a file of millions of values is not written whole for a few
/// new ones (2026-11-23: 50 MB every 30 s, saves of 11 to 30 s).
pub(crate) fn grown_by(field: &RiverField, share: f64) -> bool {
    let total = field.total();
    let filed = field.filed.load(Ordering::Relaxed);
    total != filed && (filed == 0 || total as f64 >= filed as f64 * (1.0 + share))
}

#[cfg(test)]
mod memo_tests {
    use super::*;

    /// Past its cap the memo keeps what was put last (`play-slowdown`):
    /// before, it kept nothing new once full, so every later key missed.
    #[test]
    fn a_full_memo_keeps_the_newest_values() {
        let memo: Memo<u32, u32> = Memo::new(SHARDS * 2 * 8);
        for k in 0..100_000u32 {
            memo.put(k, k * 3);
        }
        for k in 99_900..100_000u32 {
            assert_eq!(memo.get(&k), Some(k * 3), "the newest key {k} is held");
        }
        assert!(memo.get(&0).is_none(), "the oldest key was forgotten");
        assert!(
            memo.len() <= SHARDS * 2 * 8,
            "held {} past the cap",
            memo.len()
        );
    }

    /// A value is the same whichever generation holds it, and a key is
    /// counted once.
    #[test]
    fn a_key_in_both_generations_counts_once() {
        let memo: Memo<u32, u32> = Memo::new(SHARDS * 2 * 4);
        memo.put(7, 70);
        memo.put(7, 70);
        assert_eq!(memo.len(), 1);
        assert_eq!(memo.get(&7), Some(70));
        memo.remove(&7);
        assert_eq!(memo.len(), 0);
        assert!(memo.get(&7).is_none());
        assert_eq!(memo.entries().len(), 0);
    }
}
