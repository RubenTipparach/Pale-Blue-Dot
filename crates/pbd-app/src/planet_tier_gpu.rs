//! The column tier as the GPU reads it, worked out off the render thread
//! (`fine-set-upload`).
//!
//! A fine set is immutable once it is shared, so everything the terrain pass
//! reads about its columns (the records with their reach tops, the light, the
//! materials, the placed sheets, the centres) is a function of it. A
//! [`TierImage`] holds that, made where the set is made: on a landing's task,
//! or by [`pack_fine_gpu`] at the end of the frame that made an edit, a water
//! tick or a relight, from the image before it, packing again only the slots
//! that differ. The [`Delta`] says which, so the upload writes those ranges
//! and the render thread works nothing out.

use super::GpuCell;
use super::column::{
    self, GpuColumn, LIGHT_WORDS, MATERIAL_WORDS, pack_materials, pack_placed_sheets, reach_rings,
    reach_tops, with_reach_top,
};
use super::lod::{self, FineSet, PlanetFine};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use std::ops::Range;
use std::sync::Arc;

/// Slots closer than this are written as one range: an edit's relit
/// neighbourhood is a few writes, not hundreds (design D4).
const SLOT_GAP: usize = 16;
/// The same for the levels' cell records.
const CELL_GAP: usize = 64;
/// Cell records compared at a time when the levels are diffed.
const CELL_CHUNK: usize = 64;

/// What the terrain pass reads about one fine set's columns, packed.
pub struct TierImage {
    /// The set it was made from: what the next version is diffed against,
    /// and, held, an address no other set can take.
    pub set: Arc<FineSet>,
    /// The reflection reach its records' tops were worked out for.
    pub reach_m: f32,
    /// Neighbour steps the reach tops spread over: a function of the layout.
    rings: usize,
    pub records: Vec<GpuColumn>,
    pub light: Vec<u32>,
    pub materials: Vec<u32>,
    pub sheets: Vec<u32>,
    pub centres: Vec<[f32; 4]>,
}

/// What differs between a version and the one before it: ascending,
/// disjoint ranges of record indices per level, and of slots per column
/// buffer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Delta {
    /// The fine-set version these are the changes from.
    pub base: u64,
    pub levels: [Vec<Range<usize>>; 4],
    pub records: Vec<Range<usize>>,
    pub light: Vec<Range<usize>>,
    pub materials: Vec<Range<usize>>,
    pub sheets: Vec<Range<usize>>,
}

/// The image of the fine set `PlanetFine` holds at `version`, shared with the
/// render world; `delta` is `None` where everything must go up.
#[derive(Resource, Clone, ExtractResource)]
pub struct FineGpu {
    pub version: u64,
    pub image: Arc<TierImage>,
    pub delta: Option<Arc<Delta>>,
}

/// What a landing's task hands back: the set, its contact tier, its bounce
/// and lamp bake, and its GPU image.
pub(crate) type Landed = (
    Arc<FineSet>,
    super::contact::PreparedFine,
    super::bounce::SetLight,
    Arc<TierImage>,
);

/// A landing's image, built on its task with the set (design D3), waiting for
/// [`pack_fine_gpu`] to take it.
#[derive(Resource, Default)]
pub struct PreparedGpu(pub Option<Arc<TierImage>>);

impl TierImage {
    /// The whole image of `set`.
    pub fn build(set: Arc<FineSet>, reach_m: f32) -> Self {
        let tier = &set.columns;
        let records = tier.gpu_records();
        let rings = reach_rings(records, tier.centre_dirs(), reach_m);
        let tops = reach_tops(records, rings);
        let records = records
            .iter()
            .zip(tops)
            .map(|(record, top)| with_reach_top(record, top))
            .collect();
        let slots = tier.columns.len();
        let mut light = vec![0u32; slots * LIGHT_WORDS];
        for (slot, out) in light.chunks_exact_mut(LIGHT_WORDS).enumerate() {
            tier.pack_light(slot, out);
        }
        let mut materials = vec![0u32; slots * MATERIAL_WORDS];
        let mut sheets = vec![0u32; slots * MATERIAL_WORDS];
        for ((column, m), s) in tier
            .columns
            .iter()
            .zip(materials.chunks_exact_mut(MATERIAL_WORDS))
            .zip(sheets.chunks_exact_mut(MATERIAL_WORDS))
        {
            pack_materials(column, m);
            pack_placed_sheets(column, s);
        }
        let centres = tier.gpu_centres();
        Self {
            set,
            reach_m,
            rings,
            records,
            light,
            materials,
            sheets,
            centres,
        }
    }

