//! A region's canonical encoding and its digest (`multiplayer` design 2 and
//! 16, task 3.1).
//!
//! The server sends each region's digest; a client digests its own copy and
//! asks for the region when the two differ. So the encoding is fixed byte
//! for byte, little-endian with no padding, and the hash is FNV-1a 64, never
//! `std`'s hasher, whose seed is random per process.
//!
//! ```text
//! u8   encoding version (1)
//! u32  region address (`cell_key::region`)
//! then each section that has content, in tag order:
//!   u8   tag
//!   u32  entry count
//!   ...  entries
//! tag 1, edits: per cell by place ascending
//!   u16  place
//!   u16  layer count
//!   per layer, in the order `Edits` keeps them: u16 layer, u8 material code
//! ```
//!
//! An empty section is left out, so a region with no liquids or records
//! digests the same before and after those sections are added (tags 2 and
//! 3, design 16). Derived data (columns, light, GPU records) is never
//! digested: it is a pure function of this.

use crate::cell_key::region;
use crate::edits::Edits;
use crate::terrain::{Material, material_code};
use std::collections::{BTreeMap, HashMap};

/// The encoding's version, its first byte.
pub const ENCODING_VERSION: u8 = 1;
/// The edits section's tag.
pub const EDITS_TAG: u8 = 1;

/// FNV-1a over 64 bits.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes
        .iter()
        .fold(OFFSET, |h, &b| (h ^ u64::from(b)).wrapping_mul(PRIME))
}

/// The canonical bytes of a region whose edited cells are `cells`, each
/// `(place, its layers in order)`, in any order (they are sorted here).
pub fn encode(region: u32, cells: &[(u16, &[(u16, Material)])]) -> Vec<u8> {
    let mut sorted: Vec<&(u16, &[(u16, Material)])> = cells
        .iter()
        .filter(|(_, layers)| !layers.is_empty())
        .collect();
    sorted.sort_unstable_by_key(|(place, _)| *place);
    let mut out = Vec::with_capacity(5 + 7 + sorted.len() * 8);
    out.push(ENCODING_VERSION);
    out.extend_from_slice(&region.to_le_bytes());
    if !sorted.is_empty() {
        out.push(EDITS_TAG);
        out.extend_from_slice(&(sorted.len() as u32).to_le_bytes());
        for (place, layers) in sorted {
            out.extend_from_slice(&place.to_le_bytes());
            out.extend_from_slice(&(layers.len() as u16).to_le_bytes());
            for &(layer, material) in layers.iter() {
                out.extend_from_slice(&layer.to_le_bytes());
                out.push(material_code(material));
            }
        }
    }
    out
}

/// One edited cell of a region: its place, and its layers in order.
pub type Cell = (u16, Vec<(u16, Material)>);

/// A region's state read back from its canonical bytes: its address and
/// each edited cell's place and layers, by place. `None` for bytes that are
/// not one region's encoding of this version (a section this build does not
/// know is refused, not skipped: a client that cannot read a region must
/// not pretend to hold it).
pub fn decode(bytes: &[u8]) -> Option<(u32, Vec<Cell>)> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Option<&[u8]> {
        let out = bytes.get(at..at + n)?;
        at += n;
        Some(out)
    };
    if take(1)? != [ENCODING_VERSION] {
        return None;
    }
    let region = u32::from_le_bytes(take(4)?.try_into().ok()?);
    let mut cells = Vec::new();
    if let Some(tag) = take(1) {
        if tag != [EDITS_TAG] {
            return None;
        }
        let count = u32::from_le_bytes(take(4)?.try_into().ok()?) as usize;
        let mut last: Option<u16> = None;
        for _ in 0..count {
            let place = u16::from_le_bytes(take(2)?.try_into().ok()?);
            // Ascending and unique, as encoded: anything else is not ours.
            if last.is_some_and(|l| l >= place) {
                return None;
            }
            last = Some(place);
            let n = u16::from_le_bytes(take(2)?.try_into().ok()?) as usize;
            if n == 0 {
                return None;
            }
            let mut layers = Vec::with_capacity(n.min(crate::column::LAYERS));
            for _ in 0..n {
                let layer = u16::from_le_bytes(take(2)?.try_into().ok()?);
                let material = crate::terrain::material_of(take(1)?[0])?;
                layers.push((layer, material));
            }
            cells.push((place, layers));
        }
        if take(1).is_some() {
            return None;
        }
    }
    Some((region, cells))
}

