//! The order a world's changes were accepted in, per region (`multiplayer`
//! design 4, `edit-pipeline` 5).
//!
//! ACCEPT gives each change the next sequence number of its region, and
//! APPLY takes changes in that order. A client applies a region's changes
//! in sequence and asks for the region again on a gap; a snapshot carries
//! the sequence it is as of. The digest kept beside it is the region's
//! state at that sequence (`net::digest`).
//!
//! The numbers live with the world while it is open. They are saved with
//! the region snapshots of `world-persistence` 5.1 when those are built;
//! until then a world opens with every region at its edit count, which is
//! the same for every process that opens the same save.

use super::digest::Regions;
use crate::edits::{Edit, Edits};
use std::collections::HashMap;

/// Where a key no cell has is numbered: no region address is all ones (its
/// face would be 31).
pub const NO_REGION: u32 = u32::MAX;

/// A change, accepted and numbered in its region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sequenced {
    pub region: u32,
    /// 1 for a region's first change.
    pub seq: u64,
    pub edit: Edit,
}

/// Each region's last sequence and its digest.
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    /// A keyed lookup only.
    seqs: HashMap<u32, u64>,
    regions: Regions,
}

impl Ledger {
    /// The ledger of a world opened with `edits`: each region at the number
    /// of layer changes it holds, so two processes opening one save agree.
    pub fn of(edits: &Edits) -> Self {
        let mut seqs: HashMap<u32, u64> = HashMap::new();
        for edit in edits.all() {
            if let Some(r) = crate::cell_key::region::of_key(edit.cell) {
                *seqs.entry(r).or_default() += 1;
            }
        }
        Self {
            seqs,
            regions: Regions::of(edits),
        }
    }

    /// Number `edit`, which `edits` now holds: the next sequence of its
    /// region, and the region's digest brought up to date. A key no cell has
    /// (which no real edit has, only a test's) is numbered under
    /// [`NO_REGION`] and digested nowhere: the ledger never refuses what the
    /// save took.
    pub fn accept(&mut self, edit: Edit, edits: &Edits) -> Sequenced {
        let region = self.regions.note(edit.cell, edits).unwrap_or(NO_REGION);
        let seq = self.seqs.entry(region).or_default();
        *seq += 1;
        Sequenced {
            region,
            seq: *seq,
            edit,
        }
    }

    /// `region` replaced whole from a snapshot as of `seq` (design 29): its
    /// cells are `keys`, which `edits` now hold; its digest is refreshed and
    /// its sequence set to the server's.
    pub fn resync(&mut self, region: u32, seq: u64, keys: &[u32], edits: &Edits) {
        self.regions.refile(region, keys, edits);
        self.seqs.insert(region, seq);
    }

    /// A region's last sequence: nought if nothing in it was ever changed.
    pub fn seq(&self, region: u32) -> u64 {
        self.seqs.get(&region).copied().unwrap_or(0)
    }

    /// A region's digest now.
    pub fn digest(&self, region: u32) -> u64 {
        self.regions.digest(region)
    }

    /// The regions and their digests, for encoding a snapshot.
    pub fn regions(&self) -> &Regions {
        &self.regions
    }

    /// A region whole, as of its last sequence (`world-persistence` 5.1):
    /// what a server sends on subscribe or resync, and what a save's region
    /// file will hold.
    pub fn snapshot(&self, region: u32, edits: &Edits) -> Snapshot {
        Snapshot {
            region,
            seq: self.seq(region),
            bytes: self.regions.encode(region, edits),
        }
    }
}

