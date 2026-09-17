# Tasks

## 1. Prove the seam before building the tiers
- [ ] Prototype the band boundary: two adjacent LOD levels on the real topology,
      in a still frame, at the worst angle. A mockup before a large feature is
      this repository's rule, and a seam is what a still frame settles.
- [ ] Pick the selection rule from `design.md` on that evidence.

## 2. Per-level topology, uploaded once
- [ ] Build corner rings for every level from the finest down, and confirm the
      measured total is about `4/3` of the finest level alone.
- [ ] Confirm against the real `dual_sphere` that cell `i` is the same direction
      at every level that contains it, as a test rather than a comment.
- [ ] Coarse levels index the existing height array; no per-level height data.

## 3. GPU level selection and culling, one-way
- [ ] Extend `planet_visibility.wgsl` to write a level per visible tile
      alongside the compaction it already does. One pass, not two.
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
