# Tasks

## 1. The column, in the core

- [x] `pbd_core::column`: a fixed 320-layer span over the measured relief,
      generated from `surface_altitude` and the existing material rule so the
      surface has one source across both tiers.
- [x] Bedrock at the bottom entry, never mineable.
- [x] The 3D carve: ridged noise on `gnoise3d_seed`, thresholded so tunnels
      connect, tightening toward the surface so the ground is not lace.
- [x] Tests: the top of the column agrees with `surface_altitude` to the metre;
      bedrock is solid everywhere; a carve leaves connected runs rather than
      isolated bubbles; the same cell generates the same column every time.

## 2. Collision

- [ ] `Column::contact` answers the top of the solid RUN, not the layer: a
      point inside a three-layer wall reports the wall top. Core, one test.
- [ ] `PlanetContact::stand(position) -> Stand { floor, ceiling, water }`,
      column-aware inside the tier through the finest record index, the
      heightfield everywhere else. One function decides, so the walker never
      learns that two representations exist.
- [ ] The walker: a footprint takes the MIN ceiling as it takes the MAX floor;
      a candidate without headroom is a wall; the head clamps to the ceiling
      after the sweep, with only the rise zeroed underwater.
- [ ] Tests: standing on a cave floor, a jump under a roof stopping at the
      roof, a 1.5 m gap impassable, a cave meeting the tier's rim blocking as
      a wall, and every terrace scenario unchanged.
- [ ] Not ported, and said so: Tenebris's containment resolve and rescue stack.
      The swept five-point footprint is what makes them unnecessary.

## 2b. A way in

- [x] Measure the mouths: `cave_mouths` counts 2,530 caves in the tier and 0
      open to the surface, because the carve is damped to nothing over the top
      `roof_m` of every column. Collision alone cannot let anyone walk into a
      cave; there is nothing to walk into.
- [ ] A mouth rule in the carve - a seeded, rare patch on a hillside where the
      surface damping is lifted - written up as its own change and measured
      with `cave_mouths` before its code.

## 3. Rendering

- [x] The column tier: a sub-band of `reach_m` inside level 11, generated on the
      fine set's own async task, because one column is 22.3 us and the whole
      band would be 0.91 s of it.
- [x] Pack a column as four runs plus its six neighbours' slots, and carry the
      slot in the cell record, so the pass needs no second list.
- [x] A fifth indirect draw: a cave ceiling, a cave floor, and a flank per run
      per stretch of the **neighbour's** air - one quad over the widest stretch
      leaves the rest as a window. 864 vertices, additive to the terrain's 60.
- [x] The tier's rim is generated SOLID, because a face on the rim's outward
      side is culled and an off-tier cell draws nothing under its cap.
- [x] Measure the frame cost in the tier against the heightfield it adds to.

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

- [x] Captures: standing inside a cave (`--view cave`) and under rock that
      stands over open air (`--view overhang`), both in `docs/screenshots`.
- [ ] A dug hole, which waits on mining.
- [ ] The owner walks into one. Worldgen and collision have both passed headless
      while being wrong in the running game before.

## 6. Held

- [ ] Baked voxel light. A cave lit by surface sky occlusion is wrong inside;
      `cave_dark` is the stand-in and says so.
- [ ] Skip a SEALED column when the camera is above ground. A column whose air
      gaps all lie below every neighbour's cap cannot be seen from outside, and
      the tier is about 30% of a surface frame on the software rasteriser -
      every vertex of it invisible. The flag is computable when the tier is
      built.
- [ ] The streamed chunk store of `voxel-engine-foundation`, still the standing
      design for a planet dug to its core.