/// A region's state as of a sequence: its canonical bytes
/// (`net::digest::encode`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub region: u32,
    pub seq: u64,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_key::{Address, key};
    use crate::net::digest::recompute;
    use crate::terrain::Material;

    fn cell(i: u32, j: u32) -> u32 {
        key(Address {
            face: 6,
            level: 11,
            i,
            j,
        })
        .unwrap()
    }

    /// Each region counts its own changes from 1, and its digest follows.
    #[test]
    fn each_region_numbers_its_own_changes() {
        let (a, b, c) = (cell(10, 10), cell(11, 10), cell(900, 400));
        let mut edits = Edits::new();
        let mut ledger = Ledger::default();
        let mut seqs = Vec::new();
        for (cell, layer) in [(a, 100), (b, 100), (c, 100), (a, 101), (c, 99)] {
            let edit = Edit {
                cell,
                layer,
                material: Material::Air,
            };
            edits.set(edit);
            let s = ledger.accept(edit, &edits);
            assert_eq!(ledger.digest(s.region), recompute(s.region, &edits));
            seqs.push((s.region, s.seq));
        }
        let (ra, rc) = (seqs[0].0, seqs[2].0);
        assert_ne!(ra, rc);
        assert_eq!(
            seqs.iter().map(|s| s.1).collect::<Vec<_>>(),
            vec![1, 2, 1, 3, 2]
        );
        assert_eq!((ledger.seq(ra), ledger.seq(rc)), (3, 2));
        assert_eq!(ledger.seq(rc ^ 1), 0);
        let nowhere = Edit {
            cell: u32::MAX,
            layer: 1,
            material: Material::Air,
        };
        edits.set(nowhere);
        assert_eq!(ledger.accept(nowhere, &edits).region, NO_REGION);
        // A snapshot is as of the region's last sequence, and its bytes
        // digest to the region's digest.
        let snap = ledger.snapshot(ra, &edits);
        assert_eq!(snap.seq, 3);
        assert_eq!(crate::net::digest::fnv1a64(&snap.bytes), ledger.digest(ra));
    }

    /// Two processes opening the same edits agree on every region's
    /// sequence and digest.
    #[test]
    fn a_world_opens_with_one_ledger_wherever_it_is_opened() {
        let mut edits = Edits::new();
        for n in 0..50u32 {
            edits.set(Edit {
                cell: cell(n * 37 % 300, n * 11 % 200),
                layer: 90 + (n % 4) as u16,
                material: Material::Stone,
            });
        }
        let (one, two) = (Ledger::of(&edits), Ledger::of(&edits.clone()));
        for r in one.regions().regions() {
            assert_eq!(one.seq(r), two.seq(r));
            assert_eq!(one.digest(r), two.digest(r));
            assert!(one.seq(r) > 0);
        }
    }

    /// Design 29: a copy with a wrong extra layer, replaced whole from the
    /// server's snapshot, holds exactly the server's edits for the region,
    /// at its sequence, with its digest equal to the server's and to a full
    /// recompute.
    #[test]
    fn a_region_replaced_from_a_snapshot_matches_the_server() {
        let (a, b) = (cell(100, 100), cell(101, 100));
        let set = |e: &mut Edits, l: &mut Ledger, cell, layer, material| {
            let edit = Edit {
                cell,
                layer,
                material,
            };
            e.set(edit);
            l.accept(edit, e);
        };
        let (mut se, mut sl) = (Edits::new(), Ledger::default());
        set(&mut se, &mut sl, a, 200, Material::Air);
        set(&mut se, &mut sl, a, 199, Material::Air);
        set(&mut se, &mut sl, b, 201, Material::Stone);
        let (mut ce, mut cl) = (Edits::new(), Ledger::default());
        set(&mut ce, &mut cl, a, 200, Material::Air);
        // A layer the server never had: a merge would keep it.
        set(&mut ce, &mut cl, a, 150, Material::Stone);
        let region = crate::cell_key::region::of_key(a).unwrap();
        assert_eq!(region, crate::cell_key::region::of_key(b).unwrap());
        assert_ne!(cl.digest(region), sl.digest(region));
        let snap = sl.snapshot(region, &se);
        let (r, edits) = crate::net::digest::snapshot_edits(&snap.bytes).unwrap();
        assert_eq!(r, region);
        for cell in [a, b] {
            ce.clear_cell(cell);
        }
        for &edit in &edits {
            ce.set(edit);
        }
        let mut keys: Vec<u32> = edits.iter().map(|e| e.cell).collect();
        keys.dedup();
        cl.resync(region, snap.seq, &keys, &ce);
        assert_eq!(cl.digest(region), sl.digest(region));
        assert_eq!(cl.digest(region), recompute(region, &ce));
        assert_eq!(cl.seq(region), sl.seq(region));
        assert_eq!(ce.for_cell(a), se.for_cell(a));
    }
}