    /// Whether `set` lays its records and slots as this image's set does, so
    /// a diff against it is a diff of contents: the same anchor, level
    /// lengths, slots, slot directions and neighbour links.
    pub fn same_layout(&self, set: &FineSet, reach_m: f32) -> bool {
        let (old, new) = (&*self.set, set);
        let links = |r: &GpuColumn| {
            (
                r.neighbors,
                r.more[0],
                r.more[1],
                r.more[2] & column::DEGREE_MASK,
            )
        };
        self.reach_m.to_bits() == reach_m.to_bits()
            && old.anchor == new.anchor
            && old
                .levels
                .iter()
                .zip(&new.levels)
                .all(|(a, b)| a.len() == b.len())
            && old.columns.columns.len() == new.columns.columns.len()
            && old.columns.centre_dirs() == new.columns.centre_dirs()
            && old
                .columns
                .gpu_records()
                .iter()
                .zip(new.columns.gpu_records())
                .all(|(a, b)| links(a) == links(b))
    }

    /// The image of `set`, a set of the same layout as this image's
    /// ([`TierImage::same_layout`]), and what differs from this one. Only a
    /// slot whose column changed has its materials packed again.
    pub fn update(&self, set: Arc<FineSet>, base: u64) -> (Self, Delta) {
        let (old, new) = (&self.set.columns, &set.columns);
        let slots = new.columns.len();
        let mut delta = Delta {
            base,
            ..Delta::default()
        };
        let mut light = self.light.clone();
        let mut materials = self.materials.clone();
        let mut sheets = self.sheets.clone();
        let mut packed = [0u32; MATERIAL_WORDS];
        let mut lit = [0u32; LIGHT_WORDS];
        let (mut light_slots, mut material_slots, mut sheet_slots) = (vec![], vec![], vec![]);
        for slot in 0..slots {
            let words = slot * LIGHT_WORDS..(slot + 1) * LIGHT_WORDS;
            new.pack_light(slot, &mut lit);
            if light[words.clone()] != lit {
                light[words].copy_from_slice(&lit);
                light_slots.push(slot);
            }
            if old.columns[slot] == new.columns[slot] {
                continue;
            }
            let words = slot * MATERIAL_WORDS..(slot + 1) * MATERIAL_WORDS;
            pack_materials(&new.columns[slot], &mut packed);
            if materials[words.clone()] != packed {
                materials[words.clone()].copy_from_slice(&packed);
                material_slots.push(slot);
            }
            pack_placed_sheets(&new.columns[slot], &mut packed);
            if sheets[words.clone()] != packed {
                sheets[words].copy_from_slice(&packed);
                sheet_slots.push(slot);
            }
        }
        // The tops spread over the whole tier, and are cheap: worked out
        // again, then compared record by record.
        let tops = reach_tops(new.gpu_records(), self.rings);
        let records: Vec<GpuColumn> = new
            .gpu_records()
            .iter()
            .zip(tops)
            .map(|(record, top)| with_reach_top(record, top))
            .collect();
        let record_slots: Vec<usize> = records
            .iter()
            .zip(&self.records)
            .enumerate()
            .filter(|(_, (a, b))| bytemuck::bytes_of(*a) != bytemuck::bytes_of(*b))
            .map(|(i, _)| i)
            .collect();
        delta.records = ranges(record_slots, SLOT_GAP);
        delta.light = ranges(light_slots, SLOT_GAP);
        delta.materials = ranges(material_slots, SLOT_GAP);
        delta.sheets = ranges(sheet_slots, SLOT_GAP);
        // The levels are tens of megabytes: compared on a thread each.
        let changed: Vec<Vec<usize>> = std::thread::scope(|scope| {
            let jobs: Vec<_> = self
                .set
                .levels
                .iter()
                .zip(&set.levels)
                .map(|(a, b)| {
                    scope.spawn(move || {
                        a.chunks(CELL_CHUNK)
                            .zip(b.chunks(CELL_CHUNK))
                            .enumerate()
                            .filter(|(_, (x, y))| {
                                bytemuck::cast_slice::<GpuCell, u8>(x)
                                    != bytemuck::cast_slice::<GpuCell, u8>(y)
                            })
                            .map(|(i, _)| i)
                            .collect()
                    })
                })
                .collect();
            jobs.into_iter()
                .map(|job| job.join().expect("a level's diff"))
                .collect()
        });
        for (k, (changed, b)) in changed.into_iter().zip(&set.levels).enumerate() {
            delta.levels[k] = ranges(changed, CELL_GAP / CELL_CHUNK)
                .into_iter()
                .map(|r| r.start * CELL_CHUNK..(r.end * CELL_CHUNK).min(b.len()))
                .collect();
        }
        let image = Self {
            set,
            reach_m: self.reach_m,
            rings: self.rings,
            records,
            light,
            materials,
            sheets,
            centres: self.centres.clone(),
        };
        (image, delta)
    }

