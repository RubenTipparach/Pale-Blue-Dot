## Context

`upload_fine` (render world, `planet.rs`) runs when `PlanetFine.version`
moves. Today it calls, on the render thread, `gpu_records_with_reach`,
`gpu_light`, `gpu_materials`, `gpu_placed_sheets` and `gpu_centres` on the
whole `ColumnTier`, and writes every buffer whole. The producers of a version
are: a landing (`planet_lod`, the set built on an `AsyncComputeTaskPool`
task), an edit (`digging.rs`, cloned, written, relit on the main thread), a
water tick (`water_flow.rs`), the dusk relight (`switch_dusk_lamps`), a
contact relight (`planet_contact.rs`) and the loads (`planet.rs`).

## Decisions

### D1. The light is a cast

`Light` is a `u8` newtype and `light::Baked` is `Vec<[Light; LAYERS]>`;
`gpu_light` builds `words[slot * LIGHT_WORDS + layer / 4] |= level << 8 *
(layer % 4)`. On a little-endian target that is the bytes of the arrays as
they lie: each word is `u32::from_le_bytes` of four levels, the same on any
target, with no shifting layer by layer (`ColumnTier::pack_light`).
`pbd-core` takes no `bytemuck` for it. Pinned by a test that the image's
light and `gpu_light`'s loop agree on a baked tier.

### D2. `TierImage`, and `FineGpu` beside `PlanetFine`

```text
TierImage { set: Arc<FineSet>, reach_m, records, tops, light, materials,
            sheets, centres }
FineGpu   { version, image: Arc<TierImage>, delta: Option<Arc<Delta>> }
Delta     { base: u64, levels: [ranges; 4], records, light, materials,
            sheets: ranges of slots / record indices }
```

`FineGpu` is a main-world resource extracted with `PlanetFine`. The image
holds the set it was made from (a strong `Arc`, so its address is never
reused and it can be diffed against): one extra set alive at most, the one
the GPU holds.

The image is made by one system, `pack_fine_gpu`, in `Last` (before the frame
pacing), after every producer of the frame. When `FineGpu.version` is behind
`PlanetFine.version` it picks a base:

1. the landing's prepared image, if it was made from this very set
   (`Arc::ptr_eq`): used as it is, a full upload;
2. else an image whose set has the same layout as this one (same anchor,
   level lengths, slot count, centres): the prepared image (a landing that
   replayed edits made while it was built) or the last one. Diffed against
   it, slot by slot: a slot whose column is unchanged keeps its materials
   and sheets; records and light are compared word for word; the reach tops
   are worked out again from the cached ring count. The delta names what
   differs from the last image's version;
3. else built whole (the first set after a load, tests).

Stable order: ranges ascend, slots in tier order. Nothing iterates a hash map.

### D3. The landing builds its image on its task

The task that builds the set, bakes its bounce and prepares its contact tier
also builds its `TierImage`, with the `reflection_reach_m` the request was
made with. It travels in `Held` with the set and goes to
`PreparedGpu` (a resource) when the set lands; `pack_fine_gpu` takes it.

### D4. The upload writes ranges, and only on its base

`upload_fine` uploads when `FineGpu.version == PlanetFine.version` (both
extracted from the same frame). A delta is used only when
- `delta.base == planet.uploaded` (the GPU holds exactly the base), and
- the landing is in place (not a dissolve into the other half);
the light delta only when the light goes into the newest half, which holds
the base's light; a relight into the other half (`baked-light-ease` 7.3)
writes the light whole. Everything else writes whole. The writes themselves
go through one function, `tier_writes`, which a test applies to byte arrays.

Ranges closer than 16 slots (or 64 records) are merged, so an edit's relit
neighbourhood is a few writes, not hundreds.

### D5. No torn generation, no overflow

All writes of a version are queued in one `upload_fine` call, before the
frame's submit, as today. Sizes are clamped as today (`LIGHT_HALF_WORDS`,
the buffers' capacity, `FINE_CAPACITY` for the counts); a range is clipped to
what the buffer can hold.

## Measured

M4 Max, release, windowed, `--no-vsync`, `--frame-log`, `--time 10`, fresh
throwaway worlds; before `71b7868c` (`claude/cascade-perf`), after this
branch; interleaved, two repeats, nothing else running. The screen was locked
for the sitting, so the perf suite's frame-time A/B was not run (it refuses);
these are CPU times of the upload itself (`UPLOAD_PARTS`, `TIER_IMAGE`, debug
level), which a locked screen does not pace.

Scenarios: `cascade-walk` (`--walk --at 23.2255 -3.012 --yaw 0 --pitch -10
--walk-distance 250`: 3 digs, 11-12 water ticks, 9-10 landings), `walk`
(`--walk-distance 600`, landings only), `stand` (`--walk --at 23.2255 -3.012
--stand 20`, the first set).

The render thread per version (`render: fine-set upload`), p50 (max):

| | before | after |
| --- | ---: | ---: |
| a landing, `walk` | 8.9 (9.9) ms | **3.3 (4.1) ms** |
| a landing, `cascade-walk` | 8.5 (10.4) ms, every version alike | **3.2 (4.0) ms** |
| an edit or a water tick, `cascade-walk` | the same 8.5 ms | **0.02 (0.02) ms**, 1-4 writes, a few KiB |
| the first set, `stand` | 7.7-8.1 ms | 2.6 ms |

Before, by part (`cascade-walk` p50): the levels' cell records written
2.8-3.0 ms, records with reach 0.27 ms, light packed 2.3 ms and written
0.15 ms, materials and sheets packed and written 2.8-2.9 ms. (The
`cascade-perf` proposal reads the first lap as "records"; it is the levels.)

After, a landing still writes its 34-35 MiB in 9 writes; what is left
(3.2-3.4 ms) is wgpu copying the levels' cell records into its staging, the
size of the set. An edit writes 12 KiB of level records and under 1 KiB of
records, light and materials.

The main thread (`pack_fine_gpu`): 0.00 ms for a landing (its image came
from its task), 1.34 ms p50 (1.57 max) to diff an edit's or a tick's
version (the levels compared on a thread each), 2.1-2.3 ms for the first set
after a load. An edit's own cost (clone, relight, contact) is unchanged,
11.8-12.9 ms before and after.

The same picture, `--capture` before against after, 2880x1800: the cascade
(`--walk --at 23.2255 -3.012 --yaw 0 --pitch -10`) 0 of 5,184,000 pixels
differ; Ashenstead (`--walk --at 19.644 59.178`) 0; a dig at the spawn
(`--walk --pitch -60 --break 4 --frames 500`, edit deltas) 0. A pour
(`--walk --pitch -30 --yaw 90 --pour 3 --pour-ahead 12 --pit 9 --frames
700`, water-tick deltas) comes out one of two ways run to run on either exe
(68 pixels apart, by 1 level): before gave A, A, B, after B, A, A, and every
after capture is byte-identical to a before capture.

Next: a landing's level records could be written into a mapped staging
buffer on its task and copied on the GPU, leaving the render thread nothing
of the 3.2 ms. Not done here.

## Risks

- A producer added later that bypasses `pack_fine_gpu`: impossible by
  construction, the system reads `PlanetFine` itself. A version it has not
  packed is never drawn half: the render waits a frame.
- The diff costs the main thread: measured in task 3 against what the
  render thread no longer spends.
