## Why

`cascade-perf` found the walking spikes (proposal, "Spikes, walking along
it"): every new fine-set version costs the render thread 8-11 ms in
`render: fine-set upload` on the M4 Max, for the whole tier, every time:

- the four levels' records written (`levels`);
- the column records with their reach tops worked out (`records`, 3-4.5 ms);
- the light field packed (`light`, 2.2-3.3 ms, 387,828 words);
- the materials and placed sheets packed and written (`materials`,
  2.8-3.3 ms);
- the centres.

A landing makes a version every 3 s or so on foot; so does every edit, every
water tick that moves a cell and the dusk relight. 8-11 ms on the render
thread is a missed 60 Hz frame each time. Nothing about the picture needs it
done there: all of it is a function of the set, which is immutable once it is
shared.

Every cut here keeps the picture and the sound (CLAUDE.md, "Performance
improvements need no permission"): the GPU buffers end up holding the same
bytes, in the same frame, as before.

## What changes

1. **The light field goes up as it is stored.** `gpu_light` packs four
   one-byte levels to a word, layer order, low byte first, which is the
   little-endian layout of the `[Light; LAYERS]` arrays the field already
   is. The packing loop becomes a cast: no work at all.
2. **The tier's GPU image is made where the set is made.** A `TierImage`
   (records with reach tops, light words, materials, placed sheets, centres)
   is worked out:
   - for a landing, on the landing's own task, beside the contact tier it
     already prepares there;
   - for anything else (an edit, a water tick, a relight, a replay), in the
     main world's `Last`, after every producer of the frame, from the image
     of the version before it: only the slots whose column or light differ
     are packed again.
   The render thread copies; it works nothing out.
3. **Only what changed goes up.** When the new version has the same layout
   as the one the GPU holds (the same anchor, records and slots: an edit, a
   water tick, a relight), the image carries the slot ranges that differ
   from that version, per buffer, and the upload writes those sub-ranges
   with `write_buffer`. A landing, a fresh start, a version the GPU did not
   hold, or a relight into the other light half writes everything, as now.
4. **One generation at a time.** All of a version's writes go into the
   queue in the one frame, as before, so no frame draws half of one. The
   render world uploads a version only with its own image; a set without
   its image (never expected) waits rather than draws half-updated.

No buffer layout, binding or shader changes.

## Measured

See `design.md`, "Measured", and `docs/benchmarks/2026-11-17-fine-set-upload/`.