    /// Bytes this version writes, with `delta` or whole.
    #[cfg(test)]
    pub fn bytes_written(&self, delta: Option<&Delta>, light_whole: bool) -> usize {
        let mut bytes = 0;
        tier_writes(self, delta, light_whole, |_, _, data| bytes += data.len());
        bytes
    }
}

/// Ascending indices as ranges, joining two closer than `gap`.
fn ranges(indices: Vec<usize>, gap: usize) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = vec![];
    for i in indices {
        match out.last_mut() {
            Some(last) if i < last.end + gap.max(1) => last.end = i + 1,
            _ => out.push(i..i + 1),
        }
    }
    out
}

/// A buffer the tier's upload writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Fine level `k`'s region of the cell buffer, from its start.
    Level(usize),
    /// The column records.
    Records,
    /// The light half being written, from its start.
    Light,
    /// The materials, from the buffer's start.
    Materials,
    /// The placed sheets, from `PLACED_SHEETS_BASE`.
    Sheets,
    /// The slot centres.
    Centres,
}

/// Every write that brings buffers holding `delta.base`'s image to this
/// image, or, with no delta, every write of the whole image, as `(target,
/// byte offset in it, bytes)`. Clipped to each buffer's capacity. The light
/// goes up whole where `light_whole` (a relight into the other half). With no
/// columns only the levels go up, as before this change.
pub fn tier_writes(
    image: &TierImage,
    delta: Option<&Delta>,
    light_whole: bool,
    mut write: impl FnMut(Target, u64, &[u8]),
) {
    #[allow(clippy::single_range_in_vec_init)]
    let whole = |len: usize| if len == 0 { vec![] } else { vec![0..len] };
    let cap = column::COLUMN_CAPACITY as usize;
    let cell = size_of::<GpuCell>();
    for (k, level) in image.set.levels.iter().enumerate() {
        let level = &level[..level.len().min(lod::FINE_CAPACITY as usize)];
        let spans = delta.map_or_else(|| whole(level.len()), |d| d.levels[k].clone());
        for r in spans {
            let r = r.start.min(level.len())..r.end.min(level.len());
            if !r.is_empty() {
                write(
                    Target::Level(k),
                    (r.start * cell) as u64,
                    bytemuck::cast_slice(&level[r]),
                );
            }
        }
    }
    if image.records.is_empty() {
        return;
    }
    let mut put = |target: Target, words: &[u32], per_slot: usize, spans: Vec<Range<usize>>| {
        let slots = (words.len() / per_slot).min(cap);
        for r in spans {
            let r = r.start.min(slots)..r.end.min(slots);
            if !r.is_empty() {
                write(
                    target,
                    (r.start * per_slot * 4) as u64,
                    bytemuck::cast_slice(&words[r.start * per_slot..r.end * per_slot]),
                );
            }
        }
    };
    let slots = image.records.len();
    put(
        Target::Records,
        bytemuck::cast_slice(&image.records),
        size_of::<GpuColumn>() / 4,
        delta.map_or_else(|| whole(slots), |d| d.records.clone()),
    );
    let light = match delta {
        Some(d) if !light_whole => d.light.clone(),
        _ => whole(image.light.len() / LIGHT_WORDS),
    };
    put(Target::Light, &image.light, LIGHT_WORDS, light);
    put(
        Target::Materials,
        &image.materials,
        MATERIAL_WORDS,
        delta.map_or_else(|| whole(slots), |d| d.materials.clone()),
    );
    put(
        Target::Sheets,
        &image.sheets,
        MATERIAL_WORDS,
        delta.map_or_else(|| whole(slots), |d| d.sheets.clone()),
    );
    // The centres are the layout: a delta never changes them.
    if delta.is_none() {
        put(
            Target::Centres,
            bytemuck::cast_slice(&image.centres),
            4,
            whole(image.centres.len()),
        );
    }
}

