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
they lie. `Light` gets `#[repr(transparent)]` and the image copies the bytes
(`bytemuck`). A big-endian target keeps the loop. Pinned by a test that the
cast and the loop agree on a baked tier.

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

Filled in by the implementation (task 3).

## Risks

- A producer added later that bypasses `pack_fine_gpu`: impossible by
  construction, the system reads `PlanetFine` itself. A version it has not
  packed is never drawn half: the render waits a frame.
- The diff costs the main thread: measured in task 3 against what the
  render thread no longer spends.
