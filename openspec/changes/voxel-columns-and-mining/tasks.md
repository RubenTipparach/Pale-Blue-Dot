# Tasks

## 1. The column, in the core

- [ ] `pbd_core::column`: a fixed 320-layer span over the measured relief,
      generated from `surface_altitude` and the existing material rule so the
      surface has one source across both tiers.
- [ ] Bedrock at the bottom entry, never mineable.
- [ ] The 3D carve: ridged noise on `gnoise3d_seed`, thresholded so tunnels
      connect, tightening toward the surface so the ground is not lace.
- [ ] Tests: the top of the column agrees with `surface_altitude` to the metre;
      bedrock is solid everywhere; a carve leaves connected runs rather than
      isolated bubbles; the same cell generates the same column every time.

## 2. Collision

- [ ] `ColumnContact` - the floor at or below a point and the ceiling above it.
- [ ] One function decides column tier or heightfield, so the walker never
      learns that two representations exist.
- [ ] Tests: standing on a cave floor, a head bump on its roof, walking in at a
      mouth, and the heightfield answer unchanged outside the band.

## 3. Rendering

- [ ] Draw a cap and walls per solid run, four runs at 240 vertices, the largest
      four when a column has more.
- [ ] Measure the frame cost in the band against the heightfield it replaces.

## 4. Mining and placing

- [ ] The ray march to the first solid layer.
- [ ] Break gives the block to the slots; place takes from the selected slot and
      refuses where the player stands.
- [ ] **Durable on the same input frame**, per the repo's non-negotiable rule,
      with the edit store keyed by cell so an edit outlives the record slot.
- [ ] The first tool, which is the first equipment worth a slot.
- [ ] Tests: break then place restores the column; an edit survives a reload; an
      edit outside the band is not lost when the band moves.

## 5. Prove it

- [ ] Captures: standing inside a cave, under an overhang, and a dug hole.
- [ ] The owner walks into one. Worldgen and collision have both passed headless
      while being wrong in the running game before.

## 6. Held

- [ ] Baked voxel light. A cave lit by surface sky occlusion is wrong inside.
- [ ] The streamed chunk store of `voxel-engine-foundation`, still the standing
      design for a planet dug to its core.