/// The edits a region's snapshot holds, as `Edit`s with their cell keys,
/// in place order: what a client folds into its copy when it replaces the
/// region (`multiplayer` design 2, 3).
pub fn snapshot_edits(bytes: &[u8]) -> Option<(u32, Vec<crate::edits::Edit>)> {
    let (r, cells) = decode(bytes)?;
    let mut out = Vec::new();
    for (place, layers) in cells {
        let a = region::address(r, place)?;
        let cell = crate::cell_key::key(a)?;
        for (layer, material) in layers {
            out.push(crate::edits::Edit {
                cell,
                layer,
                material,
            });
        }
    }
    Some((r, out))
}

/// Every region's edited cells and its digest, kept as edits are made, so a
/// digest check is a lookup (design 2).
///
/// Keyed lookups only: what is iterated, a region's cells, is a `BTreeMap`
/// by place, and the list of regions is sorted before it is handed out.
#[derive(Clone, Debug, Default)]
pub struct Regions {
    /// Per region, its edited cells by place, each to its cell key.
    cells: HashMap<u32, BTreeMap<u16, u32>>,
    /// Per region, the digest of its encoding now.
    digests: HashMap<u32, u64>,
}

impl Regions {
    /// The regions of every edit in `edits`.
    pub fn of(edits: &Edits) -> Self {
        let mut regions = Self::default();
        let mut touched: Vec<u32> = Vec::new();
        for edit in edits.all() {
            if let Some(r) = regions.file(edit.cell) {
                touched.push(r);
            }
        }
        touched.sort_unstable();
        touched.dedup();
        for r in touched {
            regions.refresh(r, edits);
        }
        regions
    }

    /// Note that the cell `key` was just changed in `edits`, and bring its
    /// region's digest up to date. Returns the region, or `None` for a key no
    /// region holds (which no edit has).
    pub fn note(&mut self, key: u32, edits: &Edits) -> Option<u32> {
        let r = self.file(key)?;
        self.refresh(r, edits);
        Some(r)
    }

    /// The digest of `region` now: of its empty encoding if nothing in it
    /// has been edited.
    pub fn digest(&self, region: u32) -> u64 {
        self.digests
            .get(&region)
            .copied()
            .unwrap_or_else(|| fnv1a64(&encode(region, &[])))
    }

    /// The canonical bytes of `region` from `edits`: what a snapshot sends.
    pub fn encode(&self, region: u32, edits: &Edits) -> Vec<u8> {
        let cells: Vec<(u16, &[(u16, Material)])> = self
            .cells
            .get(&region)
            .map(|cells| {
                cells
                    .iter()
                    .map(|(&place, &key)| (place, edits.for_cell(key)))
                    .collect()
            })
            .unwrap_or_default();
        encode(region, &cells)
    }

    /// Every region with an edit, ascending.
    pub fn regions(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.cells.keys().copied().collect();
        out.sort_unstable();
        out
    }

    /// Forget `region` and file again only the cells of `keys`, then refresh
    /// its digest from `edits`: a region replaced whole (design 29).
    pub fn refile(&mut self, region: u32, keys: &[u32], edits: &Edits) {
        self.cells.remove(&region);
        for &key in keys {
            self.file(key);
        }
        if self.cells.contains_key(&region) {
            self.refresh(region, edits);
        } else {
            self.digests.remove(&region);
        }
    }

    fn file(&mut self, key: u32) -> Option<u32> {
        let a = crate::cell_key::unpack(key)?;
        let (r, p) = (region::region(a)?, region::place(a)?);
        self.cells.entry(r).or_default().insert(p, key);
        Some(r)
    }