/// The reflection reach the records are made for.
pub fn reach_m(lighting: Option<&crate::config::LightingSettings>) -> f32 {
    lighting.map_or_else(
        || crate::config::LightingSettings::default().reflection_reach_m,
        |l| l.reflection_reach_m,
    )
}

/// Make the image of the fine set the frame ends with (design D2): run in
/// `Last`, after everything that makes a version.
pub fn pack_fine_gpu(
    mut commands: Commands,
    fine: Option<Res<PlanetFine>>,
    gpu: Option<Res<FineGpu>>,
    prepared: Option<ResMut<PreparedGpu>>,
    lighting: Option<Res<crate::config::LightingSettings>>,
) {
    let Some(fine) = fine else {
        return;
    };
    let prepared = prepared.and_then(|mut p| p.0.take());
    if gpu.as_ref().is_some_and(|g| g.version == fine.version) {
        return;
    }
    let started = std::time::Instant::now();
    let reach = reach_m(lighting.as_deref());
    let (image, delta, how) = match prepared {
        Some(p) if Arc::ptr_eq(&p.set, &fine.set) && p.reach_m.to_bits() == reach.to_bits() => {
            (p, None, "landed")
        }
        prepared => {
            let last = gpu.as_ref().map(|g| (g.image.clone(), g.version));
            // A replayed landing diffs against its own prepared image; the
            // delta it makes is not one the GPU holds the base of, so it
            // goes up whole.
            if let Some(p) = prepared.filter(|p| p.same_layout(&fine.set, reach)) {
                let (image, _) = p.update(fine.set.clone(), u64::MAX);
                (Arc::new(image), None, "replayed")
            } else if let Some((last, base)) = last.filter(|(l, _)| l.same_layout(&fine.set, reach))
            {
                let (image, delta) = last.update(fine.set.clone(), base);
                (Arc::new(image), Some(Arc::new(delta)), "delta")
            } else {
                (
                    Arc::new(TierImage::build(fine.set.clone(), reach)),
                    None,
                    "whole",
                )
            }
        }
    };
    debug!(
        "TIER_IMAGE v{} {how} in {:.2} ms{}",
        fine.version,
        started.elapsed().as_secs_f64() * 1e3,
        delta.as_ref().map_or(String::new(), |d| format!(
            ": {} record, {} light, {} material ranges",
            d.records.len(),
            d.light.len(),
            d.materials.len()
        ))
    );
    lod::spent("tier image", started);
    commands.insert_resource(FineGpu {
        version: fine.version,
        image,
        delta,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::terrain::Material;

    /// The bytes of each buffer the upload writes, as a GPU would hold them.
    #[derive(Default, Clone, PartialEq, Eq, Debug)]
    struct Buffers {
        levels: [Vec<u8>; 4],
        records: Vec<u8>,
        light: Vec<u8>,
        materials: Vec<u8>,
        sheets: Vec<u8>,
        centres: Vec<u8>,
    }

    impl Buffers {
        fn apply(&mut self, image: &TierImage, delta: Option<&Delta>, light_whole: bool) {
            tier_writes(image, delta, light_whole, |target, at, data| {
                let buffer = match target {
                    Target::Level(k) => &mut self.levels[k],
                    Target::Records => &mut self.records,
                    Target::Light => &mut self.light,
                    Target::Materials => &mut self.materials,
                    Target::Sheets => &mut self.sheets,
                    Target::Centres => &mut self.centres,
                };
                let end = at as usize + data.len();
                if buffer.len() < end {
                    buffer.resize(end, 0);
                }
                buffer[at as usize..end].copy_from_slice(data);
            });
        }
    }

    /// Where two sets of buffers first differ, by buffer and byte, without
    /// printing megabytes of them.
    fn same(a: &Buffers, b: &Buffers, what: &str) {
        let pairs = [
            ("level 0", &a.levels[0], &b.levels[0]),
            ("level 1", &a.levels[1], &b.levels[1]),
            ("level 2", &a.levels[2], &b.levels[2]),
            ("level 3", &a.levels[3], &b.levels[3]),
            ("records", &a.records, &b.records),
            ("light", &a.light, &b.light),
            ("materials", &a.materials, &b.materials),
            ("sheets", &a.sheets, &b.sheets),
            ("centres", &a.centres, &b.centres),
        ];
        for (name, x, y) in pairs {
            assert_eq!(x.len(), y.len(), "{what}: {name} length");
            let first = x.iter().zip(y.iter()).position(|(p, q)| p != q);
            assert!(first.is_none(), "{what}: {name} differs at byte {first:?}");
        }
    }

    impl Buffers {
        /// What a set's whole upload leaves in its first bytes: a GPU buffer
        /// is its capacity long, and a smaller set leaves the tail it does not
        /// reach as it was, before this change as after.
        fn cut_to(&mut self, like: &Buffers) {
            for (a, b) in self.levels.iter_mut().zip(&like.levels) {
                a.truncate(b.len());
            }
            self.records.truncate(like.records.len());
            self.light.truncate(like.light.len());
            self.materials.truncate(like.materials.len());
            self.sheets.truncate(like.sheets.len());
            self.centres.truncate(like.centres.len());
        }
    }

    fn whole(set: &Arc<FineSet>) -> Buffers {
        let mut b = Buffers::default();
        b.apply(&TierImage::build(set.clone(), 64.0), None, false);
        b
    }

    fn ground() -> Arc<FineSet> {
        crate::planet::test_ground(Vec3::new(0.3, 0.8, 0.52).normalize())
            .0
            .set
    }

    /// A slot and the layer of its top solid cell, away from the tier's rim.
    fn top_of(set: &FineSet) -> (usize, usize) {
        let slot = set.columns.columns.len() / 2;
        let column = &set.columns.columns[slot];
        let top = (1..pbd_core::column::LAYERS)
            .rev()
            .find(|&l| column.material(l) != Material::Air && !column.material(l).is_water())
            .unwrap();
        (slot, top)
    }

    /// `fine-set-upload` task 2.2: an edit, a water change, a relight and a
    /// landing each leave the buffers byte for byte as a whole upload of the
    /// new version does.
    #[test]
    fn a_partial_update_writes_what_a_full_one_does() {
        let a = ground();
        let base = TierImage::build(a.clone(), 64.0);
        let mut gpu = whole(&a);
        let edits = pbd_core::edits::Edits::new();
        let (slot, top) = top_of(&a);
        let record = a.columns.water.records[slot];
        let check = |gpu: &mut Buffers, from: &TierImage, to: Arc<FineSet>, what: &str| {
            assert!(from.same_layout(&to, 64.0), "{what}: same layout");
            let (image, delta) = from.update(to.clone(), 1);
            gpu.apply(&image, Some(&delta), false);
            same(gpu, &whole(&to), &format!("{what}: partial against whole"));
            let written = image.bytes_written(Some(&delta), false);
            let full = image.bytes_written(None, false);
            assert!(written < full / 4, "{what}: {written} of {full} bytes");
            image
        };
        // An edit as digging makes one: the top dug out, relit.
        let mut dug = (*a).clone();
        assert!(dug.write_layer(record, top, Material::Air, &edits));
        dug.columns.relight();
        dug.mark_sea();
        let dug = Arc::new(dug);
        let after_dig = check(&mut gpu, &base, dug.clone(), "edit");
        // A water change as the rule writes one, on the dug set.
        let mut wet = (*dug).clone();
        assert!(wet.write_layer(record, top, Material::Water, &edits));
        wet.columns.relight();
        let wet = Arc::new(wet);
        let after_wet = check(&mut gpu, &after_dig, wet.clone(), "water");
        // The dusk relight.
        let mut dusk = (*wet).clone();
        dusk.columns.set_dusk(!dusk.columns.dusk());
        let dusk = Arc::new(dusk);
        let (image, delta) = after_wet.update(dusk.clone(), 1);
        // Into the other half the light goes whole.
        let mut other = gpu.clone();
        other.light.clear();
        other.apply(&image, Some(&delta), true);
        same(&other, &whole(&dusk), "relight into the other half");
        gpu.apply(&image, Some(&delta), false);
        same(&gpu, &whole(&dusk), "relight in place");
        // A landing elsewhere is a new layout: it goes up whole.
        let landed = crate::planet::test_ground(Vec3::new(0.31, 0.8, 0.51).normalize())
            .0
            .set;
        assert!(!image.same_layout(&landed, 64.0));
        gpu.apply(&TierImage::build(landed.clone(), 64.0), None, false);
        let want = whole(&landed);
        gpu.cut_to(&want);
        same(&gpu, &want, "landing");
    }

    /// `fine-set-upload` D1: the light as it lies is the packed light.
    #[test]
    fn the_light_cast_is_the_packed_light() {
        let set = ground();
        let image = TierImage::build(set.clone(), 64.0);
        assert_eq!(image.light, set.columns.gpu_light());
        assert_eq!(image.materials, set.columns.gpu_materials());
        assert_eq!(image.sheets, set.columns.gpu_placed_sheets());
        let records = set.columns.gpu_records_with_reach(64.0);
        assert_eq!(
            bytemuck::cast_slice::<GpuColumn, u8>(&image.records),
            bytemuck::cast_slice::<GpuColumn, u8>(&records)
        );
    }

    #[test]
    fn ranges_join_close_indices() {
        assert_eq!(ranges(vec![], 4), Vec::<Range<usize>>::new());
        assert_eq!(ranges(vec![1, 2, 3, 9, 30], 4), vec![1..4, 9..10, 30..31]);
        assert_eq!(ranges(vec![1, 4, 7], 4), vec![1..8]);
        assert_eq!(ranges(vec![1, 2], 1), vec![1..3]);
        assert_eq!(ranges(vec![1, 3], 1), vec![1..2, 3..4]);
    }

    /// `fine-set-upload` D2 and D5: the image follows the version in the
    /// frame that made it, an unchanged set makes none, and a version the
    /// diff cannot follow goes up whole.
    #[test]
    fn the_image_follows_the_version_in_its_frame() {
        let set = ground();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PreparedGpu>()
            .insert_resource(PlanetFine {
                set: set.clone(),
                version: 3,
            })
            .add_systems(Last, pack_fine_gpu);
        app.update();
        let gpu = app.world().resource::<FineGpu>().clone();
        assert_eq!(gpu.version, 3);
        assert!(gpu.delta.is_none(), "the first image goes up whole");
        let (slot, top) = top_of(&set);
        let mut dug = (*set).clone();
        dug.write_layer(
            set.columns.water.records[slot],
            top,
            Material::Air,
            &pbd_core::edits::Edits::new(),
        );
        dug.columns.relight();
        app.insert_resource(PlanetFine {
            set: Arc::new(dug),
            version: 4,
        });
        app.update();
        let gpu = app.world().resource::<FineGpu>().clone();
        assert_eq!(gpu.version, 4);
        assert_eq!(gpu.delta.as_ref().map(|d| d.base), Some(3));
        // A landing's own image is taken as it is.
        let landed = crate::planet::test_ground(Vec3::new(0.31, 0.8, 0.51).normalize())
            .0
            .set;
        let image = Arc::new(TierImage::build(landed.clone(), reach_m(None)));
        app.world_mut().resource_mut::<PreparedGpu>().0 = Some(image.clone());
        app.insert_resource(PlanetFine {
            set: landed,
            version: 5,
        });
        app.update();
        let gpu = app.world().resource::<FineGpu>().clone();
        assert_eq!(gpu.version, 5);
        assert!(gpu.delta.is_none());
        assert!(Arc::ptr_eq(&gpu.image, &image));
    }
}
