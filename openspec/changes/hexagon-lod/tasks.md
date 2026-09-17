# Tasks

## 1. Prove the seam before building the tiers
- [ ] Prototype the band boundary: two adjacent LOD levels on the real topology,
      in a still frame, at the worst angle. A mockup before a large feature is
      this repository's rule, and a seam is what a still frame settles.
      Taken as the first capture of the built partition rather than a
      separate mockup, on the owner's instruction to build.
- [x] Pick the selection rule: partition by the coarser level's cells, with
      midpoint cells split per fragment, fine floors and cut walls closing the
      seam. `design.md`, "Implementation decisions".

## 2. Per-level topology
- [ ] A lattice generator: position by `(face, i, j, level)` through the
      recursive midpoint construction, bit-identical to `dual_sphere`, with a
      test that says so on the shared points.
- [ ] A local dual builder over the level-`L` triangles intersecting a cap,
      reusing `dual_sphere`'s dual construction; a test that every cell inside
      the cap has a complete ring and the measured tile width at level 11 on
      4,800 m is 2.833 m.
- [ ] Records carry level, two owner directions and six fine floors; the base
      level is uploaded once and the fine levels are regenerated per anchor.

## 3. GPU level selection and culling, one-way
- [ ] Extend `planet_visibility.wgsl` with the partition rule: a tile draws
      when it is not fine itself and its owner is; midpoint cells whose owners
      disagree draw and split per fragment. One pass, not two.
- [ ] Walls to the fine floor on a fine side, the cut wall along a midpoint
      cell's diagonal, and the per-fragment owner test in the surface shader.
- [ ] Keep the draw indirect and the readback at zero. Nothing about a tile
      returns to the CPU.
- [ ] Keep the 32-group coarse cull (12 icosahedron vertices plus 20 face
      centres) if it measures better than testing every tile.

## 4. The far tier is hexagons
- [ ] Draw the far tier from the same vertex-pulling path and the same shading
      terms as the near tier.
- [ ] Do not port `distant.fs.glsl` or any triangle impostor.

## 5. The culling the pass already pays for but does not do

Found while exploring, independent of LOD and cheap. These belong here rather
than in their own change because they touch `planet_visibility.wgsl`, and
touching one pass twice for two reasons is exactly the divergent-path risk this
project's rules warn about.

- [x] **Frustum cull.** `params.clip_from_world` was declared and BOUND in the
      visibility pass while `compact_visible` never read it, so the only test was
      the sphere-horizon one: standing on the ground submitted the whole visible
      hemisphere, about 327,000 cells. The pass now tests body-local cap/wall
      bounding spheres against all six homogeneous clip planes and retains
      intersecting bounds conservatively.
- [x] **Per-cell foliage distance.** `FOLIAGE_DRAW_CUTOFF_ALTITUDE` was one
      global altitude switch, so below 3,200 m every visible cell was submitted
      at 162 vertices and the tree branch discarded beyond 2,300 m *after*
      submission, as degenerate triangles - about 108 wasted tree vertices per
      cell across hundreds of thousands of cells. Nearby vegetated cells now
      compact into a separate ID buffer and a 108-vertex indirect foliage draw,
      while terrain uses 54. The compute pass owns the only foliage eligibility
      test, keeping the 3,200 m altitude gate as a conservative early disable and
      giving the GPU the per-cell 2,300 m range and material/seed predicate.
      There is no runtime GPU readback.
- [ ] Measure both, before and after, in a reproducible release scene. Neither is
      a speedup until it has a number.

Tier selection, the seam and performance measurement remain open; this covers
only the two culling fixes, which were independent of LOD.

## 6. Then the radius
- [ ] With LOD landed, revisit the radius in
      `preview-scale-and-shader-parity`. The resident cost stops being
      `10*4^L + 2` for the whole globe, so the ladder stops being the
      constraint that picks the planet size.