    fn refresh(&mut self, region: u32, edits: &Edits) {
        let digest = fnv1a64(&self.encode(region, edits));
        self.digests.insert(region, digest);
    }
}

/// A region's digest worked out from nothing but `edits`: the full
/// recompute the kept digest is checked against.
pub fn recompute(region: u32, edits: &Edits) -> u64 {
    let mut cells: Vec<(u16, &[(u16, Material)])> = Vec::new();
    let mut keys: Vec<u32> = edits.all().iter().map(|e| e.cell).collect();
    keys.dedup();
    for key in keys {
        let Some(a) = crate::cell_key::unpack(key) else {
            continue;
        };
        if region::region(a) == Some(region) {
            cells.push((region::place(a).unwrap_or(0), edits.for_cell(key)));
        }
    }
    fnv1a64(&encode(region, &cells))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_key::{Address, key};
    use crate::edits::Edit;

    fn cell(face: u8, i: u32, j: u32) -> u32 {
        key(Address {
            face,
            level: 11,
            i,
            j,
        })
        .unwrap()
    }

    /// FNV-1a 64's published test vectors.
    #[test]
    fn fnv1a64_matches_its_reference_values() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    /// Task 3.1: a pinned digest for a fixed region. If this moves, every
    /// client of an older build disagrees with every server of a newer one:
    /// change it only with `ENCODING_VERSION`.
    #[test]
    fn a_fixed_regions_digest_is_pinned() {
        let mut edits = Edits::new();
        for (c, layer, m) in [
            (cell(4, 70, 10), 200, Material::Air),
            (cell(4, 70, 10), 199, Material::Air),
            (cell(4, 65, 3), 201, Material::Torch),
            (cell(4, 100, 20), 150, Material::Stone),
        ] {
            edits.set(Edit {
                cell: c,
                layer,
                material: m,
            });
        }
        let regions = Regions::of(&edits);
        let r = region::of_key(cell(4, 70, 10)).unwrap();
        assert_eq!(regions.regions(), vec![r]);
        let bytes = regions.encode(r, &edits);
        assert_eq!(bytes[0], ENCODING_VERSION);
        assert_eq!(&bytes[1..5], &r.to_le_bytes());
        assert_eq!(bytes[5], EDITS_TAG);
        assert_eq!(u32::from_le_bytes(bytes[6..10].try_into().unwrap()), 3);
        assert_eq!(bytes.len(), 10 + 3 * 4 + 4 * 3);
        assert_eq!(regions.digest(r), 0xb109_82f7_a284_8128, "pinned digest");
    }

    /// An empty region's encoding has no section, and a region never edited
    /// digests as empty.
    #[test]
    fn an_unedited_region_digests_as_empty() {
        let r = region::of_key(cell(9, 1000, 1000)).unwrap();
        assert_eq!(encode(r, &[]).len(), 5);
        assert_eq!(Regions::default().digest(r), fnv1a64(&encode(r, &[])));
        assert_eq!(recompute(r, &Edits::new()), fnv1a64(&encode(r, &[])));
    }

    /// Task 3.1: the kept digest equals a full recompute after a random
    /// sequence of changes, in every region, and two worlds built from the
    /// same edits in a different order of regions agree.
    #[test]
    fn the_kept_digest_equals_a_full_recompute() {
        let mut rng = crate::fauna::Rng::new(0x5eed);
        let mut edits = Edits::new();
        let mut regions = Regions::default();
        let materials = [
            Material::Air,
            Material::Stone,
            Material::Torch,
            Material::Dirt,
        ];
        // A few hundred cells over a handful of regions, edges included.
        let mut cells = Vec::new();
        for face in [0u8, 1, 5] {
            for n in 0..60u32 {
                cells.push(cell(face, (n * 13) % 200, (n * 7) % 130));
            }
            cells.push(cell(face, 0, 64));
            cells.push(cell(face, 64, 0));
        }
        for step in 0..4000 {
            let c = cells[(rng.next_u64() % cells.len() as u64) as usize];
            let layer = 150 + (rng.next_u64() % 8) as u16;
            let material = materials[(rng.next_u64() % 4) as usize];
            edits.set(Edit {
                cell: c,
                layer,
                material,
            });
            let r = regions.note(c, &edits).unwrap();
            if step % 97 == 0 {
                assert_eq!(regions.digest(r), recompute(r, &edits), "step {step}");
            }
        }
        let rebuilt = Regions::of(&edits);
        for r in regions.regions() {
            assert_eq!(regions.digest(r), recompute(r, &edits));
            assert_eq!(rebuilt.digest(r), regions.digest(r));
            assert_eq!(rebuilt.encode(r, &edits), regions.encode(r, &edits));
        }
        assert!(regions.regions().len() >= 6);
    }

    /// `world-persistence` 5.1: a region's snapshot round-trips: decoded
    /// and folded into an empty world, it encodes to the same bytes and
    /// digests the same; a cut or extended one is refused.
    #[test]
    fn a_regions_snapshot_round_trips() {
        let mut edits = Edits::new();
        for (c, layer, m) in [
            (cell(4, 70, 10), 200, Material::Air),
            (cell(4, 70, 10), 199, Material::Air),
            (cell(4, 65, 3), 201, Material::Torch),
            (cell(4, 100, 20), 150, Material::flowing(2, false)),
        ] {
            edits.set(Edit {
                cell: c,
                layer,
                material: m,
            });
        }
        let regions = Regions::of(&edits);
        let r = regions.regions()[0];
        let bytes = regions.encode(r, &edits);
        let (back, held) = snapshot_edits(&bytes).unwrap();
        assert_eq!(back, r);
        let mut copy = Edits::new();
        for edit in held {
            copy.set(edit);
        }
        let copied = Regions::of(&copy);
        assert_eq!(copied.encode(r, &copy), bytes);
        assert_eq!(copied.digest(r), regions.digest(r));
        for n in 0..bytes.len() {
            if n == 5 {
                // The header alone is an empty region, which is a region.
                assert_eq!(decode(&bytes[..n]).map(|d| d.1.len()), Some(0));
                continue;
            }
            assert!(decode(&bytes[..n]).is_none(), "cut at {n}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(decode(&longer).is_none());
        let empty = encode(r, &[]);
        assert_eq!(decode(&empty), Some((r, Vec::new())));
    }

    /// `world-persistence` 5.1: every finest cell's region contains it: the
    /// cell's key comes back from its region and place.
    #[test]
    fn every_cells_region_contains_it() {
        for face in 0..20u8 {
            for (i, j) in [
                (0, 0),
                (2048, 0),
                (0, 2048),
                (1024, 1024),
                (63, 0),
                (64, 0),
                (100, 1947),
            ] {
                let a = Address {
                    face,
                    level: 11,
                    i,
                    j,
                };
                let Some(k) = key(a) else { continue };
                let c = crate::cell_key::unpack(k).unwrap();
                let (r, p) = (region::region(c).unwrap(), region::place(c).unwrap());
                assert_eq!(region::address(r, p).and_then(key), Some(k));
            }
        }
    }

    /// One layer changed in one cell changes that region's digest and no
    /// other's.
    #[test]
    fn a_change_moves_only_its_own_regions_digest() {
        let (a, b) = (cell(2, 10, 10), cell(2, 1000, 500));
        let mut edits = Edits::new();
        edits.set(Edit {
            cell: a,
            layer: 100,
            material: Material::Air,
        });
        edits.set(Edit {
            cell: b,
            layer: 100,
            material: Material::Air,
        });
        let mut regions = Regions::of(&edits);
        let (ra, rb) = (region::of_key(a).unwrap(), region::of_key(b).unwrap());
        assert_ne!(ra, rb);
        let (da, db) = (regions.digest(ra), regions.digest(rb));
        edits.set(Edit {
            cell: a,
            layer: 100,
            material: Material::Stone,
        });
        regions.note(a, &edits);
        assert_ne!(regions.digest(ra), da);
        assert_eq!(regions.digest(rb), db);
    }
}
